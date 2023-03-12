#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod constants;
pub mod secrets;
pub mod api_calls;
pub mod file_operations;
pub mod file_name_recognition;
pub mod rss_parser;
pub mod recommendation;
pub mod file_name_recognition_tests;

#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use file_name_recognition_tests::FilenameTest;
use regex::Regex;
use rss_parser::RssEntry;
use serde::{Serialize, Deserialize};
use tauri::async_runtime::Mutex;
use tauri::Manager;
use window_titles::{Connection, ConnectionTrait};
use std::{collections::HashMap, path::Path, time::{Duration, Instant, SystemTime, UNIX_EPOCH}, thread};
use open;
use api_calls::{TokenData, UserSettings, AnilistDate};
use crate::{api_calls::{AnimeInfo, UserAnimeInfo}, file_name_recognition::AnimePath};



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RefreshUI {
    pub anime_list: bool,
    pub tracking_progress: bool,
    pub canvas: bool,
}

impl RefreshUI {
    
    pub fn clear(&mut self) {
        self.anime_list = false;
        self.tracking_progress = false;
        self.canvas = false;
    }
}


lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_DATA: Mutex<HashMap<i32, UserAnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_LISTS: Mutex<HashMap<String, Vec<i32>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_SETTINGS: Mutex<UserSettings> = Mutex::new(UserSettings::new());
    static ref GLOBAL_ANIME_PATH: Mutex<HashMap<i32,HashMap<i32,AnimePath>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_REFRESH_UI: Mutex<RefreshUI> = Mutex::new(RefreshUI::default());
    static ref GLOBAL_UPDATE_ANIME_DELAYED: Mutex<HashMap<i32, Instant>> = Mutex::new(HashMap::new());
}

// takes a oauth code from the user and exchanges it for a oauth access token
#[tauri::command]
async fn anilist_oauth_token(code: String) -> (bool, String) {
    
    let token = api_calls::anilist_get_access_token(code).await;
    let combine = format!("{}\n{}", token.token_type, token.access_token);

    if token.access_token.len() == 0 {
        return (false, combine);
    }
    else {
        *GLOBAL_TOKEN.lock().await = token;
    }

    write_token_data().await;
    
    (true, String::new())
}

// save token data to a file
#[tauri::command]
async fn write_token_data() {
    file_operations::write_file_token_data().await;
}

// get all data for a specific anime
#[tauri::command]
async fn get_anime_info_query(id: i32) -> api_calls::AnimeInfo {
    
    let response = api_calls::anilist_api_call(id).await;    
    print!("{}", response.id);
    response
}

// sets the user's settings taken from the settings ui
#[tauri::command]
async fn set_user_settings(settings: UserSettings) {

    let mut user_settings = GLOBAL_USER_SETTINGS.lock().await;

    // check if the folders have changed
    let mut scan = false;
    if user_settings.folders.len() != settings.folders.len() {
        scan = true;
    } else {
        for i in 0..settings.folders.len() {
            if settings.folders[i] == "" {
                continue;
            }
            if user_settings.folders[i] != settings.folders[i] {
                scan = true;
            }
        }
    }

    let old_current_tab = user_settings.current_tab.clone();
    let score_format = user_settings.score_format.clone();
    let old_username = user_settings.username.clone();
    *user_settings = settings;
    user_settings.score_format = score_format;
    user_settings.current_tab = old_current_tab;

    if old_username != user_settings.username {
        GLOBAL_USER_ANIME_LISTS.lock().await.clear();
        GLOBAL_USER_ANIME_DATA.lock().await.clear();

        user_settings.score_format = api_calls::get_user_score_format(user_settings.username.clone()).await;
    }

    drop(user_settings);
    if scan {
        file_name_recognition::parse_file_names(None).await;
    }

    file_operations::write_file_user_settings().await;
}



// retrieves user's settings from a file
#[tauri::command]
async fn get_user_settings() -> UserSettings {
    load_user_settings().await;
    GLOBAL_USER_SETTINGS.lock().await.clone()
}

// retrieves user's settings from a file
#[tauri::command]
async fn set_current_tab(current_tab: String) {
    GLOBAL_USER_SETTINGS.lock().await.current_tab = current_tab;
    file_operations::write_file_user_settings().await;
}

