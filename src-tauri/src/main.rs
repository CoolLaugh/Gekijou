#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod constants;
pub mod secrets;
pub mod api_calls;
pub mod file_operations;
pub mod file_name_recognition;
pub mod rss_parser;
pub mod recommendation;
pub mod file_name_recognition_tests;

#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use file_name_recognition_tests::FilenameTest;
use regex::Regex;
use rss_parser::RssEntry;
use serde::{Serialize, Deserialize};
use tauri::async_runtime::Mutex;
use tauri::Manager;
use window_titles::{Connection, ConnectionTrait};
use std::{collections::HashMap, path::Path, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use open;
use api_calls::{TokenData, UserSettings, AnilistDate};
use crate::{api_calls::{AnimeInfo, UserAnimeInfo}, file_name_recognition::AnimePath};



//stores details on which parts of the UI need to be refreshed
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RefreshUI {
    pub anime_list: bool,
    pub tracking_progress: bool,
    pub canvas: bool,
}

impl RefreshUI {
    
    pub fn clear(&mut self) {
        self.anime_list = false;
        self.tracking_progress = false;
        self.canvas = false;
    }
}



lazy_static! {
    static ref GLOBAL_TOKEN: Mutex<TokenData> = Mutex::new(TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() });
    static ref GLOBAL_ANIME_DATA: Mutex<HashMap<i32, AnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_DATA: Mutex<HashMap<i32, UserAnimeInfo>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_ANIME_LISTS: Mutex<HashMap<String, Vec<i32>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_USER_SETTINGS: Mutex<UserSettings> = Mutex::new(UserSettings::new());
    static ref GLOBAL_ANIME_PATH: Mutex<HashMap<i32,HashMap<i32,AnimePath>>> = Mutex::new(HashMap::new());
    static ref GLOBAL_REFRESH_UI: Mutex<RefreshUI> = Mutex::new(RefreshUI::default());
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
            if settings.folders[i] == "" {
                continue;
            }
            if user_settings.folders[i] != settings.folders[i] {
                scan = true;
            }
        }
    }

    let old_current_tab = user_settings.current_tab.clone(); // don't change this value
    let score_format = user_settings.score_format.clone(); // don't change this value
    let old_username = user_settings.username.clone();
    *user_settings = settings;
    user_settings.score_format = score_format;
    user_settings.current_tab = old_current_tab;

    // user is different, their list and score format will be different
    if old_username != user_settings.username {
        GLOBAL_USER_ANIME_LISTS.lock().await.clear();
        GLOBAL_USER_ANIME_DATA.lock().await.clear();

        user_settings.score_format = api_calls::get_user_score_format(user_settings.username.clone()).await;
    }

    drop(user_settings);
    if scan {
        file_name_recognition::parse_file_names(None).await;
    }

    file_operations::write_file_user_settings().await;
}



// retrieves user's settings from a file
#[tauri::command]
async fn get_user_settings() -> UserSettings {
    load_user_settings().await;
    GLOBAL_USER_SETTINGS.lock().await.clone()
}



// retrieves user's settings from a file
#[tauri::command]
async fn set_current_tab(current_tab: String) {
    GLOBAL_USER_SETTINGS.lock().await.current_tab = current_tab;
    file_operations::write_file_user_settings().await;
}



// gets anime data for all anime in a specific list
#[tauri::command]
async fn get_list(list_name: String) -> (Vec<AnimeInfo>, Option<String>) {

    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        if error_message.is_some() {
            return (Vec::new(), error_message)
        }
        file_operations::write_file_anime_info_cache(&*GLOBAL_ANIME_DATA.lock().await);
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



// returns all information of anime on the users anime list
// information is paged, 50 entries are returned per page
// sorting and ascending are only used for page 0, other pages use the sorting order of the last time page 0 was called
#[tauri::command]
async fn get_list_paged(list_name: String, sort: String, ascending: bool, page: usize) -> (Vec<(AnimeInfo, UserAnimeInfo)>, Option<String>){

    // list won't exist if user doesn't exist
    if GLOBAL_USER_SETTINGS.lock().await.username.is_empty() {
        return (Vec::new(), None);
    }

    // get list from anilist if it does not exist
    if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&list_name) == false {
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), list_name.clone(), GLOBAL_TOKEN.lock().await.access_token.clone()).await;
        if let Some(error_message_string) = error_message {
            //println!("{}", error_message_string);
            return (Vec::new(), Some(error_message_string));
        }
        file_operations::write_file_anime_info_cache(&*GLOBAL_ANIME_DATA.lock().await);
        file_operations::write_file_user_info().await;
    }

    let mut anime_lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let list = anime_lists.get_mut(&list_name).unwrap();
    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let user_data = GLOBAL_USER_ANIME_DATA.lock().await;

    // before showing the list for the first time check for missing information, sort by selected category, and check for airing times
    if page == 0 {

        // check for missing information
        let unknown_ids: Vec<i32> = list.iter().map(|id| *id).filter(|&id| anime_data.contains_key(&id) == false).collect();
        if unknown_ids.is_empty() == false {
            
            api_calls::anilist_api_call_multiple(unknown_ids, &mut anime_data).await;
            file_operations::write_file_anime_info_cache(&anime_data);
        }
        
        sort_list(list, &anime_data, &user_data, sort).await;
        // if list is descending
        if ascending == false {
            list.reverse();
        }

        // check for next airing episode that is in the past and update it with a new time
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs() as i32;

        // get airing times for anime if they are outdated
        let get_airing_time_ids = list.iter()
            .map(|id| *id)
            .filter(|id| 
                if let Some(anime) = anime_data.get(id) {
                    if let Some(airing) = &anime.next_airing_episode {
                        airing.airing_at < current_time
                    } else {
                        false
                    }
                } else {
                    false
                }
            ).collect();
        api_calls::anilist_airing_time(get_airing_time_ids, &mut anime_data).await;
    }

    let start = page * constants::ANIME_PER_PAGE;
    let finish = // list bounds check
    if (page + 1) * constants::ANIME_PER_PAGE > list.len() {
        list.len()
    } else {
        (page + 1) * constants::ANIME_PER_PAGE
    };

    // prepare list to return
    let mut list_info: Vec<(AnimeInfo, UserAnimeInfo)> = Vec::new();
    for i in start..finish {
        if let Some(id) = list.get(i) {
            if let Some(anime_entry) = anime_data.get(id) {
                if let Some(user_entry) = user_data.get(id) {
                    list_info.push((anime_entry.clone(), user_entry.clone()));
                }
            }
        }
    }

    (list_info, None)
}



