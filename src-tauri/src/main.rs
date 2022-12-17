/*#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
*/


pub mod secrets;
pub mod api_calls;
pub mod file_operations;
pub mod file_name_recognition;

#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use regex::Regex;
use serde::{Serialize, Deserialize};
use tauri::async_runtime::Mutex;
use window_titles::{Connection, ConnectionTrait};
use std::{collections::HashMap, path::Path, time::{Duration, Instant}, thread};
use open;

use api_calls::{TokenData, UserSettings, AnilistDate};

use crate::{api_calls::{AnimeInfo, UserAnimeInfo}, file_name_recognition::AnimePath};

lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_DATA: Mutex<HashMap<i32, UserAnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_LISTS: Mutex<HashMap<String, Vec<i32>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_SETTINGS: Mutex<UserSettings> = Mutex::new(UserSettings::new());
    static ref GLOBAL_ANIME_PATH: Mutex<HashMap<i32,HashMap<i32,AnimePath>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_REFRESH_UI: Mutex<bool> = Mutex::new(false);
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
            if user_settings.folders[i] != settings.folders[i] {
                scan = true;
            }
        }
    }

    let score_format = user_settings.score_format.clone();
    let old_username = user_settings.username.clone();
    *user_settings = settings;
    user_settings.score_format = score_format;

    if old_username != user_settings.username {
        GLOBAL_USER_ANIME_LISTS.lock().await.clear();
        GLOBAL_USER_ANIME_DATA.lock().await.clear();

        user_settings.score_format = api_calls::get_user_score_format(user_settings.username.clone()).await;
    }

    if scan {
        file_name_recognition::parse_file_names(&user_settings.folders, None).await;
    }

    drop(user_settings);
    file_operations::write_file_user_settings().await;
}

// retrieves user's settings from a file
#[tauri::command]
async fn get_user_settings() -> UserSettings {
    GLOBAL_USER_SETTINGS.lock().await.clone()
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
async fn get_list_paged(list_name: String, sort: String, ascending: bool, page: usize) -> Vec<(AnimeInfo, UserAnimeInfo)> {

    let anime_per_page: usize = 50;

    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        if error_message.is_some() {
            //return (Vec::new(), error_message)
            println!("{}", error_message.unwrap());
        }
        file_operations::write_file_anime_info_cache().await;
        file_operations::write_file_user_info().await;
    }

    let mut anime_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = anime_lists.get_mut(&list_name).unwrap();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let user_data = GLOBAL_USER_ANIME_DATA.lock().await;

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
    }

    let start = page * anime_per_page;
    let finish = 
    if (page + 1) * anime_per_page > list.len() {
        list.len()
    } else {
        (page + 1) * anime_per_page
    };

    let mut list_info: Vec<(AnimeInfo, UserAnimeInfo)> = Vec::new();
    for i in start..finish {
        list_info.push((anime_data.get(list.get(i).unwrap()).unwrap().clone(), user_data.get(list.get(i).unwrap()).unwrap().clone()));
    }

    list_info
}

// get all user data for all anime in a specific list
#[tauri::command]
async fn get_list_user_info(list_name: String) -> Vec<UserAnimeInfo> {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    let mut list: Vec<UserAnimeInfo> = Vec::new();
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    for item in GLOBAL_USER_ANIME_LISTS.lock().await.entry(list_name).or_insert(Vec::new()) {
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

        let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * 60) as f64;
        
        let anime = found_anime.iter().next().unwrap();
        
        return (anime.1.timer.elapsed().as_secs_f64() / delay, anime.1.episode, anime.1.title.clone(), (delay as i64) - (anime.1.timer.elapsed().as_secs() as i64));
    }

    return (0.0, 0, String::new(), 0)
}

// get data for a specific anime
#[tauri::command]
async fn get_anime_info(id: i32) -> AnimeInfo {

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }

    let anime_data = GLOBAL_ANIME_DATA.lock().await.get(&id).unwrap().clone();
    anime_data
}