// gets anime data for all anime in a specific list
#[tauri::command]
async fn get_list(list_name: String) -> (Vec<AnimeInfo>, Option<String>) {

    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        if error_message.is_some() {
            return (Vec::new(), error_message)
        }
        file_operations::write_file_anime_info_cache().await;
        file_operations::write_file_user_info().await;
    }
    
    let anime_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = anime_lists.get(&list_name).unwrap();

    let anime_data = GLOBAL_ANIME_DATA.lock().await;

    let mut list_info: Vec<AnimeInfo> = Vec::new();
    for id in list {
        list_info.push(anime_data.get(id).unwrap().clone());
    }

    (list_info, None)
}


#[tauri::command]
async fn get_list_paged(list_name: String, sort: String, ascending: bool, page: usize) -> (Vec<(AnimeInfo, UserAnimeInfo)>, Option<String>){

    if GLOBAL_USER_SETTINGS.lock().await.username.is_empty() {
        return (Vec::new(), None);
    }

    // get list from anilist if it does not exist
    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        if error_message.is_some() {
            //println!("{}", error_message.unwrap());
            return (Vec::new(), Some(error_message.unwrap()));
        }
        file_operations::write_file_anime_info_cache().await;
        file_operations::write_file_user_info().await;
    }

    let mut anime_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = anime_lists.get_mut(&list_name).unwrap();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let user_data = GLOBAL_USER_ANIME_DATA.lock().await;

    // before showing the list sort the contents by the currently selected sorting category
    if page == 0 {
        match sort.as_str() {
            "Alphabetical" => {
                let user_settings = GLOBAL_USER_SETTINGS.lock().await;
                match user_settings.title_language.as_str() {
                    "romaji" => list.sort_by(|i, j| { anime_data.get(i).unwrap().title.romaji.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.romaji.clone().unwrap().to_lowercase()).unwrap() }),
                    "english" => list.sort_by(|i, j| { 
                        if anime_data.get(i).unwrap().title.english.is_none() && anime_data.get(j).unwrap().title.english.is_none() {
                            anime_data.get(i).unwrap().title.romaji.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.romaji.clone().unwrap().to_lowercase()).unwrap()
                        } else if anime_data.get(i).unwrap().title.english.is_none() {
                            anime_data.get(i).unwrap().title.romaji.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.english.clone().unwrap().to_lowercase()).unwrap()
                        } else if anime_data.get(j).unwrap().title.english.is_none() {
                            anime_data.get(i).unwrap().title.english.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.romaji.clone().unwrap().to_lowercase()).unwrap()
                        } else {
                            anime_data.get(i).unwrap().title.english.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.english.clone().unwrap().to_lowercase()).unwrap() 
                        }
                    }),
                    "native" => list.sort_by(|i, j| { anime_data.get(i).unwrap().title.native.clone().unwrap().to_lowercase().partial_cmp(&anime_data.get(j).unwrap().title.native.clone().unwrap().to_lowercase()).unwrap() }),
                    &_ => (),
                }
            },
            "Score" => list.sort_by(|i, j| { anime_data.get(i).unwrap().average_score.partial_cmp(&anime_data.get(j).unwrap().average_score).unwrap() }),
            "MyScore" => list.sort_by(|i, j| { user_data.get(i).unwrap().score.partial_cmp(&user_data.get(j).unwrap().score).unwrap() }),
            "Date" => list.sort_by(|i, j| { anime_data.get(i).unwrap().start_date.partial_cmp(&anime_data.get(j).unwrap().start_date).unwrap() }),
            "Popularity" => list.sort_by(|i, j| { anime_data.get(i).unwrap().popularity.partial_cmp(&anime_data.get(j).unwrap().popularity).unwrap() }),
            "Trending" => list.sort_by(|i, j| { anime_data.get(i).unwrap().trending.partial_cmp(&anime_data.get(j).unwrap().trending).unwrap() }),
            "Started" => list.sort_by(|i, j| { user_data.get(i).unwrap().started_at.partial_cmp(&user_data.get(j).unwrap().started_at).unwrap() }),
            "Completed" => list.sort_by(|i, j| { user_data.get(i).unwrap().completed_at.partial_cmp(&user_data.get(j).unwrap().completed_at).unwrap() }),
            &_ => (),
        }
        if ascending == false {
            list.reverse();
        }

        // check for next airing episode that is in the past and update it with a new time
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i32;
        let mut get_airing_time_ids: Vec<i32> = Vec::new();
        for anime_id in &mut *list {
            let anime_info = anime_data.get(anime_id).unwrap();
            if anime_info.next_airing_episode.is_some() {

                let airing_at = anime_info.next_airing_episode.as_ref().unwrap().airing_at;

                if airing_at < current_time {
                    get_airing_time_ids.push(*anime_id);
                }
            }
        }
        drop(anime_data);
        api_calls::anilist_airing_time(get_airing_time_ids).await;
    }
    let anime_data = GLOBAL_ANIME_DATA.lock().await;

    let start = page * constants::ANIME_PER_PAGE;
    let finish = 
    if (page + 1) * constants::ANIME_PER_PAGE > list.len() {
        list.len()
    } else {
        (page + 1) * constants::ANIME_PER_PAGE
    };

    let mut list_info: Vec<(AnimeInfo, UserAnimeInfo)> = Vec::new();
    for i in start..finish {
        let id = list.get(i).unwrap();
        list_info.push((anime_data.get(id).unwrap().clone(), user_data.get(id).unwrap().clone()));
    }

    (list_info, None)
}

