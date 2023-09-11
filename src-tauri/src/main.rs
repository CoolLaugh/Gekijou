#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod constants;
pub mod secrets;
pub mod api_calls;
pub mod file_operations;
pub mod rss_parser;
pub mod recommendation;
pub mod file_name_recognition_tests;
pub mod user_data;
pub mod anime_data;

#[macro_use]
extern crate lazy_static;

use anime_data::AnimeInfo;
use file_name_recognition_tests::FilenameTest;
use regex::Regex;
use rss_parser::RssEntry;
use serde::{Serialize, Deserialize};
use tauri::async_runtime::Mutex;
use tauri::Manager;
use user_data::{UserData, UserInfo, UserSettings};
use window_titles::{Connection, ConnectionTrait};
use std::{collections::{HashMap, HashSet}, path::Path, time::{Duration, Instant}, ops::{Range, Deref}};
use open;
use api_calls::MangaInfo;
use crate::anime_data::{AnimeData, AnimePath};



//stores details on which parts of the UI need to be refreshed
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RefreshUI {
    pub anime_list: bool,
    pub tracking_progress: bool,
    pub canvas: bool,
    pub no_internet: bool,
    pub scan_data: ScanData,
    pub errors: Vec<String>,
    pub loading_dialog: Option<String>,
}

impl RefreshUI {
    