// updates a entry on anilist with new information
#[tauri::command]
async fn update_user_entry(anime: UserAnimeInfo) {

    let old_status: String = if GLOBAL_USER_ANIME_DATA.lock().await.contains_key(&anime.media_id) {
        GLOBAL_USER_ANIME_DATA.lock().await.entry(anime.media_id).or_default().status.clone()
    } else {
        String::new()
    };
    
    let new_status = if anime.status == "REPEATING" {
        String::from("CURRENT")
    } else {
        anime.status.clone()
    };

    if old_status != new_status {
        
        if old_status.is_empty() == false {

            GLOBAL_USER_ANIME_LISTS.lock().await.entry(old_status.clone()).and_modify(|data|{ 
                data.remove(data.iter().position(|&v| v == anime.media_id).unwrap());
            });
        }
        
        let mut lists = GLOBAL_USER_ANIME_LISTS.lock().await;
        if lists.contains_key(&new_status) {
            
            let list = lists.entry(new_status).or_default();
            if list.len() > 0 {

                list.push(anime.media_id);
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
    let score_format = api_calls::get_user_score_format(GLOBAL_USER_SETTINGS.lock().await.username.clone()).await;
    GLOBAL_USER_SETTINGS.lock().await.score_format = score_format;
    scan_anime_folder().await;
}

// loads data from files and looks for episodes on disk
#[tauri::command]
async fn load_user_settings() {

    file_operations::read_file_user_settings().await;
}

// go ahead with any updates that haven't been completed yet before closing
#[tauri::command]
async fn on_shutdown() {

    check_delayed_updates(false).await;
}

// check if enough time has passed before updating the episode of a anime
// this delay is to prevent spamming or locking when the user increases or decreases the episode count multiple times
async fn check_delayed_updates(wait: bool) {
    
    let delay = 15;
    let delayed_update = GLOBAL_UPDATE_ANIME_DELAYED.lock().await.clone();
    for entry in delayed_update.iter() {
        
        if entry.1.elapsed() >= Duration::from_secs(delay) || wait == false {

            api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), GLOBAL_USER_ANIME_DATA.lock().await.get(&entry.0).unwrap().clone()).await;
        }
    }
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.retain(|_, v| v.elapsed() < Duration::from_secs(delay));
}

// opens the file for the next episode in the default program
#[tauri::command]
async fn play_next_episode(id: i32) {
    
    let next_episode = GLOBAL_USER_ANIME_DATA.lock().await.get(&id).unwrap().progress + 1;
    let paths = GLOBAL_ANIME_PATH.lock().await;

    if paths.contains_key(&id) {
        let media = paths.get(&id).unwrap();
        if media.contains_key(&next_episode) {
            
            let next_episode_path = Path::new(&media.get(&next_episode).unwrap().path);
            match open::that(next_episode_path) {
                Err(why) => panic!("{}",why),
                Ok(e) => {e},
            }
            println!("opened {}", next_episode_path.to_str().unwrap());
        } else {
            println!("no episode key {}", next_episode);
        }
    } else {
        println!("no media key {}", id);
    }

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
        
        let max_episodes = GLOBAL_ANIME_DATA.lock().await.get(&anime_id).unwrap().episodes.unwrap();
        if (change == -1 && progress == 0) || (change == 1 && progress == max_episodes) {
            return;
        }

        let anime = user_data.get_mut(&anime_id).unwrap();
        change_episode(anime, anime.progress + change, GLOBAL_ANIME_DATA.lock().await.get(&anime.media_id).unwrap().episodes.unwrap()).await;

        let mut delayed_update = GLOBAL_UPDATE_ANIME_DELAYED.lock().await;
        if delayed_update.contains_key(&anime_id) {
            delayed_update.entry(anime_id).and_modify(|entry| {
                *entry = Instant::now();
            });
        } else {
            delayed_update.insert(anime_id, Instant::now());
        }
    }
}

// scan folders for episodes of anime
#[tauri::command]
async fn scan_anime_folder() {
    file_name_recognition::parse_file_names(&GLOBAL_USER_SETTINGS.lock().await.folders, None).await;
}