// get all user data for all anime in a specific list
#[tauri::command]
async fn get_list_user_info(list_name: String) -> Vec<UserAnimeInfo> {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    let mut list: Vec<UserAnimeInfo> = Vec::new();
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    for item in GLOBAL_USER_ANIME_LISTS.lock().await.entry(list_name.clone()).or_insert(Vec::new()) {

        list.push(user_data.entry(*item).or_insert(UserAnimeInfo::new()).clone());
    }

    list
}

// get user info for a specific anime
#[tauri::command]
async fn get_user_info(id: i32) -> UserAnimeInfo {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    GLOBAL_USER_ANIME_DATA.lock().await.entry(id).or_insert(UserAnimeInfo::new()).clone()
}

#[tauri::command]
async fn get_delay_info() -> (f64, i32, String, i64) {

    let found_anime = WATCHING_TRACKING.lock().await.clone();
    
    if found_anime.len() > 0 {

        let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * constants::SECONDS_IN_MINUTES) as f64;
        
        let (_, anime) = found_anime.iter().next().unwrap();
        
        return (anime.timer.elapsed().as_secs_f64() / delay, anime.episode + (anime.length - 1), anime.title.clone(), (delay as i64) - (anime.timer.elapsed().as_secs() as i64));
    }

    return (0.0, 0, String::new(), 0)
}

// get data for a specific anime
#[tauri::command]
async fn get_anime_info(id: i32) -> AnimeInfo {

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }

    if GLOBAL_ANIME_DATA.lock().await.contains_key(&id) == false {
        api_calls::anilist_get_anime_info_single(id).await;
    }

    let anime_data = GLOBAL_ANIME_DATA.lock().await.get(&id).unwrap().clone();
    anime_data
}

// updates a entry on anilist with new information
#[tauri::command]
async fn update_user_entry(mut anime: UserAnimeInfo) {

    let old_status: String = if GLOBAL_USER_ANIME_DATA.lock().await.contains_key(&anime.media_id) {
        GLOBAL_USER_ANIME_DATA.lock().await.entry(anime.media_id).or_default().status.clone()
    } else {
        String::new()
    };

    let old_list: String = if old_status == "REPEATING" {
        String::from("CURRENT")
    } else {
        old_status.clone()
    };
    
    let new_list = if anime.status == "REPEATING" {
        String::from("CURRENT")
    } else {
        anime.status.clone()
    };

    // we need to change what list the anime is in
    if old_list != new_list {
        
        // if the anime is not newly added
        if old_status.is_empty() == false {

            GLOBAL_USER_ANIME_LISTS.lock().await.entry(old_list.clone()).and_modify(|data|{ 
                data.remove(data.iter().position(|&v| v == anime.media_id).unwrap());
            });
        }
        
        let mut lists = GLOBAL_USER_ANIME_LISTS.lock().await;
        if lists.contains_key(&new_list) {
            
            let list = lists.entry(new_list.clone()).or_default();
            if list.len() > 0 {

                if list.contains(&anime.media_id) == false {
        
                    list.push(anime.media_id);
                }
            }
        }
    }

    if new_list == "COMPLETED" {
        let now: DateTime<Local> = Local::now();
        anime.completed_at = Some(AnilistDate {
            year: Some(now.year()),
            month: Some(now.month() as i32),
            day: Some(now.day() as i32),
        });

        if anime.started_at.is_none() {
            anime.started_at = anime.started_at.clone();
        } else {
            let started_at = anime.started_at.clone().unwrap();
            if started_at.day.is_none() && started_at.month.is_none() && started_at.year.is_none() {
                anime.started_at = anime.started_at.clone();
            }
        }
    }

    let response = api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), anime).await;

    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let new_info: UserAnimeInfo = serde_json::from_value(json["data"]["SaveMediaListEntry"].to_owned()).unwrap();
    let media_id = new_info.media_id.clone();

    let mut anime_data = GLOBAL_USER_ANIME_DATA.lock().await;
    if anime_data.contains_key(&media_id) {

        anime_data.entry(media_id).and_modify(|entry| {
            *entry = new_info;
        });
        drop(anime_data);
        file_operations::write_file_user_info().await;
    } else {
        anime_data.insert(media_id, new_info);
    }
}

