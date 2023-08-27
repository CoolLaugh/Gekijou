use std::collections::{HashMap, HashSet};
use std::fs::{self, create_dir};
use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tauri::async_runtime::Mutex;

use crate::anime_data::{AnimeInfo, AnimePath};
use crate::user_data::{TokenData2, UserSettings, UserInfo};
use crate::GLOBAL_REFRESH_UI;

extern crate dirs;

#[cfg(debug_assertions)]
const GEKIJOU_FOLDER: &str = "Gekijou_debug";

#[cfg(not(debug_assertions))]
const GEKIJOU_FOLDER: &str = "Gekijou";

pub async fn write_file_token_data(token: &TokenData2) {
    write_file_data(&token, "token");
}

pub async fn read_file_token_data(token: &mut TokenData2) -> Result<(), &'static str> {
    read_file_data(token, "token").await
}

pub async fn write_file_user_settings(settings: &UserSettings) {
    write_file_data(&settings, "user_settings");
}

pub async fn read_file_user_settings(settings: &mut UserSettings) -> Result<(), &'static str> {
    read_file_data(settings, "user_settings").await
}

pub async fn write_file_user_data(user_data: &HashMap<i32, UserInfo>) {
    write_file_data(&user_data, "user_data");
}

pub async fn read_file_user_data(user_data: &mut HashMap<i32, UserInfo>) -> Result<(), &'static str> {
    read_file_data(user_data, "user_data").await
}

pub async fn write_file_user_lists(user_lists: &HashMap<String, Vec<i32>>) {
    write_file_data(&user_lists, "user_lists");
}

pub async fn read_file_user_lists(user_lists: &mut HashMap<String, Vec<i32>>) -> Result<(), &'static str> {
    read_file_data(user_lists, "user_lists").await
}

pub async fn write_file_update_queue(update_queue: &Vec<UserInfo>) {
    write_file_data(&update_queue, "update_queue");
}

pub async fn read_file_update_queue(update_queue: &mut Vec<UserInfo>) -> Result<(), &'static str> {
    read_file_data(update_queue, "update_queue").await
}

pub async fn write_file_anime_info_cache(anime_data: &HashMap<i32, AnimeInfo>) {
    write_file_data(&anime_data, "anime_cache");
}

pub async fn read_file_anime_info_cache(anime_data: &mut HashMap<i32, AnimeInfo>) -> Result<(), &'static str> {
    read_file_data(anime_data, "anime_cache").await
}

pub async fn write_file_anime_missing_ids(missing_ids: &HashSet<i32>) {
    write_file_data(&missing_ids, "404_ids");
}

pub async fn read_file_anime_missing_ids(missing_ids: &mut HashSet<i32>) -> Result<(), &'static str> {
    read_file_data(missing_ids, "404_ids").await
}

pub async fn write_file_episode_path(episode_path: &HashMap<i32, HashMap<i32,AnimePath>>) {
    write_file_data(&episode_path, "episode_path");
}

pub async fn read_file_episode_path(episode_path: &mut HashMap<i32, HashMap<i32,AnimePath>>) -> Result<(), &'static str> {
    read_file_data(episode_path, "episode_path").await
}

pub async fn write_file_known_files(known_files: &HashSet<String>) {
    write_file_data(&known_files, "known_files");
}

pub async fn read_file_known_files(known_files: &mut HashSet<String>) -> Result<(), &'static str> {
    read_file_data(known_files, "known_files").await
}



async fn write_file_data_mutex<T: Serialize>(global: &Mutex<T>, filename: &str) {
    write_file_data(&*global.lock().await, filename);
}

// writes all held data on anime to a file
pub fn write_file_data<T: Serialize>(global: &T, filename: &str) {
    
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
        match file.write_all(serde_json::to_string(global).unwrap().as_bytes()) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(file) => file,
        };
    }
}



async fn read_file_data_mutex<T: DeserializeOwned>(global: &Mutex<T>, filename: &str) -> Result<(), &'static str> {
    read_file_data(&mut *global.lock().await, filename).await
}
// reads all stored data from a file into the global collection
pub async fn read_file_data<T: DeserializeOwned>(global: &mut T, filename: &str) -> Result<(), &'static str> {

    let file_location = format!("{}/{}/{}.json", dirs::config_dir().unwrap().to_str().unwrap(), GEKIJOU_FOLDER, filename);
    let file_path = Path::new(&file_location);
    
    if file_path.exists() {

        // open the file
        let mut file = match File::open(&file_path) {
            Err(_why) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't open file"));
                return Err("Can't open file");
            },
            Ok(file) => file,
        };
    
        // read all data out of the file
        let mut buffer = String::new();
        match file.read_to_string(&mut buffer) {
            Err(_why) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't read file"));
                return Err("Can't open file");
            },
            Ok(file) => file,
        };
        
        match serde_json::from_str(&buffer) {
            Ok(result) => {
                *global = result;
            },
            Err(_error) => {
                GLOBAL_REFRESH_UI.lock().await.errors.push(String::from(filename.to_owned() + " Can't process json"));
            },
        }
    }

    Ok(())
}




// checks if gekijou folder exists, if it does not exist it will try to create it
// returns true if the folder exists
fn gekijou_folder_exists_or_created() -> bool {

    let folder = dirs::config_dir().unwrap().join(GEKIJOU_FOLDER);
    if folder.exists() {
        true
    } else {
        match create_dir(folder) {
            Err(why) => panic!("ERROR: {}", why),
            Ok(_e) => true,
        }
    }
}



// delete all files that store information between sessions
pub fn delete_data() -> bool {

    let files = vec!["token","token_backup","user_settings","user_settings_backup","anime_cache","anime_cache_backup",
                                "user_data","user_data_backup","user_lists","user_lists_backup","episode_path","episode_path_backup",
                                "update_queue","update_queue_backup","known_files","known_files_backup","404_ids","404_ids_backup"];

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