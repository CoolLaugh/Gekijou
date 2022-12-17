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

use tauri::async_runtime::Mutex;
use std::{collections::HashMap};

use api_calls::{TokenData, UserSettings};

use crate::api_calls::{AnimeInfo, UserAnimeInfo};

lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_DATA: Mutex<HashMap<i32, UserAnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_LISTS: Mutex<HashMap<String, Vec<i32>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_SETTINGS: Mutex<UserSettings> = Mutex::new(UserSettings::new());
}

#[tauri::command]
async fn anilist_oauth_token(code: String) -> bool {
    
    let token = api_calls::anilist_get_access_token(code).await;

    if token.access_token.len() == 0 {
        return false;
    }
    else {
        *GLOBAL_TOKEN.lock().await = token;
    }

    write_token_data().await;

    true
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
async fn set_user_settings(username: String, title_language: String) {

    GLOBAL_USER_SETTINGS.lock().await.username = username;
    GLOBAL_USER_SETTINGS.lock().await.title_language = title_language;

    file_operations::write_file_user_settings(&*GLOBAL_USER_SETTINGS.lock().await);
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

    api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    file_operations::write_file_anime_info_cache().await;

    let lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = lists.get(&list_name).unwrap();

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
            let entry = anime_list.entry(item.clone()).or_insert(AnimeInfo::new()).clone();
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
        
        GLOBAL_USER_ANIME_LISTS.lock().await.entry(old_status.clone()).and_modify(|data|{ 
            data.remove(data.iter().position(|&v| v == anime.media_id).unwrap());
        });
        
        GLOBAL_USER_ANIME_LISTS.lock().await.entry(new_status).or_default().push(anime.media_id);
    }

    let response = api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), anime).await;
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let new_info: UserAnimeInfo = serde_json::from_value(json["data"]["SaveMediaListEntry"].to_owned()).unwrap();
    let media_id = new_info.media_id.clone();

    GLOBAL_USER_ANIME_DATA.lock().await.entry(media_id).and_modify(|entry| {
        *entry = new_info;
    });

}

#[tauri::command]
async fn on_startup() {

    *GLOBAL_TOKEN.lock().await = file_operations::read_file_token_data();
    file_operations::read_file_anime_info_cache().await;
}


#[tauri::command]
async fn test() -> String {

    file_name_recognition::parse_file_names(vec![String::from("D:\\Anime")]).await;

    //api_calls::anilist_get_list("Fuzzywuzhe".to_string(), "CURRENT".to_string(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    //let anime: Vec<i32> = [5114,9253,21202,17074,2904].to_vec();
    //let response = api_calls::test(0, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    //return response;
    return String::new();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query,test,anilist_oauth_token,read_token_data,write_token_data,set_user_settings,get_user_settings,get_watching_list,get_list_user_info,get_anime_info,get_user_info,update_user_entry,get_list,on_startup])
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