// loads data from files and looks for episodes on disk
#[tauri::command]
async fn on_startup() {
    file_operations::read_file_token_data().await;
    file_operations::read_file_anime_info_cache().await;
    file_operations::read_file_user_info().await;
    file_operations::read_file_episode_path().await;
    if GLOBAL_USER_SETTINGS.lock().await.score_format.is_empty() {
        GLOBAL_USER_SETTINGS.lock().await.score_format = api_calls::get_user_score_format(GLOBAL_USER_SETTINGS.lock().await.username.clone()).await;
    }
    *GLOBAL_STARTUP_FINISHED.lock().await = true;
}



lazy_static! {
    static ref GLOBAL_SETTINGS_LOADED: Mutex<bool> = Mutex::new(false);
    static ref GLOBAL_STARTUP_FINISHED: Mutex<bool> = Mutex::new(false);
}
// loads data from files and looks for episodes on disk
#[tauri::command]
async fn load_user_settings() {

    let mut loaded = GLOBAL_SETTINGS_LOADED.lock().await;
    if *loaded == false {
        file_operations::read_file_user_settings().await;
        *loaded = true;
    }
    let mut user_settings = GLOBAL_USER_SETTINGS.lock().await;
    if user_settings.highlight_color.is_empty() {
        user_settings.highlight_color = String::from(constants::DEFAULT_HIGHLIGHT_COLOR);
    }
    if user_settings.show_airing_time.is_none() {
        user_settings.show_airing_time = Some(true);
    }
    if user_settings.theme.is_none() {
        user_settings.theme = Some(0);
    }
}

// go ahead with any updates that haven't been completed yet before closing
#[tauri::command]
async fn on_shutdown() {

    check_delayed_updates(false).await;
}

// check if enough time has passed before updating the episode of a anime
// this delay is to prevent spamming or locking when the user increases or decreases the episode count multiple times
async fn check_delayed_updates(wait: bool) {
    
    let delayed_update = GLOBAL_UPDATE_ANIME_DELAYED.lock().await.clone();
    for (id, time) in delayed_update.iter() {
        
        if time.elapsed() >= Duration::from_secs(constants::ANIME_UPDATE_DELAY) || wait == false {

            api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), GLOBAL_USER_ANIME_DATA.lock().await.get(id).unwrap().clone()).await;
        }
    }
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.retain(|_, v| v.elapsed() < Duration::from_secs(constants::ANIME_UPDATE_DELAY));
}

// opens the file for the next episode in the default program
#[tauri::command]
async fn play_next_episode(id: i32) {
    
    let next_episode = GLOBAL_USER_ANIME_DATA.lock().await.get(&id).unwrap().progress + 1;

    if play_episode(id, next_episode).await == false {
        file_name_recognition::parse_file_names(None).await;
        play_episode(id, next_episode).await;
    }
}

// play the episode from the anime id
// returns true if the episode was played
async fn play_episode(anime_id: i32, episode: i32) -> bool {

    let mut episode_opened = false;
    let paths = GLOBAL_ANIME_PATH.lock().await;
    if paths.contains_key(&anime_id) {
        let media = paths.get(&anime_id).unwrap();
        if media.contains_key(&episode) {
            
            let next_episode_path = Path::new(&media.get(&episode).unwrap().path);
            match open::that(next_episode_path) {
                Err(why) => panic!("{}",why),
                Ok(_e) => { episode_opened = true },
            }
        }
    }

    episode_opened
}

// changes the progress for a anime by +-1
// anilist api call is delayed to prevent spam/locking
#[tauri::command]
async fn increment_decrement_episode(anime_id: i32, change: i32) {

    if change.abs() != 1 {
        return;
    }

    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    if user_data.contains_key(&anime_id) {

        let progress = user_data.get(&anime_id).unwrap().progress;

        let anime_data = GLOBAL_ANIME_DATA.lock().await;
        if anime_data.get(&anime_id).unwrap().episodes.is_some() {
            let max_episodes = anime_data.get(&anime_id).unwrap().episodes.unwrap();
            if change == 1 && progress == max_episodes {
                return;
            }
        }

        if change == -1 && progress == 0 {
            return;
        }

        let anime = user_data.get_mut(&anime_id).unwrap();
        change_episode(anime, anime.progress + change, anime_data.get(&anime.media_id).unwrap().episodes).await;

        let mut delayed_update = GLOBAL_UPDATE_ANIME_DELAYED.lock().await;
        if delayed_update.contains_key(&anime_id) {
            delayed_update.entry(anime_id).and_modify(|entry| {
                *entry = Instant::now();
            });
        } else {
            delayed_update.insert(anime_id, Instant::now());
        }
    }
    drop(user_data);
    file_operations::write_file_user_info().await;
}

