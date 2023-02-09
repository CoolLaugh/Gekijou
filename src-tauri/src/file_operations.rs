use std::fs::{self, create_dir};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::async_runtime::Mutex;

use crate::{GLOBAL_ANIME_DATA, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_ANIME_LISTS, GLOBAL_USER_SETTINGS, GLOBAL_TOKEN, GLOBAL_ANIME_PATH};

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

pub async fn write_file_episode_path() {
    write_file_data(&GLOBAL_ANIME_PATH, "episode_path").await;
}

pub async fn read_file_episode_path() {
    read_file_data(&GLOBAL_ANIME_PATH, "episode_path").await;
}

// writes all held data on anime to a file
async fn write_file_data<T: Serialize>(global: &Mutex<T>, filename: &str) {

    let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_backup_location = format!("{}/{}/{}_backup.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_path = Path::new(&file_location);
    let file_backup_path = Path::new(&file_backup_location);
    let mut file: File;

    // backup file before replacing it
    if file_path.exists() {

        // remove backup so we can make a new one
        if file_backup_path.exists() {
            match fs::remove_file(file_backup_path) {
                Err(why) => panic!("unable to remove, {}", why),
                Ok(file) => file,
            };
        }

        // change file into backup file
        match fs::rename(file_path, file_backup_path) {
            Err(why) => panic!("unable to move, {}", why),
            Ok(file) => file,
        };
    }

    if gekijou_folder_exists_or_created() {

        // create the file
        file = match File::create(file_path) {
            Err(why) => panic!("unable to open, {}", why),
            Ok(file) => file,
        };

        // write contents into file
        match file.write_all(serde_json::to_string(&*global.lock().await).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }
}

// reads all stored data from a file into the global collection
async fn read_file_data<T: DeserializeOwned>(global: &Mutex<T>, filename: &str) {

    let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_backup_location = format!("{}/{}/{}_backup.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_path = Path::new(&file_location);
    let file_backup_path = Path::new(&file_backup_location);

    if file_path.exists() {

        // open the file
        let mut file = match File::open(&file_path) {
            Err(why) => {
                // try to use the backup file if the file doesn't work
                if file_backup_path.exists() {
                    match File::open(&file_backup_path) {
                        Err(why2) => panic!("ERROR: {}", why2),
                        Ok(file) => file,
                    }
                } else {
                    panic!("ERROR: {}", why);
                }
            },
            Ok(file) => file,
        };
    
        // read all data out of the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
        
        *global.lock().await = serde_json::from_str(&buffer).unwrap();
    }
}

// checks if gekijou folder exists, if it does not exist it will try to create it
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



pub fn delete_data() -> bool {

    let files = vec!["token","token_backup","user_settings","user_settings_backup",
                                "anime_cache","anime_cache_backup","user_data","user_data_backup",
                                "user_Lists","user_Lists_backup","episode_path","episode_path_backup"];

    for file in files {

        let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, file);
        let file_path = Path::new(&file_location);
        if file_path.exists() {
            match fs::remove_file(file_path) {
                Err(_why) => return false,
                Ok(file) => file,
            };
        }
    }

    true
}