#[derive(Debug, Clone)]
struct WatchingTracking {
    timer: std::time::Instant,
    monitoring: bool,
    episode: i32,
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

// loops through timed tasks like recognizing playing anime and delayed updates
#[tauri::command]
async fn anime_update_delay_loop() {

    let mut hour_timer = Instant::now();
    let one_hour = Duration::from_secs(60 * 60);

    loop { 

        anime_update_delay().await;

        check_delayed_updates(true).await;

        if hour_timer.elapsed() > one_hour {
            file_name_recognition::parse_file_names(&GLOBAL_USER_SETTINGS.lock().await.folders, None).await;
            hour_timer = Instant::now();
        }

        thread::sleep(Duration::from_secs(5)); 
    }
}

// scans for and identifies windows playing anime and sets up a delayed update
#[tauri::command]
async fn anime_update_delay() {

    //let mut filename: String = String::new();
    //let mut ignore_filenames: Vec<String> = Vec::new();
    let regex = Regex::new(r"\.mkv|\.avi|\.mp4").unwrap();
    let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * 60)  as u64;
    let language = GLOBAL_USER_SETTINGS.lock().await.title_language.clone();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut watching_data = WATCHING_TRACKING.lock().await;
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    watching_data.iter_mut().for_each(|entry| {
        entry.1.monitoring = false;
    });

    let mut titles: Vec<String> = get_titles();
    titles.retain(|v| regex.is_match(v));

    for title in titles {

        let mut title_edit: String = regex.replace(&title, "").to_string();
        title_edit = file_name_recognition::remove_brackets(&title_edit);
        let (episode_str, episode) = file_name_recognition::identify_number(&title_edit);
        title_edit = title_edit.replace(episode_str.as_str(), "");
        title_edit = file_name_recognition::irrelevant_information_removal(title_edit);

        let (media_id, media_score) = file_name_recognition::identify_media_id(&title_edit, &anime_data, None);
        if media_score < 0.8 { continue; }
        //println!("{} {} {} {:.4}", title_edit, episode.1, media_id, media_score);
        if watching_data.contains_key(&media_id) {
            watching_data.entry(media_id).and_modify(|entry| {
                entry.monitoring = true;
            });
        } else if user_data.contains_key(&media_id) && user_data.get(&media_id).unwrap().progress + 1 == episode { // only add if it is in the users list and it is the next episode
            if language == "romaji" {
                watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode, title: anime_data.get(&media_id).unwrap().title.romaji.clone().unwrap()});
            } else if language == "english" {
                watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode, title: anime_data.get(&media_id).unwrap().title.english.clone().unwrap()});
            } else if language == "native" {
                watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode, title: anime_data.get(&media_id).unwrap().title.native.clone().unwrap()});
            }
        }
    }

    let mut update_entries: Vec<UserAnimeInfo> = Vec::new();
    for (media_id, tracking_info) in watching_data.iter_mut() {
        let seconds = tracking_info.timer.elapsed().as_secs();
        if seconds >= delay {
            tracking_info.monitoring = false;
            // update anime
            
            let anime = user_data.get_mut(&media_id).unwrap();
            change_episode(anime, tracking_info.episode, anime_data.get(media_id).unwrap().episodes.unwrap()).await;

            update_entries.push(user_data.get(media_id).unwrap().clone());
            *GLOBAL_REFRESH_UI.lock().await = true;
        }
    }

    watching_data.retain(|_, v| v.monitoring == true);

    // unlock mutexes before doing api calls which might take awhile
    drop(anime_data);
    drop(watching_data);
    drop(user_data);

    let access_token = GLOBAL_TOKEN.lock().await.access_token.clone();
    for anime in update_entries {

        api_calls::update_user_entry(access_token.clone(), anime).await;
    }
}