// scan folders for episodes of anime
#[tauri::command]
async fn scan_anime_folder() {
    file_name_recognition::parse_file_names(None).await;
}

#[derive(Debug, Clone)]
struct WatchingTracking {
    timer: std::time::Instant,
    monitoring: bool,
    episode: i32,
    length: i32,
    title: String,
}
lazy_static! {
    static ref WATCHING_TRACKING: Mutex<HashMap<i32, WatchingTracking>> = Mutex::new(HashMap::new());
}



// get the titles of all active windows
fn get_titles() -> Vec<String> {
    let connection = Connection::new();
    let titles: Vec<String> = connection.unwrap().window_titles().unwrap();
    titles
}



// scans for and identifies windows playing anime and sets up a delayed update
#[tauri::command]
async fn anime_update_delay() {

    let regex = Regex::new(r"\.mkv|\.avi|\.mp4").unwrap();

    let vlc_remove = Regex::new(r" - VLC media player").unwrap();
    let gom_remove = Regex::new(r" - GOM Player").unwrap();
    let zoom_remove = Regex::new(r" - Zoom Player MAX").unwrap();
    let mpv_remove = Regex::new(r" - mpv").unwrap();
    let pot_remove = Regex::new(r" - PotPlayer").unwrap();

    let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * constants::SECONDS_IN_MINUTES)  as u64;
    let language = GLOBAL_USER_SETTINGS.lock().await.title_language.clone();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut watching_data = WATCHING_TRACKING.lock().await;
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;

    // reset monitoring
    watching_data.iter_mut().for_each(|(_, entry)| {
        entry.monitoring = false;
    });

    let mut titles: Vec<String> = get_titles(); // for some reason mutex locking has to happen before this function
    titles.retain(|title| regex.is_match(title));

    for title in titles {

        // remove file extension
        let mut title_edit: String = regex.replace(&title, "").to_string();
        // remove video player suffixes 
        title_edit = vlc_remove.replace(&title_edit, "").to_string();
        title_edit = gom_remove.replace(&title_edit, "").to_string();
        title_edit = zoom_remove.replace(&title_edit, "").to_string();
        title_edit = mpv_remove.replace(&title_edit, "").to_string();
        title_edit = pot_remove.replace(&title_edit, "").to_string();
        //println!("{} {}", title, title_edit);
        title_edit = file_name_recognition::remove_brackets(&title_edit);
        // get the episode number from the filename
        let (episode_str, mut episode, length) = file_name_recognition::identify_number(&title_edit);
        // remove episode number from filename
        title_edit = title_edit.replace(episode_str.as_str(), "");
        // remove irrelevant information like source, final, episode prefix, etc
        title_edit = file_name_recognition::irrelevant_information_removal(title_edit);

        let (mut media_id, _, media_score) = file_name_recognition::identify_media_id(&title_edit, &anime_data, None);
        if media_score < constants::SIMILARITY_SCORE_THRESHOLD { 
            continue;
        }
        (media_id, episode) = file_name_recognition::replace_with_sequel(media_id, episode, &anime_data);

        file_name_recognition::episode_fix(media_id, &mut episode, &anime_data);

        let next_episode: bool = episode > user_data.get(&media_id).unwrap().progress && episode <= user_data.get(&media_id).unwrap().progress + length;

        // if the file is being monitored and the episode is the next episode
        if watching_data.contains_key(&media_id) && next_episode {
            watching_data.entry(media_id).and_modify(|entry| {
                if entry.episode == episode { 
                    entry.monitoring = true;
                }
            });
        } else if user_data.contains_key(&media_id) && 
            next_episode && 
            episode > 0 && episode <= anime_data.get(&media_id).unwrap().episodes.unwrap() { // only add if it is in the users list, it is the next episode, and the episode is within range

                let title = if language == "romaji" {
                    anime_data.get(&media_id).unwrap().title.romaji.clone().unwrap()
                } else if language == "english" {
                    anime_data.get(&media_id).unwrap().title.english.clone().unwrap()
                } else if language == "native" {
                    anime_data.get(&media_id).unwrap().title.native.clone().unwrap()
                } else {
                    String::from("language selection error")
                };

                watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode, length: length, title: title});
        }
    }

    let mut save_file = false;
    let mut update_entries: Vec<UserAnimeInfo> = Vec::new();
    // check if media has been playing for long enough to update
    for (media_id, tracking_info) in watching_data.iter_mut() {
        let seconds = tracking_info.timer.elapsed().as_secs();
        if seconds >= delay {
            // user progress will be updated to this episode so we no longer want to monitor it
            tracking_info.monitoring = false;
            
            // update anime
            let anime = user_data.get_mut(&media_id).unwrap();
            change_episode(anime, tracking_info.episode + tracking_info.length - 1, anime_data.get(media_id).unwrap().episodes).await;
            save_file = true;

            // store entry for later after mutexes are dropped
            update_entries.push(user_data.get(media_id).unwrap().clone());
            // update ui with episode progress
            GLOBAL_REFRESH_UI.lock().await.canvas = true;
        }
    }

    // remove episodes that are no longer being played
    watching_data.retain(|_, v| v.monitoring == true);

    // unlock mutexes before doing api calls which might block other threads
    drop(anime_data);
    drop(watching_data);
    drop(user_data);

    let access_token = GLOBAL_TOKEN.lock().await.access_token.clone();
    for anime in update_entries {
        // update anilist with new episode/status
        api_calls::update_user_entry(access_token.clone(), anime).await;
    }
    // update the file with the new episode/status
    if save_file {
        file_operations::write_file_user_info().await;
    }
}