    pub fn clear(&mut self) {
        self.anime_list = false;
        self.tracking_progress = false;
        self.canvas = false;
        self.no_internet = false;
        self.scan_data.clear();
        self.errors.clear();
        self.loading_dialog = None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ScanData {
    pub current_folder: i32,
    pub total_folders: i32,
    pub completed_chunks: i32,
    pub total_chunks: i32,
}

impl ScanData {
    
    pub fn clear(&mut self) {
        self.current_folder = 0;
        self.total_folders = 0;
        self.completed_chunks = 0;
        self.total_chunks = 0;
    }
}



lazy_static! {
    static ref GLOBAL_REFRESH_UI: Mutex<RefreshUI> = Mutex::new(RefreshUI::default());
    static ref GLOBAL_UPDATE_ANIME_DELAYED: Mutex<HashMap<i32, Instant>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_DATA: Mutex<UserData> = Mutex::new(UserData::new());
    static ref GLOBAL_ANIME_DATA: Mutex<AnimeData> = Mutex::new(AnimeData::new());
}



// takes a oauth code from the user and exchanges it for a oauth access token
#[tauri::command]
async fn anilist_oauth_token(code: String) -> (bool, String) {
    
    GLOBAL_USER_DATA.lock().await.anilist_oauth_token(code).await
}



// save token data to a file
#[tauri::command]
async fn write_token_data() {
    //file_operations::write_file_token_data().await;
}



// sets the user's settings taken from the settings ui
#[tauri::command]
async fn set_user_settings(settings: UserSettings) {

    let (scan, media_ids) = GLOBAL_USER_DATA.lock().await.set_user_settings(settings).await;

    if let Some(ids) = media_ids {
        // get anime data from anilist
        GLOBAL_REFRESH_UI.lock().await.loading_dialog = Some(format!("Downloading Anime Data"));
        let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
        anime_data.get_anime_list_data(ids).await;
        if anime_data.new_anime == true {
            GLOBAL_USER_DATA.lock().await.set_max_episodes(anime_data.get_anime_episodes());
            anime_data.new_anime = false;
        }
        GLOBAL_REFRESH_UI.lock().await.loading_dialog = Some(format!("Finished Downloading Anime Data"));
    }

    if scan {
        scan_anime_folder();
    }

    GLOBAL_REFRESH_UI.lock().await.loading_dialog = None;

    //file_operations::write_file_user_settings().await;
}



// retrieves user's settings from a file
#[tauri::command]
async fn get_user_settings() -> user_data::UserSettings {
    GLOBAL_USER_DATA.lock().await.get_user_settings()
}



// retrieves user's settings from a file
#[tauri::command]
async fn set_current_tab(current_tab: String) {
    GLOBAL_USER_DATA.lock().await.set_current_tab(current_tab);
}



// returns all information of anime on the users anime list
// information is paged, 50 entries are returned per page
// sorting and ascending are only used for page 0, other pages use the sorting order of the last time page 0 was called
#[tauri::command]
async fn get_list_paged(list_name: String, sort: String, ascending: bool, page: usize) -> Result<Vec<(anime_data::AnimeInfo, UserInfo)>, &'static str>{

    // // list won't exist if user doesn't exist
    // if GLOBAL_USER_SETTINGS.lock().await.username.is_empty() {
    //     return (Vec::new(), None);
    // }
    let mut user_data = GLOBAL_USER_DATA.lock().await;

    let list = match user_data.get_list(&list_name).await {
        Ok(list) => { list },
        Err(error) => { return Err(error); },
    };

    let user_list_data = match user_data.get_data(&list).await {
        Ok(list) => { list },
        Err(error) => { return Err(error); },
    };

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let anime_list_data = match anime_data.get_anime_list_data(list).await {
        Ok(list) => { 
            if anime_data.new_anime == true {
                user_data.set_max_episodes(anime_data.get_anime_episodes());
                anime_data.new_anime = false;
            }
            list },
        Err(error) => { return Err(error); },
    };
    drop(user_data);

    let mut combined_list: Vec<(anime_data::AnimeInfo, UserInfo)> = Vec::new();
    for i in 0..anime_list_data.len() {
        combined_list.push((anime_list_data[i].clone(), user_list_data[i].clone()));
    }

    anime_data.sort_list(&mut combined_list, Some(sort));
    if ascending == false {
        combined_list.reverse();
    }
    Ok(combined_list)
}



// get user info for a specific anime
#[tauri::command]
async fn get_user_info(id: i32) -> Option<UserInfo> {

    match GLOBAL_USER_DATA.lock().await.get_user_data(id) {
        Ok(result) => {
            return Some(result);
        },
        Err(error) => return None,
    }
}



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UpdateDelayInfo {
    pub percent: f64,
    pub episode: i32,
    pub title: String,
    pub time_remaining: i64,
}
// get info on when the currently watching episode will be updated
#[tauri::command]
async fn get_delay_info() -> UpdateDelayInfo {

    let found_anime = WATCHING_TRACKING.lock().await.clone();
    
    if found_anime.len() > 0 {

        let delay = (GLOBAL_USER_DATA.lock().await.get_update_delay() * constants::SECONDS_IN_MINUTES) as f64;
        
        let (_, anime) = found_anime.iter().next().unwrap();
        
        return UpdateDelayInfo {
            percent: anime.timer.elapsed().as_secs_f64() / delay, 
            episode: anime.episode + (anime.length - 1), 
            title: anime.title.clone(),
            time_remaining: (delay as i64) - (anime.timer.elapsed().as_secs() as i64)
        };
    }

    return UpdateDelayInfo::default()
}



// get data for a specific anime
#[tauri::command]
async fn get_anime_info(id: i32) -> Option<AnimeInfo> {

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    match anime_data.get_anime_data(id).await {
        Ok(result) => {
            if anime_data.new_anime == true {
                GLOBAL_USER_DATA.lock().await.set_max_episodes(anime_data.get_anime_episodes());
                anime_data.new_anime = false;
            }
            return Some(result);
        },
        Err(error) => return None,
    }
}



// get data for a specific anime
#[tauri::command]
async fn get_manga_info(id: i32) -> Option<MangaInfo> {

    match api_calls::anilist_get_manga_ln_info(id).await {
        Ok(result) => {
            return Some(result);
        },
        Err(_error) => { 
            GLOBAL_REFRESH_UI.lock().await.errors.push(String::from("Manga information is missing. No internet connection."));
            return None; 
        },
    }
}


// updates a entry on anilist with new information
#[tauri::command]
async fn update_user_entry(anime: UserInfo) {

    println!("main.rs update_user_entry");

    if anime.status == "BROWSE" {
        println!("{:?}", anime);
    }

    GLOBAL_USER_DATA.lock().await.set_user_data(anime, true).await;
}



// changes the custom title of anime with id of anime_id to title
#[tauri::command]
async fn set_custom_filename(anime_id: i32, title: String) {
    GLOBAL_ANIME_DATA.lock().await.set_custom_filename(anime_id, title).await;
}



// returns the custom title set by the user previously, if the custom title or anime doesn't exist a empty string is returned
#[tauri::command]
async fn get_custom_filename(anime_id: i32) -> String {
    match GLOBAL_ANIME_DATA.lock().await.get_custom_filename(anime_id) {
        Some(title) => return title,
        None => return String::new(),
    }
}



// loads data from files and looks for episodes on disk
#[tauri::command]
async fn on_startup() {

    let mut user_data = GLOBAL_USER_DATA.lock().await;
    user_data.read_files().await;
    user_data.pull_updates().await;

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    anime_data.read_files().await;
    let episodes = anime_data.get_anime_episodes();
    user_data.set_max_episodes(episodes);

    for id in anime_data.nonexistent_ids.clone() {
        match user_data.remove_anime(id).await {
            Ok(_) => {},
            Err(error) => println!("{}", error)
        }
    }

    *GLOBAL_STARTUP_FINISHED.lock().await = true;
}




lazy_static! {
    static ref GLOBAL_SETTINGS_LOADED: Mutex<bool> = Mutex::new(false);
    static ref GLOBAL_STARTUP_FINISHED: Mutex<bool> = Mutex::new(false);
}

// opens the file for the next episode in the default program
#[tauri::command]
async fn play_next_episode(id: i32) -> Result<(), &'static str> {
    
