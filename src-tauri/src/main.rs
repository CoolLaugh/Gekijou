#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod secrets;
pub mod api_calls;
pub mod file_operations;
pub mod file_name_recognition;

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use tauri::{async_runtime::Mutex, Event, Manager};
use window_titles::{Connection, ConnectionTrait};
use std::{collections::HashMap, path::Path, time::{Duration, Instant}, thread, task::Poll};

use api_calls::{TokenData, UserSettings};

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

#[tauri::command]
async fn anilist_oauth_token(code: String) -> (bool, String) {
    
    let token = api_calls::anilist_get_access_token(code).await;

    if token.access_token.len() == 0 {
        return (false, token.token_type);
    }
    else {
        *GLOBAL_TOKEN.lock().await = token;
    }

    write_token_data().await;
    
    (true, String::new())
}

#[tauri::command]
async fn read_token_data() {
    
    if file_operations::token_data_file_exists() == true {
        *GLOBAL_TOKEN.lock().await = file_operations::read_file_token_data();
    }

}

#[tauri::command]
async fn write_token_data() {
    file_operations::write_file_token_data(&*GLOBAL_TOKEN.lock().await);
}

#[tauri::command]
async fn get_anime_info_query(id: i32) -> api_calls::AnimeInfo {
    
    let response = api_calls::anilist_api_call(id).await;    
    print!("{}", response.id);
    response
}

#[tauri::command]
async fn set_user_settings(username: String, title_language: String, show_spoilers: bool, show_adult: bool, folders: Vec<String>) {

    let mut user_settings = GLOBAL_USER_SETTINGS.lock().await;

    user_settings.username = username;
    user_settings.title_language = title_language;
    user_settings.show_spoilers = show_spoilers;
    user_settings.show_adult = show_adult;
    user_settings.folders = folders;

    file_operations::write_file_user_settings(&*user_settings);
}

#[tauri::command]
async fn get_user_settings() -> UserSettings {

    let length = GLOBAL_USER_SETTINGS.lock().await.username.len();

    if length == 0 {
        *GLOBAL_USER_SETTINGS.lock().await = file_operations::read_file_user_settings();
    }

   GLOBAL_USER_SETTINGS.lock().await.clone()
}

#[tauri::command]
async fn get_list(list_name: String) -> Vec<AnimeInfo> {

    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        
        api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        file_operations::write_file_anime_info_cache().await;
    }
    
    let anime_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = anime_lists.get(&list_name).unwrap();

    let anime_data = GLOBAL_ANIME_DATA.lock().await;

    let mut list_info: Vec<AnimeInfo> = Vec::new();
    for id in list {
        list_info.push(anime_data.get(id).unwrap().clone());
    }

    list_info
}

#[tauri::command]
async fn get_watching_list(list_name: String) -> Vec<AnimeInfo> {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }


    let mut missing_anime: Vec<i32> = Vec::new();

    {
        let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;
        for item in GLOBAL_USER_ANIME_LISTS.lock().await.entry(list_name.clone()).or_insert(Vec::new()) {
            if anime_list.contains_key(&item) == false || anime_list[&item].cover_image.large.is_empty() {
                missing_anime.push(item.clone());
            }
        }
    }

    print!("\nmissing anime: {}", missing_anime.len());
    if missing_anime.len() > 0 {
            
        api_calls::anilist_get_anime_info_split(missing_anime).await;
        file_operations::write_file_anime_info_cache().await;
    }

    let mut return_data: Vec<AnimeInfo> = Vec::new();
    {
        let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;
        for item in GLOBAL_USER_ANIME_LISTS.lock().await.entry(list_name.clone()).or_insert(Vec::new()) {
            //print!("\n{} ", item.media_id);
            let entry = anime_list.entry(item.clone()).or_insert(AnimeInfo::default()).clone();
            //print!("{}", entry.title.english.unwrap());
            return_data.push(entry);
        }
    }

    print!("\n{}\n", return_data.len());
    return_data
}

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

#[tauri::command]
async fn get_user_info(id: i32) -> UserAnimeInfo {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    GLOBAL_USER_ANIME_DATA.lock().await.entry(id).or_insert(UserAnimeInfo::new()).clone()
}

#[tauri::command]
async fn get_anime_info(id: i32) -> AnimeInfo {

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }

    let anime_data = GLOBAL_ANIME_DATA.lock().await.get(&id).unwrap().clone();
    anime_data
}

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
    } else {
        anime_data.insert(media_id, new_info);
    }


}