// change the episode for the user entry and trigger other changes depending on the episode
async fn change_episode(anime: &mut UserAnimeInfo, episode: i32, max_episodes: Option<i32>) {

    let progress = anime.progress;
    anime.progress = episode;

    // set start date when the first episode is watched
    if progress == 0 && episode >= 1 {
        let now: DateTime<Local> = Local::now();
        anime.started_at = Some(AnilistDate {
            year: Some(now.year()),
            month: Some(now.month() as i32),
            day: Some(now.day() as i32),
        });
    }

    // add anime to watching if progress increases
    if episode > progress && (max_episodes.is_none() || episode != max_episodes.unwrap()) && anime.status != "CURRENT"{

        change_list(anime, String::from("CURRENT")).await;
        GLOBAL_REFRESH_UI.lock().await.anime_list = true;
    }

    // add anime to completed if the last episode was watched and set complete date
    if max_episodes.is_some() && episode == max_episodes.unwrap() {
        let now: DateTime<Local> = Local::now();
        anime.completed_at = Some(AnilistDate {
            year: Some(now.year()),
            month: Some(now.month() as i32),
            day: Some(now.day() as i32),
        });
        if anime.status != "COMPLETED" {

            change_list(anime, String::from("COMPLETED")).await;
        }
        GLOBAL_REFRESH_UI.lock().await.anime_list = true;
    }
}

async fn change_list(anime: &mut UserAnimeInfo, new_list: String) {

    let mut lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let old_list = if anime.status == "REPEATING" {
        String::from("CURRENT")
    } else {
        anime.status.clone()
    };

    lists.entry(old_list).and_modify(|list| {
        let index = list.iter().position(|v| *v == anime.media_id).unwrap();
        list.remove(index);
    });

    anime.status = new_list;
    let new_list = if anime.status == "REPEATING" {
        String::from("CURRENT")
    } else {
        anime.status.clone()
    };

    lists.entry(new_list).and_modify(|list| {
        
        if list.contains(&anime.media_id) == false {

            list.push(anime.media_id);
        }
    });
}



lazy_static! {
    static ref SCAN_TIMER: Mutex<Instant> = Mutex::new(Instant::now());
    static ref STARTUP_SCAN: Mutex<bool> = Mutex::new(false);
}
// allows the ui to check if a anime has been updated to determine if the ui will be refreshed
#[tauri::command]
async fn refresh_ui() -> RefreshUI {

    let mut refresh = GLOBAL_REFRESH_UI.lock().await;
    let mut refresh_ui = refresh.clone();
    refresh_ui.tracking_progress = WATCHING_TRACKING.lock().await.len() > 0;
    refresh.anime_list = false;
    refresh.canvas = false;

    refresh_ui
}