// change the episode for the user entry and trigger other changes depending on the episode
async fn change_episode(anime: &mut UserAnimeInfo, episode: i32, max_episodes: i32) {

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
    if episode > progress && episode != max_episodes && anime.status != "CURRENT"{

        change_list(anime, String::from("CURRENT")).await;
    }

    // add anime to completed if the last episode was watched and set complete date
    if episode == max_episodes {
        let now: DateTime<Local> = Local::now();
        anime.completed_at = Some(AnilistDate {
            year: Some(now.year()),
            month: Some(now.month() as i32),
            day: Some(now.day() as i32),
        });
        if anime.status != "COMPLETED" {

            change_list(anime, String::from("COMPLETED")).await;
        }
    }
}

async fn change_list(anime: &mut UserAnimeInfo, new_list: String) {

    let mut lists = GLOBAL_USER_ANIME_LISTS.lock().await;

    lists.entry(anime.status.clone()).and_modify(|list| {
        let index = list.iter().position(|v| *v == anime.media_id).unwrap();
        list.remove(index);
    });

    anime.status = new_list;

    lists.entry(anime.status.clone()).and_modify(|list| {
        list.push(anime.media_id);
    });
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RefreshUI {
    pub anime_list: bool,
    pub tracking_progress: bool,
}

// allows the ui to check if a anime has been updated to determine if the ui will be refreshed
#[tauri::command]
async fn get_refresh_ui() -> RefreshUI {

    thread::sleep(Duration::from_millis(1000));

    let mut refresh_ui = RefreshUI::default();

    let mut refresh = GLOBAL_REFRESH_UI.lock().await;
    refresh_ui.anime_list = refresh.clone();
    *refresh = false;

    refresh_ui.tracking_progress = WATCHING_TRACKING.lock().await.len() > 0;

    refresh_ui
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
async fn browse(year: String, season: String, genre: String, format: String, order: String) -> Vec<AnimeInfo> {

    let mut list: Vec<AnimeInfo> = Vec::new();

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut has_next_page = true;

    let mut page = 0;
    while has_next_page {
        
        page += 1;
        let response = api_calls::anilist_browse_call(page, year.clone(), season.clone(), genre.clone(), format.clone(), order.clone()).await;

        for anime in response["data"]["Page"]["media"].as_array().unwrap() {

            let id = anime["id"].as_i64().unwrap() as i32;

            let anime_entry: AnimeInfo = serde_json::from_value(anime.clone()).unwrap();
            list.push(anime_entry.clone());
            anime_data.insert(id, anime_entry);
        }
        
        if page >= 2 {
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
    file_name_recognition::parse_file_names(&GLOBAL_USER_SETTINGS.lock().await.folders, Some(id)).await;
}


#[tauri::command]
async fn remove_anime(id: i32, media_id: i32) -> bool {

    let removed = api_calls::anilist_remove_entry(id, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    if removed == true {

        let status = GLOBAL_USER_ANIME_DATA.lock().await.get(&media_id).unwrap().status.clone();

        GLOBAL_USER_ANIME_LISTS.lock().await.entry(status).and_modify(|list| {

            let position = list.iter().position(|v| *v == media_id).unwrap();
            list.remove(position);
        });

        GLOBAL_USER_ANIME_DATA.lock().await.remove(&media_id);
    }
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query,set_highlight,get_highlight,anilist_oauth_token,write_token_data,set_user_settings,
            get_user_settings,get_list_user_info,get_anime_info,get_user_info,update_user_entry,get_list,on_startup,load_user_settings,scan_anime_folder,
            play_next_episode,anime_update_delay,anime_update_delay_loop,get_refresh_ui,increment_decrement_episode,on_shutdown,episodes_exist,browse,
            add_to_list,remove_anime,episodes_exist_single,get_delay_info,get_list_paged])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn get_user_data() {

    let response = api_calls::anilist_list_query_call(GLOBAL_USER_SETTINGS.lock().await.username.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    
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
                //println!("{} {}", entry.id, entry.progress);
                list.push(entry.media_id.clone());
                user_data.insert(entry.media_id, entry);
            }
        });
    }
}