    let user_data = GLOBAL_USER_DATA.lock().await;
    match user_data.get_user_data(id) {
        Ok(user_info) => {

            let next_episode = user_info.progress + 1;

            if play_episode(id, next_episode).await == false {
                // if episode location is unknown, search for new episodes and try again
                let folders = user_data.get_user_settings().folders;
                // don't interrupt another scan
                if GLOBAL_REFRESH_UI.lock().await.scan_data.total_folders > 0 {
                    return Ok(());
                }
                GLOBAL_REFRESH_UI.lock().await.loading_dialog = Some(String::from("Searching For Episode"));
                if GLOBAL_ANIME_DATA.lock().await.scan_folders(folders, false, Some(id)).await {
                    GLOBAL_REFRESH_UI.lock().await.canvas = true;
                }
                GLOBAL_REFRESH_UI.lock().await.loading_dialog = None;
                play_episode(id, next_episode).await;
            }
            Ok(())
        },
        Err(error) => return Err(error),
    }
}



// play the episode from the anime id
// returns true if the episode was played
async fn play_episode(anime_id: i32, episode: i32) -> bool {
    println!("play {} episode {}", anime_id, episode);
    GLOBAL_ANIME_DATA.lock().await.play_episode(anime_id, episode).await
}



// changes the progress for a anime by +-1
// anilist api call is delayed to prevent spam/locking
#[tauri::command]
async fn increment_decrement_episode(anime_id: i32, change: i32) {

    GLOBAL_USER_DATA.lock().await.increment_episode(anime_id, change).await;
}



// scan folders for episodes of anime
#[tauri::command]
async fn scan_anime_folder() -> bool {
    if GLOBAL_REFRESH_UI.lock().await.scan_data.total_folders > 0 {
        return false;
    }
    let folders = GLOBAL_USER_DATA.lock().await.get_user_settings().folders;
    // make a copy because scan folders will take a long time
    let mut anime_data: AnimeData = GLOBAL_ANIME_DATA.lock().await.clone();
    anime_data.scan_new_ids(folders.clone()).await;
    let file_found = anime_data.scan_folders(folders, true, None).await;
    *GLOBAL_ANIME_DATA.lock().await = anime_data;
    file_found
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
    connection.unwrap().window_titles().unwrap()
}