#[tauri::command]
async fn on_startup() {

    *GLOBAL_TOKEN.lock().await = file_operations::read_file_token_data();
    *GLOBAL_USER_SETTINGS.lock().await =  file_operations::read_file_user_settings();
    file_operations::read_file_anime_info_cache().await;
    scan_anime_folder().await;
}

#[tauri::command]
async fn on_shutdown() {

    check_delayed_updates(false).await;
}

async fn check_delayed_updates(wait: bool) {
    
    let delay = 15;
    for entry in GLOBAL_UPDATE_ANIME_DELAYED.lock().await.iter() {
            
        if entry.1.elapsed() >= Duration::from_secs(delay) || wait == false {

            api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), GLOBAL_USER_ANIME_DATA.lock().await.get(&entry.0).unwrap().clone()).await;
        }
    }
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.retain(|_, v| v.elapsed() < Duration::from_secs(delay));
}

#[tauri::command]
async fn play_next_episode(id: i32) {
    println!("entered function");
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
        user_data.entry(anime_id).and_modify(|data| {
            println!("{}",data.progress);
            data.progress += change;
            println!("{}",data.progress);
        });

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

#[tauri::command]
async fn scan_anime_folder() {
    file_name_recognition::parse_file_names(&GLOBAL_USER_SETTINGS.lock().await.folders).await;
}

#[derive(Debug, Clone)]
struct WatchingTracking {
    timer: std::time::Instant,
    monitoring: bool,
    episode: i32,
}
lazy_static! {
    static ref WATCHING_TRACKING: Mutex<HashMap<i32, WatchingTracking>> = Mutex::new(HashMap::new());
}

fn get_titles() -> Vec<String> {
    let connection = Connection::new();
    let titles: Vec<String> = connection.unwrap().window_titles().unwrap();
    titles
}

#[tauri::command]
async fn anime_update_delay_loop() {

    loop { 

        anime_update_delay().await;

        check_delayed_updates(true).await;

        thread::sleep(Duration::from_secs(5)); 
    }
}

#[tauri::command]
async fn anime_update_delay() {

    //let mut filename: String = String::new();
    //let mut ignore_filenames: Vec<String> = Vec::new();
    let regex = Regex::new(r"\.mkv|\.avi|\.mp4").unwrap();
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

        let (media_id, media_score) = file_name_recognition::identify_media_id(&title_edit, &anime_data);
        if media_score < 0.8 { continue; }
        //println!("{} {} {} {:.4}", title_edit, episode.1, media_id, media_score);
        if watching_data.contains_key(&media_id) {
            watching_data.entry(media_id).and_modify(|entry| {
                entry.monitoring = true;
            });
        } else if user_data.contains_key(&media_id) && user_data.get(&media_id).unwrap().progress + 1 == episode { // only add if it is in the users list and it is the next episode
            watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode});
        }
    }

    for (media_id, tracking_info) in watching_data.iter_mut() {
        let seconds = tracking_info.timer.elapsed().as_secs();
        if seconds >= 1 * 60 {
            tracking_info.monitoring = false;
            // update anime

            user_data.entry(*media_id).and_modify(|ud| {
                ud.progress = tracking_info.episode;
            });

            api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), user_data.get(media_id).unwrap().clone()).await;
            *GLOBAL_REFRESH_UI.lock().await = true;
        }
        println!("{} {}/60", anime_data.get(media_id).unwrap().title.romaji.clone().unwrap(), seconds);
    }
    
    watching_data.retain(|_, v| v.monitoring == true);
}

#[tauri::command]
async fn get_refresh_ui() -> bool {
    thread::sleep(Duration::from_millis(1000));
    let mut refresh = GLOBAL_REFRESH_UI.lock().await;
    let refresh_clone = refresh.clone();
    *refresh = false;
    refresh_clone
}

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
    
    list
}

#[tauri::command]
async fn add_to_list(id: i32, list: String) {

    let mut user_anime = UserAnimeInfo::default();
    user_anime.media_id = id;
    user_anime.status = list;

    update_user_entry(user_anime).await;
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
async fn test() {

}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query,test,anilist_oauth_token,read_token_data,write_token_data,set_user_settings,
            get_user_settings,get_watching_list,get_list_user_info,get_anime_info,get_user_info,update_user_entry,get_list,on_startup,scan_anime_folder,
            play_next_episode,anime_update_delay,anime_update_delay_loop,get_refresh_ui,increment_decrement_episode,on_shutdown,episodes_exist,browse,add_to_list,remove_anime])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn get_user_data() {

    let response = api_calls::anilist_list_quary_call(GLOBAL_USER_SETTINGS.lock().await.username.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    
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