#[tauri::command]
async fn scan_files() {
    
    let one_hour = Duration::from_secs(constants::ONE_HOUR);
    let on_startup_delay = Duration::from_secs(constants::STARTUP_SCAN_DELAY);
    
    anime_update_delay().await;
    check_delayed_updates(true).await;

    let mut on_startup_scan_completed = STARTUP_SCAN.lock().await;
    let mut timer = SCAN_TIMER.lock().await;
    if timer.elapsed() > one_hour || (timer.elapsed() >= on_startup_delay && *on_startup_scan_completed == false) {
        if file_name_recognition::parse_file_names(None).await {

            GLOBAL_REFRESH_UI.lock().await.canvas = true;
        }
        *on_startup_scan_completed = true;
        *timer = Instant::now();
    }
}



// returns a list of what episodes of what anime exist on disk
#[tauri::command]
async fn episodes_exist() -> HashMap<i32, Vec<i32>> {
    
    let paths = GLOBAL_ANIME_PATH.lock().await;
    let mut episodes_exist: HashMap<i32, Vec<i32>> = HashMap::new();

    for (anime_id, episodes) in paths.iter() {

        episodes_exist.insert(*anime_id, Vec::new());

        for (episode, _) in episodes {

            episodes_exist.get_mut(anime_id).unwrap().push(*episode);
        }
    }
    episodes_exist
}

// returns a list of all episodes on disk for a anime
#[tauri::command]
async fn episodes_exist_single(id: i32) -> Vec<i32> {
    
    let paths = GLOBAL_ANIME_PATH.lock().await;

    let mut episodes_exist: Vec<i32> = Vec::new();
    if paths.contains_key(&id) {

        paths.get(&id).unwrap().keys().for_each(|key| {
            episodes_exist.push(*key);
        });
    }
    episodes_exist
}


#[tauri::command]
async fn browse(year: String, season: String, genre: String, format: String, search: String, order: String) -> Vec<AnimeInfo> {

    let mut list: Vec<AnimeInfo> = Vec::new();

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut has_next_page = true;

    let mut page = 0;
    while has_next_page {
        
        page += 1;
        let response = api_calls::anilist_browse_call(page, year.clone(), season.clone(), genre.clone(), format.clone(), search.clone(), order.clone()).await;

        for anime in response["data"]["Page"]["media"].as_array().unwrap() {

            let id = anime["id"].as_i64().unwrap() as i32;
            let anime_entry: AnimeInfo = serde_json::from_value(anime.clone()).unwrap();
            list.push(anime_entry.clone());
            anime_data.insert(id, anime_entry);
        }
        
        if page >= constants::BROWSE_PAGE_LIMIT {
            break;
        }
        has_next_page = response["data"]["Page"]["pageInfo"]["hasNextPage"].as_bool().unwrap();
    }


    drop(anime_data); // anime data is used by write function
    file_operations::write_file_anime_info_cache().await;
    
    list
}

#[tauri::command]
async fn add_to_list(id: i32, list: String) {

    let mut user_anime = UserAnimeInfo::default();
    user_anime.media_id = id;
    user_anime.status = list;

    update_user_entry(user_anime).await;
    file_name_recognition::parse_file_names(Some(id)).await;
}


#[tauri::command]
async fn remove_anime(id: i32, media_id: i32) -> bool {

    let removed = api_calls::anilist_remove_entry(id, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    if removed == true {

        let status = GLOBAL_USER_ANIME_DATA.lock().await.get(&media_id).unwrap().status.clone();
        let list = if status == "REPEATING" {
            String::from("CURRENT")
        } else {
            status
        };

        GLOBAL_USER_ANIME_LISTS.lock().await.entry(list).and_modify(|list| { list.retain(|id| *id != media_id)});
        GLOBAL_USER_ANIME_DATA.lock().await.remove(&media_id);
    }
    file_operations::write_file_user_info().await;
    removed
}

#[tauri::command]
async fn set_highlight(color: String) {
    GLOBAL_USER_SETTINGS.lock().await.highlight_color = color;
}

#[tauri::command]
async fn get_highlight() -> String {
    GLOBAL_USER_SETTINGS.lock().await.highlight_color.clone()
}


#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
  // Close splashscreen
  if let Some(splashscreen) = window.get_window("splashscreen") {
    splashscreen.close().unwrap();
  }
  // Show main window
  window.get_window("main").unwrap().show().unwrap();
}


#[tauri::command]
async fn get_torrents(id: i32) -> Vec<RssEntry> {

    rss_parser::get_rss(id).await
}


