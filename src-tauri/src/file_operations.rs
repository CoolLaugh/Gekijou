use std::fs::{self, create_dir};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::async_runtime::Mutex;

use crate::{GLOBAL_ANIME_DATA, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_ANIME_LISTS, GLOBAL_USER_SETTINGS, GLOBAL_TOKEN};

extern crate dirs;

#[cfg(debug_assertions)]
const GEKIJOU_FOLDER: &str = "Gekijou_debug";

#[cfg(not(debug_assertions))]
const GEKIJOU_FOLDER: &str = "Gekijou";

// writes access token data to a file
pub async fn write_file_token_data() {
    write_file_data(&GLOBAL_TOKEN, "token").await;
}

// read access token data from the file
pub async fn read_file_token_data() {
    read_file_data(&GLOBAL_TOKEN, "token").await;
}

// writes user settings to a file
pub async fn write_file_user_settings() {
    write_file_data(&GLOBAL_USER_SETTINGS, "user_settings").await;
}

// reads user settings out of a file
pub async fn read_file_user_settings() {
    read_file_data(&GLOBAL_USER_SETTINGS, "user_settings").await;
}

// writes all held data on anime to a file
pub async fn write_file_anime_info_cache() {
    write_file_data(&GLOBAL_ANIME_DATA, "anime_cache").await;
}

// reads all stored data on anime from a file
pub async fn read_file_anime_info_cache() {
    read_file_data(&GLOBAL_ANIME_DATA, "anime_cache").await;
}

pub async fn write_file_user_info() {
    write_file_data(&GLOBAL_USER_ANIME_DATA, "user_data").await;
    write_file_data(&GLOBAL_USER_ANIME_LISTS, "user_Lists").await;
}

pub async fn read_file_user_info() {
    read_file_data(&GLOBAL_USER_ANIME_DATA, "user_data").await;
    read_file_data(&GLOBAL_USER_ANIME_LISTS, "user_Lists").await;
}

// writes all held data on anime to a file
async fn write_file_data<T: Serialize>(global: &Mutex<T>, filename: &str) {

    let user_info = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let user_info_backup = format!("{}/{}/{}_backup.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let user_info_path = Path::new(&user_info);
    let user_info_backup_path = Path::new(&user_info_backup);
    let mut file: File;

    // backup file before replacing it
    if user_info_path.exists() {

        // remove backup so we can make a new one
        if user_info_backup_path.exists() {
            match fs::remove_file(user_info_backup_path) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(user_info_path, user_info_backup_path) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    if gekijou_folder_exists_or_created() {

        // create the file
        file = match File::create(user_info_path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        // write user settings into file
        match file.write_all(serde_json::to_string(&*global.lock().await).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }
}

// reads all stored data on anime from a file
async fn read_file_data<T: DeserializeOwned>(global: &Mutex<T>, filename: &str) {

    let user_info = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let user_info_path = Path::new(&user_info);

    if user_info_path.exists() {

        let mut file = match File::open(&user_info_path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        *global.lock().await = serde_json::from_str(&buffer).unwrap();
    }
}

fn gekijou_folder_exists_or_created() -> bool {

    let folder = format!("{}/{}", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let gekijou_path = Path::new(&folder);
    if gekijou_path.exists() {
        true
    } else {
        match create_dir(gekijou_path) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(_e) => true,
        }
    }
}