#[tauri::command]
async fn anime_update_delay() {

    let video_files = Regex::new(r"\.mkv|\.avi|\.mp4").unwrap();

    let vlc_remove = Regex::new(r" - VLC media player").unwrap();
    let gom_remove = Regex::new(r" - GOM Player").unwrap();
    let zoom_remove = Regex::new(r" - Zoom Player MAX").unwrap();
    let mpv_remove = Regex::new(r" - mpv").unwrap();
    let pot_remove = Regex::new(r" - PotPlayer").unwrap();

    let settings = GLOBAL_USER_DATA.lock().await.get_user_settings();

    // get window titles and keep the ones with video files
    let mut titles: Vec<String> = get_titles(); // for some reason mutex locking has to happen before this function
    titles.retain(|title| video_files.is_match(title));

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut user_data = GLOBAL_USER_DATA.lock().await;
    let mut watching_data = WATCHING_TRACKING.lock().await;

    // reset monitoring
    watching_data.iter_mut().for_each(|entry| entry.1.monitoring = false);

    for title in titles {

        // remove video player suffixes 
        let mut title_edit: String = vlc_remove.replace(&title, "").to_string();
        title_edit = gom_remove.replace(&title_edit, "").to_string();
        title_edit = zoom_remove.replace(&title_edit, "").to_string();
        title_edit = mpv_remove.replace(&title_edit, "").to_string();
        title_edit = pot_remove.replace(&title_edit, "").to_string();

        if let Some(identify_info) = anime_data.identify_anime(title_edit, None) {

            if let Ok(user_entry) = user_data.get_user_data(identify_info.media_id) {

                let next_episode: bool = identify_info.episode > user_entry.progress && identify_info.episode <= user_entry.progress + identify_info.episode_length;

                // if the file is being monitored and the episode is the next episode
                if let Some(entry) = watching_data.get_mut(&identify_info.media_id) {
                    if next_episode && entry.episode == identify_info.episode {
                        entry.monitoring = true;
                    }
                // only add if it is in the users list, it is the next episode, and the episode is within range
                } else if next_episode && identify_info.episode > 0 {
        
                    watching_data.insert(identify_info.media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: identify_info.episode, length: identify_info.episode_length, title: identify_info.media_title});
                }
            }
        }
    }
    
    let delay = (settings.update_delay * constants::SECONDS_IN_MINUTES)  as u64;
    // check if media has been playing for long enough to update
    for (media_id, tracking_info) in watching_data.iter_mut() {
        let seconds = tracking_info.timer.elapsed().as_secs();
        if seconds >= delay {
            // user progress will be updated to this episode so we no longer want to monitor it
            tracking_info.monitoring = false;
            
            // update anime
            user_data.increment_episode(*media_id, tracking_info.length).await;

            // update ui with episode progress
            GLOBAL_REFRESH_UI.lock().await.canvas = true;
        }
    }

    // remove episodes that are no longer being played or have been played long enough
    watching_data.retain(|_, v| v.monitoring == true);

}



// allows the ui to check if a anime has been updated to determine if the ui will be refreshed
#[tauri::command]
async fn refresh_ui() -> RefreshUI {
    let length = WATCHING_TRACKING.lock().await.len();
    let mut refresh = GLOBAL_REFRESH_UI.lock().await;
    let mut refresh_ui = refresh.clone();
    refresh_ui.tracking_progress = length > 0;
    refresh.anime_list = false;
    refresh.canvas = false;

    refresh_ui
}



#[tauri::command]
async fn clear_errors() {
    GLOBAL_REFRESH_UI.lock().await.errors.clear();
}