// sort a list of anime ids
// sort_category determines what value is used to sort them and anime_data and user_data contain information used for sorting
async fn sort_list(list: &mut Vec<i32>, anime_data: &HashMap<i32, AnimeInfo>, user_data: &HashMap<i32, UserAnimeInfo>, sort_category: String) {

    match sort_category.as_str() {
        "Alphabetical" => {
            let title_language = GLOBAL_USER_SETTINGS.lock().await.title_language.clone();
            match title_language.as_str() {
                "romaji" => list.sort_by(|i, j| { 
                    if let Some(left_anime) = anime_data.get(i) {
                        if let Some(right_anime) = anime_data.get(j) {
                            if let Some(left_romaji_title) = &left_anime.title.romaji {
                                if let Some(right_romaji_title) = &right_anime.title.romaji {
                                    left_romaji_title.to_lowercase().partial_cmp(&right_romaji_title.to_lowercase()).unwrap()
                                } else {
                                    println!("anime: {} has no romaji title", j);
                                    std::cmp::Ordering::Equal // should never happen
                                }
                            } else {
                                println!("anime: {} has no romaji title", i);
                                std::cmp::Ordering::Equal // should never happen
                            }
                        } else {
                            println!("anime: {} has no data", j);
                            std::cmp::Ordering::Equal // should never happen
                        }
                    } else {
                        println!("anime: {} has no data", i);
                        std::cmp::Ordering::Equal // should never happen
                    }
                }),
                "english" => list.sort_by(|i, j| { 
                    if let Some(left_anime) = anime_data.get(i) {
                        if let Some(right_anime) = anime_data.get(j) {
                            if let Some(left_english_title) = &left_anime.title.english {
                                if let Some(right_english_title) = &right_anime.title.english {
                                    left_english_title.to_lowercase().partial_cmp(&right_english_title.to_lowercase()).unwrap()
                                } else if let Some(right_romaji_title) = &right_anime.title.romaji {
                                    left_english_title.to_lowercase().partial_cmp(&right_romaji_title.to_lowercase()).unwrap()
                                } else {
                                    println!("anime: {} has no english or romaji title", j);
                                    std::cmp::Ordering::Equal // should never happen, all anime so far has had romaji if they don't have a english title
                                }
                            } else if let Some(left_romaji_title) = &left_anime.title.romaji { 
                                if let Some(right_english_title) = &right_anime.title.english {
                                    left_romaji_title.to_lowercase().partial_cmp(&right_english_title.to_lowercase()).unwrap()
                                } else if let Some(right_romaji_title) = &right_anime.title.romaji {
                                    left_romaji_title.to_lowercase().partial_cmp(&right_romaji_title.to_lowercase()).unwrap()
                                } else {
                                    println!("anime: {} has no english or romaji title", j);
                                    std::cmp::Ordering::Equal // should never happen, all anime so far has had romaji if they don't have a english title
                                }
                            } else {
                                println!("anime: {} has no english or romaji title", i);
                                std::cmp::Ordering::Equal // should never happen, all anime so far has had romaji if they don't have a english title
                            }
                        } else {
                            println!("anime: {} has no data", j);
                            std::cmp::Ordering::Equal // should never happen
                        }
                    } else {
                        println!("anime: {} has no data", i);
                        std::cmp::Ordering::Equal // should never happen
                    }
                }),
                "native" => list.sort_by(|i, j| { 
                    if let Some(left_anime) = anime_data.get(i) {
                        if let Some(right_anime) = anime_data.get(j) {
                            if let Some(left_native_title) = &left_anime.title.native {
                                if let Some(right_native_title) = &right_anime.title.native {
                                    left_native_title.to_lowercase().partial_cmp(&right_native_title.to_lowercase()).unwrap()
                                } else {
                                    println!("anime: {} has no native title", j);
                                    std::cmp::Ordering::Equal // should never happen
                                }
                            } else {
                                println!("anime: {} has no native title", i);
                                std::cmp::Ordering::Equal // should never happen
                            }
                        } else {
                            println!("anime: {} has no data", j);
                            std::cmp::Ordering::Equal // should never happen
                        }
                    } else {
                        println!("anime: {} has no data", i);
                        std::cmp::Ordering::Equal // should never happen
                    }
                }),
                &_ => (),
            }
        },
        "Score" => list.sort_by(|i, j| { 
            if let Some(left_anime) = anime_data.get(i) {
                if let Some(right_anime) = anime_data.get(j) {
                    left_anime.average_score.partial_cmp(&right_anime.average_score).unwrap() 
                } else {
                    println!("anime: {} has no data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "MyScore" => list.sort_by(|i, j| { 
            if let Some(left_anime) = user_data.get(i) {
                if let Some(right_anime) = user_data.get(j) {
                    left_anime.score.partial_cmp(&right_anime.score).unwrap()
                } else {
                    println!("anime: {} has no data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "Date" => list.sort_by(|i, j| { 
            if let Some(left_anime) = anime_data.get(i) {
                if let Some(right_anime) = anime_data.get(j) {
                    left_anime.start_date.partial_cmp(&right_anime.start_date).unwrap() 
                } else {
                    println!("anime: {} has no data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "Popularity" => list.sort_by(|i, j| { 
            if let Some(left_anime) = anime_data.get(i) {
                if let Some(right_anime) = anime_data.get(j) {
                    left_anime.popularity.partial_cmp(&right_anime.popularity).unwrap() 
                } else {
                    println!("anime: {} has no data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "Trending" => list.sort_by(|i, j| { 
            if let Some(left_anime) = anime_data.get(i) {
                if let Some(right_anime) = anime_data.get(j) {
                    left_anime.trending.partial_cmp(&right_anime.trending).unwrap() 
                } else {
                    println!("anime: {} has no data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "Started" => list.sort_by(|i, j| { 
            if let Some(left_anime) = user_data.get(i) {
                if let Some(right_anime) = user_data.get(j) {
                    left_anime.started_at.partial_cmp(&right_anime.started_at).unwrap() 
                } else {
                    println!("anime: {} has no user data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no user data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        "Completed" => list.sort_by(|i, j| { 
            if let Some(left_anime) = user_data.get(i) {
                if let Some(right_anime) = user_data.get(j) {
                    left_anime.completed_at.partial_cmp(&right_anime.completed_at).unwrap() 
                } else {
                    println!("anime: {} has no user data", j);
                    std::cmp::Ordering::Equal // should never happen
                }
            } else {
                println!("anime: {} has no user data", i);
                std::cmp::Ordering::Equal // should never happen
            }
        }),
        &_ => (),
    }
}



// get all user data for all anime in a specific list
#[tauri::command]
async fn get_list_user_info(list_name: String) -> Vec<UserAnimeInfo> {

    if GLOBAL_USER_ANIME_DATA.lock().await.is_empty() {
        get_user_data().await;
    }

    let mut list: Vec<UserAnimeInfo> = Vec::new();
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    for item in GLOBAL_USER_ANIME_LISTS.lock().await.entry(list_name.clone()).or_insert(Vec::new()) {

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

        let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * constants::SECONDS_IN_MINUTES) as f64;
        
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
async fn get_anime_info(id: i32) -> AnimeInfo {

    if GLOBAL_ANIME_DATA.lock().await.is_empty() {
        file_operations::read_file_anime_info_cache().await;
    }

    if GLOBAL_ANIME_DATA.lock().await.contains_key(&id) == false {
        api_calls::anilist_get_anime_info_single(id).await;
    }

    let anime_data = GLOBAL_ANIME_DATA.lock().await.get(&id).unwrap().clone();
    anime_data
}



// updates a entry on anilist with new information
#[tauri::command]
async fn update_user_entry(mut anime: UserAnimeInfo) {

    let old_status: String = if GLOBAL_USER_ANIME_DATA.lock().await.contains_key(&anime.media_id) {
        GLOBAL_USER_ANIME_DATA.lock().await.entry(anime.media_id).or_default().status.clone()
    } else {
        String::new()
    };

    // repeating and current are combined into the watching list
    let old_list: String = if old_status == "REPEATING" {
        String::from("CURRENT")
    } else {
        old_status.clone()
    };
    
    // repeating and current are combined into the watching list
    let new_list = if anime.status == "REPEATING" {
        String::from("CURRENT")
    } else {
        anime.status.clone()
    };

    // we need to change what list the anime is in
    if old_list != new_list {
        
        // if the anime is not newly added
        if old_status.is_empty() == false {

            GLOBAL_USER_ANIME_LISTS.lock().await.entry(old_list.clone()).and_modify(|data|{ 
                data.remove(data.iter().position(|&id| id == anime.media_id).unwrap());
            });
        }
        
        GLOBAL_USER_ANIME_LISTS.lock().await.entry(new_list.clone()).and_modify(|list| {
            if list.contains(&anime.media_id) == false {
                list.push(anime.media_id);
            }
        }).or_insert(vec![anime.media_id]);
    }

    // update completed and started date if show is completed
    if new_list == "COMPLETED" {

        if anime.status != "REPEATING" { // don't change original start and end date if rewatching

            // set completed date to today
            if anime.completed_at.is_none() { // user didn't input a date
                let now: DateTime<Local> = Local::now();
                anime.completed_at = Some(AnilistDate {
                    year: Some(now.year()),
                    month: Some(now.month() as i32),
                    day: Some(now.day() as i32),
                });
            }
            if let Some(started_at) = &anime.started_at {
                if started_at.day.is_none() && started_at.month.is_none() && started_at.year.is_none() { // user didn't input a date
                    // set if anime is a movie or special so the user will watch it in one sitting
                    if let Some(anime_data_entry) = GLOBAL_ANIME_DATA.lock().await.get(&anime.media_id) {
                        if let Some(episodes) = anime_data_entry.episodes {
                            if episodes <= 1 { // anime is a movie or special
                                anime.started_at = anime.completed_at.clone();
                            }
                        }
                    }
                    // set if user watched the whole series at once
                    if let Some(user_entry) = GLOBAL_USER_ANIME_DATA.lock().await.get(&anime.media_id) {
                        if user_entry.progress == 0 {
                            anime.started_at = anime.completed_at.clone();
                        }
                    }
                }
            } else {
                println!("ERROR: started_at is None"); // javascript should always call this function with started_at existing
            }
        }
    }

    // update anilist
    let response = api_calls::update_user_entry(GLOBAL_TOKEN.lock().await.access_token.clone(), anime).await;

    // update user date to match anilist
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let new_info: UserAnimeInfo = serde_json::from_value(json["data"]["SaveMediaListEntry"].to_owned()).unwrap();
    GLOBAL_USER_ANIME_DATA.lock().await.insert(new_info.media_id.clone(), new_info);
    file_operations::write_file_user_info().await;
}



// loads data from files and looks for episodes on disk
#[tauri::command]
async fn on_startup() {
    file_operations::read_file_token_data().await;
    file_operations::read_file_anime_info_cache().await;
    file_operations::read_file_user_info().await;
    file_operations::read_file_episode_path().await;
    if GLOBAL_USER_SETTINGS.lock().await.score_format.is_empty() && GLOBAL_USER_SETTINGS.lock().await.username.is_empty() == false {
        GLOBAL_USER_SETTINGS.lock().await.score_format = api_calls::get_user_score_format(GLOBAL_USER_SETTINGS.lock().await.username.clone()).await;
    }
    *GLOBAL_STARTUP_FINISHED.lock().await = true;
}



lazy_static! {
    static ref GLOBAL_SETTINGS_LOADED: Mutex<bool> = Mutex::new(false);
    static ref GLOBAL_STARTUP_FINISHED: Mutex<bool> = Mutex::new(false);
}
// loads data from files and looks for episodes on disk
#[tauri::command]
async fn load_user_settings() {
    
    let mut loaded = GLOBAL_SETTINGS_LOADED.lock().await;
    if *loaded == false {
        file_operations::read_file_user_settings().await;
        *loaded = true;
    }
    
    // settings not in older versions of gekijou must be filled in
    let mut user_settings = GLOBAL_USER_SETTINGS.lock().await;
    if user_settings.highlight_color.is_empty() {
        user_settings.highlight_color = String::from(constants::DEFAULT_HIGHLIGHT_COLOR);
    }
    
    if user_settings.show_airing_time.is_none() {
        user_settings.show_airing_time = Some(true);
    }
    
    if user_settings.theme.is_none() {
        user_settings.theme = Some(0);
    }
}



// go ahead with any updates that haven't been completed yet before closing
#[tauri::command]
async fn on_shutdown() {

    check_delayed_updates(false).await;
}



// check if enough time has passed before updating the episode of a anime
// this delay is to prevent spamming or locking when the user increases or decreases the episode count multiple times
async fn check_delayed_updates(wait: bool) {
    
    let mut delayed_update = GLOBAL_UPDATE_ANIME_DELAYED.lock().await;
    for (id, time) in delayed_update.iter() {
        
        if time.elapsed() >= Duration::from_secs(constants::ANIME_UPDATE_DELAY) || wait == false {

            if let Some(anime) = GLOBAL_USER_ANIME_DATA.lock().await.get(id) {

                let access_token = GLOBAL_TOKEN.lock().await.access_token.clone();
                if access_token.is_empty() == false {
                    api_calls::update_user_entry(access_token, anime.clone()).await;
                } else {
                    println!("can't update anime, access token is empty");
                }
            }
        }
    }
    
    delayed_update.retain(|_, v| v.elapsed() < Duration::from_secs(constants::ANIME_UPDATE_DELAY));
}



// opens the file for the next episode in the default program
#[tauri::command]
async fn play_next_episode(id: i32) {
    
    let next_episode = GLOBAL_USER_ANIME_DATA.lock().await.get(&id).unwrap().progress + 1;

    if play_episode(id, next_episode).await == false {
        // if episode location is unknown, search for new episodes and try again
        file_name_recognition::parse_file_names(None).await;
        play_episode(id, next_episode).await;
    }
}



// play the episode from the anime id
// returns true if the episode was played
async fn play_episode(anime_id: i32, episode: i32) -> bool {

    let mut episode_opened = false;
    let paths = GLOBAL_ANIME_PATH.lock().await;
    if paths.contains_key(&anime_id) {
        let media = paths.get(&anime_id).unwrap();
        if media.contains_key(&episode) {
            
            let next_episode_path = Path::new(&media.get(&episode).unwrap().path);
            match open::that(next_episode_path) {
                Err(why) => panic!("{}",why),
                Ok(_e) => { episode_opened = true },
            }
        }
    }

    episode_opened
}



// changes the progress for a anime by +-1
// anilist api call is delayed to prevent spam/locking
#[tauri::command]
async fn increment_decrement_episode(anime_id: i32, change: i32) {

    // progress can only be changed by 1 or -1
    if change.abs() != 1 {
        return;
    }

    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    if let Some(user_entry) = user_data.get(&anime_id) {
        
        // can't go beyond the last episode
        let anime_data = GLOBAL_ANIME_DATA.lock().await;
        if let Some(anime_entry) = anime_data.get(&anime_id) {
            if let Some(max_episodes) = anime_entry.episodes {
                if change == 1 && user_entry.progress == max_episodes {
                    return;
                }
            }
        }

        // you can't go below 0 progress
        if change == -1 && user_entry.progress == 0 {
            return;
        }

        // change episode number
        if let Some(user_entry) = user_data.get_mut(&anime_id) {
            if let Some(anime_entry) = anime_data.get(&anime_id) {
                change_episode(user_entry, user_entry.progress + change, anime_entry.episodes).await;
            } else {
                println!("anime data is missing {}", anime_id);
            }
        } else {
            println!("user data is missing {}", anime_id);
        }

        // add anime to delayed update queue
        GLOBAL_UPDATE_ANIME_DELAYED.lock().await.insert(anime_id, Instant::now());

    } else {
        println!("user data is missing {}", anime_id);
    }
    drop(user_data);
    file_operations::write_file_user_info().await;
}



// scan folders for episodes of anime
#[tauri::command]
async fn scan_anime_folder() {
    file_name_recognition::parse_file_names(None).await;
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
    let titles: Vec<String> = connection.unwrap().window_titles().unwrap();
    titles
}



// scans for and identifies windows playing anime and sets up a delayed update
#[tauri::command]
async fn anime_update_delay() {

    let regex = Regex::new(r"\.mkv|\.avi|\.mp4").unwrap();

    let vlc_remove = Regex::new(r" - VLC media player").unwrap();
    let gom_remove = Regex::new(r" - GOM Player").unwrap();
    let zoom_remove = Regex::new(r" - Zoom Player MAX").unwrap();
    let mpv_remove = Regex::new(r" - mpv").unwrap();
    let pot_remove = Regex::new(r" - PotPlayer").unwrap();

    let delay = (GLOBAL_USER_SETTINGS.lock().await.update_delay * constants::SECONDS_IN_MINUTES)  as u64;
    let language = GLOBAL_USER_SETTINGS.lock().await.title_language.clone();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut watching_data = WATCHING_TRACKING.lock().await;
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;

    // reset monitoring
    watching_data.iter_mut().for_each(|(_, entry)| {
        entry.monitoring = false;
    });

    // get window titles and keep the ones with video files
    let mut titles: Vec<String> = get_titles(); // for some reason mutex locking has to happen before this function
    titles.retain(|title| regex.is_match(title));

    for title in titles {

        // remove file extension
        let mut title_edit: String = regex.replace(&title, "").to_string();

        // remove video player suffixes 
        title_edit = vlc_remove.replace(&title_edit, "").to_string();
        title_edit = gom_remove.replace(&title_edit, "").to_string();
        title_edit = zoom_remove.replace(&title_edit, "").to_string();
        title_edit = mpv_remove.replace(&title_edit, "").to_string();
        title_edit = pot_remove.replace(&title_edit, "").to_string();
        
        // remove extra information from the filename like episode, codec, etc
        title_edit = file_name_recognition::remove_brackets(&title_edit);
        let (episode_str, mut episode, length) = file_name_recognition::identify_number(&title_edit);
        title_edit = title_edit.replace(episode_str.as_str(), "");
        title_edit = file_name_recognition::irrelevant_information_removal(title_edit);

        // identify what anime it belongs to, if it doesn't belong to any then skip this title
        let (mut media_id, _, media_score) = file_name_recognition::identify_media_id(&title_edit, &anime_data, None);
        if media_score < constants::SIMILARITY_SCORE_THRESHOLD { 
            continue;
        }

        // check for high episode numbers
        file_name_recognition::replace_with_sequel(&mut media_id, &mut episode, &anime_data);

        // marks movies, etc as episode 1 because movies don't have a episode number
        file_name_recognition::episode_fix(media_id, &mut episode, &anime_data);

        let next_episode: bool = episode > user_data.get(&media_id).unwrap().progress && episode <= user_data.get(&media_id).unwrap().progress + length;

        // if the file is being monitored and the episode is the next episode
        if let Some(entry) = watching_data.get_mut(&media_id) {
            if next_episode && entry.episode == episode {
                entry.monitoring = true;
            }
        // only add if it is in the users list, it is the next episode, and the episode is within range
        } else if user_data.contains_key(&media_id) && 
            next_episode && 
            episode > 0 {

            if let Some(anime_entry) = anime_data.get(&media_id) {
                if anime_entry.episodes.is_none() || episode <= anime_entry.episodes.unwrap() {
                    
                    let title = if language == "romaji" {
                        if let Some(romaji_title) = anime_entry.title.romaji.clone() {
                            romaji_title
                        } else {
                            String::from("romaji missing")
                        }
                    } else if language == "english" {
                        if let Some(english_title) = anime_entry.title.english.clone() {
                            english_title
                        } else {
                            anime_entry.title.romaji.clone().unwrap()
                        }
                    } else if language == "native" {
                        if let Some(native_title) = anime_entry.title.native.clone() {
                            native_title
                        } else {
                            String::from("native missing")
                        }
                    } else {
                        String::from("language selection error")
                    };
                    watching_data.insert(media_id, WatchingTracking { timer: std::time::Instant::now(), monitoring: true, episode: episode, length: length, title: title});
                }
            } else {
                println!("anime_data is missing {}", media_id);
            }
        }
    }

    let mut save_file = false;
    let mut update_entries: Vec<UserAnimeInfo> = Vec::new();
    // check if media has been playing for long enough to update
    for (media_id, tracking_info) in watching_data.iter_mut() {
        let seconds = tracking_info.timer.elapsed().as_secs();
        if seconds >= delay {
            // user progress will be updated to this episode so we no longer want to monitor it
            tracking_info.monitoring = false;
            
            // update anime
            if let Some(user_entry) = user_data.get_mut(&media_id) {
                if let Some(anime_entry) = anime_data.get(media_id) {
                    change_episode(user_entry, tracking_info.episode + tracking_info.length - 1, anime_entry.episodes).await;

                    // save changes to file
                    save_file = true;
        
                    // store entry for later after mutexes are dropped
                    update_entries.push(user_entry.clone());
                    
                    // update ui with episode progress
                    GLOBAL_REFRESH_UI.lock().await.canvas = true;
                } else {
                    println!("anime_data is missing {}", media_id);
                }
            } else {
                println!("user_data is missing {}", media_id);
            }
        }
    }

    // remove episodes that are no longer being played
    watching_data.retain(|_, v| v.monitoring == true);

    // unlock mutexes before doing api calls which might block other threads
    drop(anime_data);
    drop(watching_data);
    drop(user_data);

    let access_token = GLOBAL_TOKEN.lock().await.access_token.clone();
    for anime in update_entries {
        // update anilist with new episode/status
        api_calls::update_user_entry(access_token.clone(), anime).await;
    }
    // update the file with the new episode/status
    if save_file {
        file_operations::write_file_user_info().await;
    }
}



// change the episode for the user entry and trigger other changes depending on the episode
async fn change_episode(anime: &mut UserAnimeInfo, episode: i32, max_episodes: Option<i32>) {

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
    if episode > progress && (max_episodes.is_none() || episode != max_episodes.unwrap()) && anime.status != "CURRENT"{

        change_list(anime, String::from("CURRENT")).await;
        GLOBAL_REFRESH_UI.lock().await.anime_list = true;
    }

    // add anime to completed if the last episode was watched and set complete date
    if max_episodes.is_some() && episode == max_episodes.unwrap() {
        // don't change completed date if the user is rewatching
        if anime.status != "REPEATING" { // don't change original start and end date if rewatching

            let now: DateTime<Local> = Local::now();
            anime.completed_at = Some(AnilistDate {
                year: Some(now.year()),
                month: Some(now.month() as i32),
                day: Some(now.day() as i32),
            });
        }
    
        if anime.status != "COMPLETED" {

            change_list(anime, String::from("COMPLETED")).await;
        }
        GLOBAL_REFRESH_UI.lock().await.anime_list = true;
    }
}



// change what list a anime belongs to
// new_list can be any status including REPEATING, REPEATING shows will be placed into the CURRENT list
// CURRENT and REPEATING shows are treated differently even though they are in the same list
async fn change_list(anime: &mut UserAnimeInfo, new_list: String) {

    let mut lists = GLOBAL_USER_ANIME_LISTS.lock().await;
    let old_list = if anime.status == "REPEATING" {
        String::from("CURRENT") // repeating and current are combined together
    } else {
        anime.status.clone()
    };

    // remove from old list
    lists.entry(old_list).and_modify(|list| {
        let index = list.iter().position(|v| *v == anime.media_id).unwrap();
        list.remove(index);
    });

    anime.status = new_list;
    let new_list = if anime.status == "REPEATING" {
        String::from("CURRENT") // repeating and current are combined together
    } else {
        anime.status.clone()
    };

    // insert into new list
    lists.entry(new_list).and_modify(|list| {
        
        if list.contains(&anime.media_id) == false {

            list.push(anime.media_id);
        }
    });
}



// allows the ui to check if a anime has been updated to determine if the ui will be refreshed
#[tauri::command]
async fn refresh_ui() -> RefreshUI {

    let mut refresh = GLOBAL_REFRESH_UI.lock().await;
    let mut refresh_ui = refresh.clone();
    refresh_ui.tracking_progress = WATCHING_TRACKING.lock().await.len() > 0;
    refresh.anime_list = false;
    refresh.canvas = false;

    refresh_ui
}



lazy_static! {
    static ref SCAN_TIMER: Mutex<Instant> = Mutex::new(Instant::now());
    static ref STARTUP_SCAN: Mutex<bool> = Mutex::new(false);
}
// performs periodic tasks like checking for anime in media players, delayed updates that must be sent, scanning folders for files
// it's expected that this function will be called periodically from the UI, it won't loop on its own
#[tauri::command]
async fn background_tasks() {
    
    // check for anime in media players
    anime_update_delay().await;
    // update anilist with delayed updates
    check_delayed_updates(true).await;

    // scan files for new episodes of anime every hour and a short time after startup
    let mut on_startup_scan_completed = STARTUP_SCAN.lock().await;
    let mut timer = SCAN_TIMER.lock().await;
    if timer.elapsed() > Duration::from_secs(constants::ONE_HOUR) || 
        (timer.elapsed() >= Duration::from_secs(constants::STARTUP_SCAN_DELAY) && *on_startup_scan_completed == false) {

        if file_name_recognition::parse_file_names(None).await {

            GLOBAL_REFRESH_UI.lock().await.canvas = true;
        }
        *on_startup_scan_completed = true;
        *timer = Instant::now();
    }
}



// returns a list of what episodes of what anime exist on disk
#[tauri::command]
async fn episodes_exist() -> HashMap<i32, Vec<i32>> {
    
    let paths = GLOBAL_ANIME_PATH.lock().await;
    let mut episodes_exist: HashMap<i32, Vec<i32>> = HashMap::new();

    for (anime_id, episodes) in paths.iter() {

        let mut episode_list: Vec<i32> = Vec::new();

        for (episode, _) in episodes {

            episode_list.push(*episode);
        }

        episodes_exist.insert(*anime_id, episode_list);
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



// returns a list of anime based on filters and sorting order
// anime in user's list does not matter and user login is not used
#[tauri::command]
async fn browse(year: String, season: String, genre: String, format: String, search: String, order: String) -> Vec<AnimeInfo> {

    let mut list: Vec<AnimeInfo> = Vec::new();

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut has_next_page = true;

    // each page has 50 entries, loop until all entries are retrieved or the limit is reached
    let mut page = 0;
    while has_next_page {
        
        let response = api_calls::anilist_browse_call(page, year.clone(), season.clone(), genre.clone(), format.clone(), search.clone(), order.clone()).await;
        page += 1;

        // add anime to return list and global data for further uses
        for anime in response["data"]["Page"]["media"].as_array().unwrap() {

            let id = anime["id"].as_i64().unwrap() as i32;
            let anime_entry: AnimeInfo = serde_json::from_value(anime.clone()).unwrap();
            list.push(anime_entry.clone());
            anime_data.insert(id, anime_entry);
        }

        // limit number of pages for a timely response
        if page >= constants::BROWSE_PAGE_LIMIT {
            break;
        }

        // check if a next page exists
        if let Some(response_next_page) = response["data"]["Page"]["pageInfo"]["hasNextPage"].as_bool() {
            has_next_page = response_next_page;
        } else {
            has_next_page = false;
        }
    }

    file_operations::write_file_anime_info_cache(&anime_data);
    
    list
}



// add anime to the users list (not for moving anime)
#[tauri::command]
async fn add_to_list(id: i32, list: String) {

    let mut user_anime = UserAnimeInfo::default();
    user_anime.media_id = id;
    user_anime.status = list;

    update_user_entry(user_anime).await;
    file_name_recognition::parse_file_names(Some(id)).await;
}



// removes anime from the users list
#[tauri::command]
async fn remove_anime(id: i32, media_id: i32) -> bool {

    // remove from users anilist
    let removed = api_calls::anilist_remove_entry(id, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    if removed == true {

        let status = GLOBAL_USER_ANIME_DATA.lock().await.get(&media_id).unwrap().status.clone();
        let list = if status == "REPEATING" {
            String::from("CURRENT")
        } else {
            status
        };
        // remove anime id from users list in gekijou
        GLOBAL_USER_ANIME_LISTS.lock().await.entry(list).and_modify(|list| { list.retain(|id| *id != media_id)});
        //GLOBAL_USER_ANIME_DATA.lock().await.remove(&media_id);
    }
    file_operations::write_file_user_info().await;
    removed
}



// sets the highlight color
#[tauri::command]
async fn set_highlight(color: String) {
    GLOBAL_USER_SETTINGS.lock().await.highlight_color = color;
}



// returns highlight color from user settings
#[tauri::command]
async fn get_highlight() -> String {
    GLOBAL_USER_SETTINGS.lock().await.highlight_color.clone()
}



// close the splashscreen and show main window
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {

  // Close splashscreen
  if let Some(splashscreen) = window.get_window("splashscreen") {
    match splashscreen.close() {
        Ok(v) => v,
        Err(e) => println!("Can't close splashscreen window, {e:?}"),
    }
  } else {
    println!("Can't find splashscreen window")
  }

  // Show main window
  if let Some(splashscreen) = window.get_window("main") {
    match splashscreen.show() {
        Ok(v) => v,
        Err(e) => println!("Can't show main window, {e:?}"),
    }
  } else {
    println!("Can't find main window")
  }
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

    let ids = recommendation::recommendations(mode, genre_filter, year_min_filter, year_max_filter, format_filter).await;
    let mut anime: Vec<AnimeInfo> = Vec::new();
    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    for id in ids {
        if anime_data.contains_key(&id) == false {
            println!("missing: {}", id);
            continue;
        }
        anime.push(anime_data.get(&id).unwrap().clone());
    }
    anime
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

    let user_lists = GLOBAL_USER_ANIME_LISTS.lock().await;

    if user_lists.contains_key(&list) {
        
        let mut ids = user_lists.get(&list).unwrap().clone();
        if GLOBAL_USER_SETTINGS.lock().await.show_adult == false {
            let anime_data = GLOBAL_ANIME_DATA.lock().await;
            ids.retain(|id| { anime_data.get(id).unwrap().is_adult == false });
        }
        return Some(ids);
    }

    None
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

    GLOBAL_TOKEN.lock().await.clear();
    GLOBAL_ANIME_DATA.lock().await.clear();
    GLOBAL_USER_ANIME_DATA.lock().await.clear();
    GLOBAL_USER_ANIME_LISTS.lock().await.clear();
    GLOBAL_USER_SETTINGS.lock().await.clear();
    GLOBAL_ANIME_PATH.lock().await.clear();
    GLOBAL_REFRESH_UI.lock().await.clear();
    GLOBAL_UPDATE_ANIME_DELAYED.lock().await.clear();

    file_operations::delete_data()
}



// returns if startup tasks are finished.  Data will be missing if startup is not completed
#[tauri::command]
async fn startup_finished() -> bool {
    return *GLOBAL_STARTUP_FINISHED.lock().await;
}



// gets user data for every anime and list in a users account and stores it in global data
async fn get_user_data() {

    let username = GLOBAL_USER_SETTINGS.lock().await.username.clone();
    if username.is_empty() {
        return; // no user exists
    }

    // get user data from anilist as json
    let response = api_calls::anilist_list_query_call(username, GLOBAL_TOKEN.lock().await.access_token.clone()).await;
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();

    if let Some(lists) = json["data"]["MediaListCollection"]["lists"].as_array() {

        for item in lists {

            let name: String = match serde_json::from_value(item["status"].clone()) {
                Err(e) => panic!("list status does not exist {}", e),
                Ok(e) => 
                    if e == "REPEATING" { 
                        String::from("CURRENT") 
                    } else { 
                        e 
                    },
            };
    
            let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await;
            
            if GLOBAL_USER_ANIME_LISTS.lock().await.contains_key(&name) == false {
                GLOBAL_USER_ANIME_LISTS.lock().await.insert(name.clone(), Vec::new());
            }
    
            GLOBAL_USER_ANIME_LISTS.lock().await.entry(name.clone()).and_modify(|list| {
                
                for item2 in item["entries"].as_array().unwrap() {
    
                    let entry: UserAnimeInfo = serde_json::from_value(item2.clone()).unwrap();
    
                    if list.contains(&entry.media_id) == false {
    
                        list.push(entry.media_id.clone());
                    }
                    user_data.insert(entry.media_id, entry);
                }
            });
        }
    }

}



// initialize and run Gekijou
fn main() {
    tauri::Builder::default()
    .setup(|app| {
        let splashscreen_window = app.get_window("splashscreen").unwrap();
        let main_window = app.get_window("main").unwrap();
        
        tauri::async_runtime::spawn(async move {

            load_user_settings().await;

            if GLOBAL_USER_SETTINGS.lock().await.first_time_setup == false {
                on_startup().await;
            }

            splashscreen_window.close().unwrap();
            main_window.show().unwrap();
        });
        Ok(())
      })
        .invoke_handler(tauri::generate_handler![get_anime_info_query,set_highlight,get_highlight,anilist_oauth_token,write_token_data,set_user_settings,
            get_user_settings,get_list_user_info,get_anime_info,get_user_info,update_user_entry,get_list,on_startup,load_user_settings,scan_anime_folder,
            play_next_episode,anime_update_delay,refresh_ui,increment_decrement_episode,on_shutdown,episodes_exist,browse,
            add_to_list,remove_anime,episodes_exist_single,get_delay_info,get_list_paged,set_current_tab,close_splashscreen,get_torrents,recommend_anime,
            open_url,get_list_ids,run_filename_tests,get_debug,delete_data,background_tasks,startup_finished])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}