#[tauri::command]
async fn recommend_anime(mode: String, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) -> Vec<AnimeInfo> {

    let ids = recommendation::recommendations(mode, genre_filter, year_min_filter, year_max_filter, format_filter).await;
    let mut anime: Vec<AnimeInfo> = Vec::new();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    for id in ids {
        if anime_data.contains_key(&id) == false {
            println!("missing: {}", id);
            continue;
        }
        anime.push(anime_data.get(&id).unwrap().clone());
    }
    anime
}


#[tauri::command]
async fn open_url(url: String) {
    match open::that(url) {
        Err(why) => panic!("{}",why),
        Ok(e) => {e},
    }
}

#[tauri::command]
async fn get_list_ids(list: String) -> Option<Vec<i32>> {

    let user_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    if user_lists.contains_key(&list) {
        
        let mut ids = user_lists.get(&list).unwrap().clone();
        if GLOBAL_USER_SETTINGS.lock().await.show_adult == false {
            let anime_data = GLOBAL_ANIME_DATA.lock().await;
            ids.retain(|id| { anime_data.get(id).unwrap().is_adult == false });
        }
        return Some(ids);
    }

    None
}


// runs tests on recognizing filenames and returns the results
// returns nothing if the program is not compiled as debug
#[tauri::command]
async fn run_filename_tests() -> Vec<FilenameTest> {

    if constants::DEBUG {
        return file_name_recognition_tests::filename_tests().await;
    }
    Vec::new()
}


// returns true if the program was compiled as debug
#[tauri::command]
async fn get_debug() -> bool {

    return constants::DEBUG;
}


// clears all user data from memory and disk
#[tauri::command]
async fn delete_data() -> bool {

    GLOBAL_TOKEN.lock().await.clear();
    GLOBAL_ANIME_DATA.lock().await.clear();
    GLOBAL_USER_ANIME_DATA.lock().await.clear();
    GLOBAL_USER_ANIME_LISTS.lock().await.clear();
    GLOBAL_USER_SETTINGS.lock().await.clear();
    GLOBAL_ANIME_PATH.lock().await.clear();
    GLOBAL_REFRESH_UI.lock().await.clear();
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.clear();

    file_operations::delete_data()
}


// returns if startup tasks are finished.  Data will be missing if startup is not completed
#[tauri::command]
async fn startup_finished() -> bool {
    return *GLOBAL_STARTUP_FINISHED.lock().await;
}



fn main() {
    tauri::Builder::default()
    .setup(|app| {
        let splashscreen_window = app.get_window("splashscreen").unwrap();
        let main_window = app.get_window("main").unwrap();
        
        tauri::async_runtime::spawn(async move {

            load_user_settings().await;

            if GLOBAL_USER_SETTINGS.lock().await.first_time_setup == false {
                on_startup().await;
            }

            splashscreen_window.close().unwrap();
            main_window.show().unwrap();
        });
        Ok(())
      })
        .invoke_handler(tauri::generate_handler![get_anime_info_query,set_highlight,get_highlight,anilist_oauth_token,write_token_data,set_user_settings,
            get_user_settings,get_list_user_info,get_anime_info,get_user_info,update_user_entry,get_list,on_startup,load_user_settings,scan_anime_folder,
            play_next_episode,anime_update_delay,refresh_ui,increment_decrement_episode,on_shutdown,episodes_exist,browse,
            add_to_list,remove_anime,episodes_exist_single,get_delay_info,get_list_paged,set_current_tab,close_splashscreen,get_torrents,recommend_anime,
            open_url,get_list_ids,run_filename_tests,get_debug,delete_data,scan_files,startup_finished])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


// gets user data for every anime and list in a users account and stores it in global data
async fn get_user_data() {

    let username = GLOBAL_USER_SETTINGS.lock().await.username.clone();
    if username.is_empty() {
        return;
    }

    let response = api_calls::anilist_list_query_call(username, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();

    for item in json["data"]["MediaListCollection"]["lists"].as_array().unwrap() {

        let mut name: String = serde_json::from_value(item["status"].clone()).unwrap();
        if name == "REPEATING" {
            name = String::from("CURRENT");
        }

        let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
        
        if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&name) == false {
            GLOBAL_USER_ANIME_LISTS.lock().await.insert(name.clone(), Vec::new());
        }

        GLOBAL_USER_ANIME_LISTS.lock().await.entry(name.clone()).and_modify(|list| {
            
            for item2 in item["entries"].as_array().unwrap() {

                let entry: UserAnimeInfo = serde_json::from_value(item2.clone()).unwrap();

                if list.contains(&entry.media_id) == false {

                    list.push(entry.media_id.clone());
                }
                user_data.insert(entry.media_id, entry);
            }
        });
    }
}