lazy_static! {
    static ref SCAN_TIMER: Mutex<Instant> = Mutex::new(Instant::now());
    static ref NO_INTERNET_TIMER: Mutex<Instant> = Mutex::new(Instant::now());
    static ref STARTUP_SCAN: Mutex<bool> = Mutex::new(false);
}
// performs periodic tasks like checking for anime in media players, delayed updates that must be sent, scanning folders for files
// it's expected that this function will be called periodically from the UI, it won't loop on its own
#[tauri::command]
async fn background_tasks() {
    
    // check for anime in media players
    anime_update_delay().await;
    // update anilist with offline updates
    //check_queued_updates().await;

    // do a full scan for anime recently added to program
    //file_name_recognition::parse_file_names(media_id);

    // scan files for new episodes of anime every hour and a short time after startup
    let mut on_startup_scan_completed = STARTUP_SCAN.lock().await;
    let mut timer = SCAN_TIMER.lock().await;
    if timer.elapsed() > Duration::from_secs(constants::ONE_HOUR) || 
        (timer.elapsed() >= Duration::from_secs(constants::STARTUP_SCAN_DELAY) && *on_startup_scan_completed == false) {

        if scan_anime_folder().await {

            GLOBAL_REFRESH_UI.lock().await.canvas = true;
        }
        *on_startup_scan_completed = true;
        *timer = Instant::now();
    }
}



// returns a list of what episodes of what anime exist on disk
#[tauri::command]
async fn episodes_exist() -> HashMap<i32, Vec<i32>> {
    GLOBAL_ANIME_DATA.lock().await.get_existing_files_all_anime()
}



// returns a list of all episodes on disk for a anime
#[tauri::command]
async fn episodes_exist_single(id: i32) -> Vec<i32> {
    
    GLOBAL_ANIME_DATA.lock().await.get_existing_files(id)
}



// returns a list of anime based on filters and sorting order
// anime in user's list does not matter and user login is not used
#[tauri::command]
async fn browse(year: String, season: String, genre: String, format: String, search: String, order: String, page: usize) -> Result<(Vec<AnimeInfo>, bool), &'static str> {

    let mut list: Vec<AnimeInfo> = Vec::new();
    let mut has_next_page = true;

    
    match api_calls::anilist_browse_call(page, year.clone(), season.clone(), genre.clone(), format.clone(), search.clone(), order.clone()).await {
        Ok(response) =>  {

            // add anime to return list and global data for further uses
            for anime in response["data"]["Page"]["media"].as_array().unwrap() {
    
                let anime_entry: AnimeInfo = serde_json::from_value(anime.clone()).unwrap();
                list.push(anime_entry);
            }
    
            // check if a next page exists
            if let Some(response_next_page) = response["data"]["Page"]["pageInfo"]["hasNextPage"].as_bool() {
                has_next_page = response_next_page;
            } else {
                has_next_page = false;
            }
        },
        Err(error) => { 
            GLOBAL_REFRESH_UI.lock().await.no_internet = true;
            GLOBAL_REFRESH_UI.lock().await.errors.push(String::from("Cannot browse anilist. No internet connection."));
            return Err(error);
        },
    }

    
    Ok((list,has_next_page))
}



// add anime to the users list (not for moving anime)
#[tauri::command]
async fn add_to_list(id: i32, list: String) {

    let mut user_anime = UserInfo::default();
    user_anime.media_id = id;
    user_anime.status = list;

    update_user_entry(user_anime).await;

    GLOBAL_ANIME_DATA.lock().await.add_id_for_scanning(id);
}



// removes anime from the users list
#[tauri::command]
async fn remove_anime(media_id: i32) -> Result<bool, &'static str> {

    GLOBAL_USER_DATA.lock().await.remove_anime(media_id).await
}



// sets the highlight color
#[tauri::command]
async fn set_highlight(color: String) {
    GLOBAL_USER_DATA.lock().await.set_highlight(color);
}



// returns highlight color from user settings
#[tauri::command]
async fn get_highlight() -> String {
    GLOBAL_USER_DATA.lock().await.get_highlight()
}



