use std::fs::{self, create_dir};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use tauri::async_runtime::Mutex;

use crate::{GLOBAL_ANIME_DATA, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_ANIME_LISTS};
use crate::api_calls::TokenData;
use crate::api_calls::UserSettings;

extern crate dirs;

#[cfg(debug_assertions)]
const GEKIJOU_FOLDER: &str = "Gekijou_debug";

#[cfg(not(debug_assertions))]
const GEKIJOU_FOLDER: &str = "Gekijou";

// writes access token data to a file
pub fn write_file_token_data(token_data: &TokenData) {

    let token = format!("{}/{}/token.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let token_backup = format!("{}/{}/token_backup.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);

    let path = Path::new(&token);
    let path_backup = Path::new(&token_backup);
    let mut file: File;

    // backup file before replacing it
    if path.exists() {

        // remove backup so we can make a new one
        if path_backup.exists() {
            match fs::remove_file(path_backup) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(path, path_backup) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    if gekijou_folder_exists_or_created() {

        // create the file
        file = match File::create(path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        // write token data into file
        match file.write_all(serde_json::to_string(token_data).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }

}


// read access token data from the file
pub fn read_file_token_data() -> TokenData {

    let token = format!("{}/{}/token.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let path = Path::new(&token);
    let token_data = if path.exists() {

        let mut file = match File::open(&path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        let token_data: TokenData =  serde_json::from_str(&buffer).unwrap();
        token_data
    } else {
        TokenData::new()
    };

    token_data
}


// returns whether or not token file exists
pub fn token_data_file_exists() -> bool {

    let token = format!("{}/{}/token.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let path = Path::new(&token);
    path.exists()
}


// writes user settings to a file
pub fn write_file_user_settings(settings: &UserSettings) {

    let user_settings = format!("{}/{}/user_settings.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let user_settings_backup = format!("{}/{}/user_settings_backup.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);

    let user_settings_path = Path::new(&user_settings);
    let user_settings_backup_path = Path::new(&user_settings_backup);

    let mut file: File;

    // backup file before replacing it
    if user_settings_path.exists() {

        // remove backup so we can make a new one
        if user_settings_backup_path.exists() {
            match fs::remove_file(user_settings_backup_path) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(user_settings_path, user_settings_backup_path) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }
    
    if gekijou_folder_exists_or_created() {
        
        // create the file
        file = match File::create(user_settings_path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };
        
        // write user settings into file
        match file.write_all(serde_json::to_string(settings).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }
}


// reads user settings out of a file
pub fn read_file_user_settings() -> UserSettings {

    let user_settings = format!("{}/{}/user_settings.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let user_settings_path = Path::new(&user_settings);

    if user_settings_path.exists() {

        let mut file = match File::open(&user_settings_path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        let token_data: UserSettings = serde_json::from_str(&buffer).unwrap();
        token_data
    }
    else {
        return UserSettings::new();
    }
}


// writes all held data on anime to a file
pub async fn write_file_anime_info_cache() {

    let anime_cache = format!("{}/{}/anime_cache.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let anime_cache_backup = format!("{}/{}/anime_cache_backup.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let anime_cache_path = Path::new(&anime_cache);
    let anime_cache_backup_path = Path::new(&anime_cache_backup);
    let mut file: File;

    // backup file before replacing it
    if anime_cache_path.exists() {

        // remove backup so we can make a new one
        if anime_cache_backup_path.exists() {
            match fs::remove_file(anime_cache_backup_path) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(anime_cache_path, anime_cache_backup_path) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    if gekijou_folder_exists_or_created() {

        // create the file
        file = match File::create(anime_cache_path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        // write user settings into file
        match file.write_all(serde_json::to_string(&*GLOBAL_ANIME_DATA.lock().await).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }

}


// reads all stored data on anime from a file
pub async fn read_file_anime_info_cache() {

    let anime_cache = format!("{}/{}/anime_cache.txt", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER);
    let anime_cache_path = Path::new(&anime_cache);

    if anime_cache_path.exists() {

        let mut file = match File::open(&anime_cache_path) {
            Err(why) => panic!("unable to open {}", why),
            Ok(file) => file,
        };
    
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    
        *GLOBAL_ANIME_DATA.lock().await = serde_json::from_str(&buffer).unwrap();
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