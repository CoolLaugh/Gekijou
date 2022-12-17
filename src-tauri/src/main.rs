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
use std::collections::HashMap;

use api_calls::TokenData;

use crate::api_calls::AnimeInfo;

lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    //static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
}

#[tauri::command]
async fn anilist_oauth_token(code: String) {
    
    *GLOBAL_TOKEN.lock().await = api_calls::anilist_get_access_token(code).await;

    write_token_data().await;
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
    //print!("writing file");
}

#[tauri::command]
async fn get_anime_info_query(id: i32) -> api_calls::AnimeInfo {
    
    let response = api_calls::anilist_api_call(id).await;    
    print!("{}", response.id);
    response
}

#[tauri::command]
async fn test() -> String {

    let response = api_calls::anilist_list_quary_call(GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    response
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query,test,anilist_oauth_token,read_token_data,write_token_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
