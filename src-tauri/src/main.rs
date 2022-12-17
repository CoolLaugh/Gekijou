#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod secrets;
pub mod api_calls;
pub mod file_operations;

#[macro_use]
extern crate lazy_static;

use tauri::async_runtime::Mutex;
use std::{collections::HashMap, ptr::null};

use api_calls::{TokenData, UserSettings};

use crate::api_calls::{AnimeInfo, UserAnimeInfo};

lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_DATA: Mutex<HashMap<String, Vec<UserAnimeInfo>>> = Mutex::new(HashMap::new());
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
async fn get_watching_list(list_name: String) -> Vec<AnimeInfo> {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }

    let list_name_formatted = format!("\"{}\"", list_name);
    print!("\n{}", list_name_formatted);
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    let list = user_data.entry(String::from(list_name_formatted.clone())).or_insert(Vec::new());

    let mut missing_anime: Vec<i32> = Vec::new();

    {
        let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;
        for item in list {
            if anime_list.contains_key(&item.media_id) == false || anime_list[&item.media_id].cover_image.large.is_empty() {
                missing_anime.push(item.media_id);
            }
        }
    }

    print!("\nmissing anime: {}", missing_anime.len());
    api_calls::anilist_get_anime_info_split(missing_anime).await;
    file_operations::write_file_anime_info_cache().await;

    let mut return_data: Vec<AnimeInfo> = Vec::new();
    {
        let list2 = user_data.entry(String::from(list_name_formatted)).or_insert(Vec::new());
        let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;
        for item in list2 {
            //print!("\n{} ", item.media_id);
            let entry = anime_list.entry(item.media_id.clone()).or_insert(AnimeInfo::new()).clone();
            //print!("{}", entry.title.english.unwrap());
            return_data.push(entry);
        }
    }

    print!("\n{}\n", return_data.len());
    return_data
}

#[tauri::command]
async fn test() -> String {

    let anime: Vec<i32> = [5114,9253,21202,17074,2904].to_vec();
    api_calls::anilist_get_anime_info(anime.clone()).await;
    return String::new();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query,test,anilist_oauth_token,read_token_data,write_token_data,set_user_settings,get_user_settings,get_watching_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    
    
}

async fn get_user_data() {

    let response = api_calls::anilist_list_quary_call(GLOBAL_USER_SETTINGS.lock().await.username.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    //print!("\n{}", response);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();

    for item in json["data"]["MediaListCollection"]["lists"].as_array().unwrap() {

        let name: String = item["name"].to_string();

        let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
        let list = user_data.entry(name.clone()).or_insert(Vec::new());

        for item2 in item["entries"].as_array().unwrap() {

            list.push(serde_json::from_value(item2.clone()).unwrap());
        }
    }
}