// returns a list of rss entries from nyaa.si for a given anime
// id is the anilist id of the anime being searched for
#[tauri::command]
async fn get_torrents(id: i32) -> Vec<RssEntry> {

    rss_parser::get_rss(id).await
}



// generate a list of anime based on user recommendations
// mode changes what anime is used for recommendations, user recommendations will use any anime and other modes will limit anime to types of relations to completed anime
// filters will remove anime that do not match the filter
#[tauri::command]
async fn recommend_anime(mode: String, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) -> Vec<AnimeInfo> {

    let mut user_data = GLOBAL_USER_DATA.lock().await;
    let completed_scores = user_data.get_scores_from_list(String::from("COMPLETED"));
    let user_anime = user_data.all_ids();
    let score_format = user_data.get_user_settings().score_format;
    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let ids = anime_data.recommendations(completed_scores, user_anime, score_format, mode, genre_filter, year_min_filter, year_max_filter, format_filter).await;
    let anime = anime_data.get_anime_list_data(ids).await;

    if anime_data.new_anime == true {
        user_data.set_max_episodes(anime_data.get_anime_episodes());
        anime_data.new_anime = false;
    }
    
    match anime {
        Ok(anime_list) => {
            return anime_list;
        },
        Err(error) => { 
            println!("{}", error);
            return Vec::new();
        },
    }
}



// open a url using the default browser
#[tauri::command]
async fn open_url(url: String) {
    match open::that(url) {
        Err(why) => panic!("{}",why),
        Ok(e) => {e},
    }
}



// returns all ids in the users list
// None is returned if list does not exist
#[tauri::command]
async fn get_list_ids(list: String) -> Option<Vec<i32>> {

    match GLOBAL_USER_DATA.lock().await.get_list(&list).await {
        Ok(result) => {
            return Some(result);
        },
        Err(error) => return None,
    }
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

    GLOBAL_USER_DATA.lock().await.clear();
    GLOBAL_ANIME_DATA.lock().await.clear();

    GLOBAL_REFRESH_UI.lock().await.clear();
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.clear();
    WATCHING_TRACKING.lock().await.clear();

    file_operations::delete_data()
}



// returns if startup tasks are finished.  Data will be missing if startup is not completed
#[tauri::command]
async fn startup_finished() -> bool {
    return *GLOBAL_STARTUP_FINISHED.lock().await;
}



#[tauri::command]
async fn manual_scan() {
    if GLOBAL_REFRESH_UI.lock().await.scan_data.total_folders > 0 {
        return;
    }
    let folders = GLOBAL_USER_DATA.lock().await.get_user_settings().folders;
    // make a copy because scan folders will take a long time
    let mut anime_data: AnimeData = GLOBAL_ANIME_DATA.lock().await.clone();
    anime_data.scan_folders(folders, false, None).await;
    *GLOBAL_ANIME_DATA.lock().await = anime_data;
}



// initialize and run Gekijou
fn main() {
    tauri::Builder::default()
    .setup(|app| {
        let splashscreen_window = app.get_window("splashscreen").unwrap();
        let main_window = app.get_window("main").unwrap();

        tauri::async_runtime::spawn(async move {

            on_startup().await;

            splashscreen_window.close().unwrap();
            main_window.show().unwrap();
        });
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![manual_scan,set_highlight,get_highlight,anilist_oauth_token,write_token_data,set_user_settings,
        get_user_settings,get_anime_info,get_manga_info,get_user_info,update_user_entry,on_startup,scan_anime_folder,
        play_next_episode,anime_update_delay,refresh_ui,clear_errors,increment_decrement_episode,episodes_exist,browse,
        add_to_list,remove_anime,episodes_exist_single,get_delay_info,get_list_paged,set_current_tab,get_torrents,recommend_anime,
        open_url,get_list_ids,run_filename_tests,get_debug,delete_data,background_tasks,startup_finished,get_custom_filename,set_custom_filename])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}