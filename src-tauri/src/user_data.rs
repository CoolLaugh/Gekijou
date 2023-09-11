use std::{collections::{HashMap, HashSet}, cmp::Ordering};

use chrono::{DateTime, Local, Datelike};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;

use crate::{constants::{USER_STATUSES, USER_LISTS, self}, GLOBAL_REFRESH_UI, api_calls, file_operations};


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserInfo {
    pub id: i32,
    pub media_id: i32,
    pub status: String,
    pub score: f32,
    pub progress: i32,
    pub started_at: Option<Date>,
    pub completed_at: Option<Date>,
    pub notes: Option<String>,
    pub updated_at: u64,
}

impl UserInfo {
    pub const fn new() -> UserInfo {
        UserInfo { id: 0, media_id: 0, status: String::new(), score: 0.0, progress: 0, started_at: None, completed_at: None, notes: None, updated_at: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Date {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> Ordering {

        if self.year.is_none() && other.year.is_some() {  return Ordering::Less }
        if self.year.is_some() && other.year.is_none() { return Ordering::Greater }
        if self.year.is_some() && other.year.is_some() {
            if self.year.unwrap() < other.year.unwrap() { return Ordering::Less }
            if self.year.unwrap() > other.year.unwrap()  { return Ordering::Greater }
        }

        if self.month.is_none() && other.month.is_some() { return Ordering::Less }
        if self.month.is_some() && self.month.is_none() { return Ordering::Greater } 
        if self.month.is_some() && other.month.is_some() {
            if self.month.unwrap() < other.month.unwrap() { return Ordering::Less } 
            if self.month.unwrap() > other.month.unwrap()  { return Ordering::Greater }
        }

        if self.day.is_none() && other.day.is_some() { return Ordering::Less } 
        if self.day.is_some() && self.day.is_none() { return Ordering::Greater } 
        if self.day.is_some() && other.day.is_some() {
            if self.day.unwrap() < other.day.unwrap() { return Ordering::Less } 
            if self.day.unwrap() > other.day.unwrap()  { return Ordering::Greater }
        }

        Ordering::Equal
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        (self.year, &self.month, &self.day) == (other.year, &other.month, &other.day)
    }
}

impl Eq for Date { }

impl Date {
    pub const fn new() -> Date {
        Date { year: None, month: None, day: None }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData2 {
    pub token_type: String,
    pub expires_in: i32,
    pub access_token: String,
    pub refresh_token: String
}

impl TokenData2 {
    pub const fn new() -> TokenData2 {
        TokenData2 { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() }
    }

    pub fn clear(&mut self) {
        self.token_type.clear();
        self.expires_in = 0;
        self.access_token.clear();
        self.refresh_token.clear();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettings {
    pub username: String,
    pub title_language: String,
    pub show_adult: bool,
    pub folders: Vec<String>,
    pub update_delay: i32,
    pub score_format: Option<String>,
    pub highlight_color: String,
    pub current_tab: String,
    pub first_time_setup: bool,
    pub show_airing_time: Option<bool>,
    pub theme: Option<i32>,
    pub user_id: Option<i32>,
}

impl UserSettings {
    pub const fn new() -> UserSettings {
        UserSettings { username: String::new(), title_language: String::new(), show_adult: false, folders: Vec::new(), update_delay: 0, score_format: None, highlight_color: String::new(), current_tab: String::new(), first_time_setup: true, show_airing_time: Some(true), theme: Some(0), user_id: None }
    }
    
    pub fn clear(&mut self) {
        self.username.clear();
        self.title_language.clear();
        self.show_adult = false;
        self.folders.clear();
        self.update_delay = 0;
        self.score_format = None;
        self.highlight_color.clear();
        self.current_tab.clear();
        self.first_time_setup = true;
        self.show_airing_time = Some(true);
        self.theme = Some(0);
        self.user_id = None;
    }
}

pub struct UserDataMutex(Mutex<UserData>);
pub struct UserData {
    setting: UserSettings,
    token: TokenData2,
    user_data: HashMap<i32, UserInfo>,
    user_lists: HashMap<String, Vec<i32>>,
    max_episodes: HashMap<i32, Option<i32>>,
    update_queue: Vec<UserInfo>,
}



impl UserData {

    pub fn new() -> UserData {
        UserData { setting: UserSettings::new(), token: TokenData2::new(), user_data: HashMap::new(), user_lists: HashMap::new(), max_episodes: HashMap::new(), update_queue: Vec::new() }
    }
    
    pub fn clear(&mut self) {
        self.user_data.clear();
        self.user_lists.clear();
    }

    pub async fn read_files(&mut self) {

        file_operations::read_file_user_settings(&mut self.setting).await;
        file_operations::read_file_token_data(&mut self.token).await;
        file_operations::read_file_user_data(&mut self.user_data).await;
        file_operations::read_file_user_lists(&mut self.user_lists).await;
        file_operations::read_file_update_queue(&mut self.update_queue).await;
    }

    pub fn get_user_data(&self, media_id: i32) -> Result<UserInfo, &'static str> {
        if let Some(data) = self.user_data.get(&media_id) {
            Ok(data.clone())
        } else {
            Err("data does not exist")
        }
    }

    async fn check_correct_list(&mut self, data: &UserInfo) {
        
        let old_status = if let Some(old_data) = self.user_data.get(&data.media_id) {
            old_data.status.clone()
        } else {
            String::new()
        };

        let mut change_list = false;
        // check if status has changed
        if old_status != data.status {
            change_list = true;
            // current and repeating are in the same list
            if (old_status == "CURRENT" && data.status == "REPEATING") ||
                (old_status == "REPEATING" && data.status == "CURRENT") {

                change_list = false;
            }
        }

        // status has changed, move to new list
        if change_list == true {
            
            self.user_lists.iter_mut().for_each(|(_, list)| list.retain(|id| *id != data.media_id));
            let list = if data.status == "REPEATING" {
                String::from("CURRENT")
            } else {
                data.status.clone()
            };
            self.user_lists.entry(list).and_modify(|entry| entry.push(data.media_id));
            file_operations::write_file_user_lists(&self.user_lists).await;
        }
    }

    pub async fn set_user_data(&mut self, mut data: UserInfo, update_website: bool) -> Result<Option<UserInfo>, &'static str> {
        
        println!("set_user_data: {:?}", data);

        if USER_STATUSES.contains(&data.status.as_str()) == false {
            return Err("invalid status");
        }

        let old_status = if let Some(old_data) = self.user_data.get(&data.media_id) {
            old_data.status.clone()
        } else {
            String::new()
        };

        let old_progress = if let Some(old_data) = self.user_data.get(&data.media_id) {
            old_data.progress
        } else {
            -1
        };

        self.check_correct_list(&data).await;

        if data.status == "COMPLETED" {
            if let Some(max_episodes) = self.max_episodes.get(&data.media_id) {
                if let Some(episodes) = max_episodes {
                    data.progress = *episodes;
                    println!("progress {}", data.progress);
                }
            }
        }

        // update completed and started date if show is completed and don't change original start and end date if rewatching
        if data.status == "COMPLETED" && old_status != "REPEATING" {
            let mut set_completed = false;
            // user didn't input a date
            if data.completed_at.is_none() {
                set_completed = true;
            } else if let Some(completed_at) = data.completed_at.clone() {
                if completed_at.day.is_none() && completed_at.month.is_none() && completed_at.year.is_none() {
                    set_completed = true;
                }
            }

            // set completed date to today
            if set_completed { 
                let now: DateTime<Local> = Local::now();
                data.completed_at = Some(Date {
                    year: Some(now.year()),
                    month: Some(now.month() as i32),
                    day: Some(now.day() as i32),
                });
            }

            // set start date if the entire entry was watched at once
            if let Some(started_at) = &data.started_at {
                if started_at.day.is_none() && started_at.month.is_none() && started_at.year.is_none() { // user didn't input a date
                    // set if anime is a movie or special so the user will watch it in one sitting
                    if let Some(episodes) = self.max_episodes.get(&data.media_id) {
                        if episodes.is_some() && episodes.unwrap() <= 1 { // anime is a movie or special
                            data.started_at = data.completed_at.clone();
                        }
                    }
                    // set if user watched the whole series at once
                    if old_progress == 0 {
                        data.started_at = data.completed_at.clone();
                    }
                }
            } else {
                println!("ERROR: started_at is None"); // this function should always be called with started_at existing
            }
        }
        println!("data {:?} old progress {}", data, old_progress);
        if data.status == constants::USER_STATUSES[0] /* current */ &&  data.progress > 0 && old_progress == 0 {
            if let Some(started_at) = &data.started_at {
                if started_at.day.is_none() && started_at.month.is_none() && started_at.year.is_none() { // user didn't input a date
                    let now: DateTime<Local> = Local::now();
                    data.started_at = Some(Date {
                        year: Some(now.year()),
                        month: Some(now.month() as i32),
                        day: Some(now.day() as i32),
                    });
                }
            }
        }

        // update anilist
        if update_website == true {
            api_calls::update_user_entry(self.token.access_token.clone(), data.clone()).await;
        }

        // set the time when user data was updated to prevent being overwritten by anilist
        data.updated_at = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

        // save new data
        let old_data = self.user_data.insert(data.media_id, data.clone());

        // write to file for the next time gekijou is opened
        file_operations::write_file_user_data(&self.user_data).await;

        Ok(old_data)
    }

    pub async fn remove_anime(&mut self, media_id: i32) ->Result<bool, &'static str> {

        if let Some(anime) = self.user_data.get(&media_id){

            // remove from users anilist
            match api_calls::anilist_remove_entry(anime.id, self.token.access_token.clone()).await {
                Ok(removed) => {
                    if removed == true {
                
                        self.user_lists.iter_mut().for_each(|(_,list)| list.retain(|list_id| *list_id != media_id));
                        self.user_data.remove(&media_id);

                        file_operations::write_file_user_data(&mut self.user_data).await;
                        file_operations::write_file_user_lists(&mut self.user_lists).await;
                    }
                    return Ok(removed)
                },
                Err(error) => return Err(error),
            }
        } else {
            return Err("media id does not exist");
        }
    }

    pub async fn get_list(&mut self, name: &String) -> Result<Vec<i32>, &'static str> {
        
        if USER_LISTS.contains(&name.as_str()) == false {
            return Err("invalid list");
        }

        if self.user_lists.contains_key(name) == false {
            // list is missing, get it
            api_calls::anilist_get_list(self.setting.username.clone(), String::from(name), self.token.access_token.clone(), &mut self.user_data, &mut self.user_lists).await;
            file_operations::write_file_user_lists(&self.user_lists).await;
            file_operations::write_file_user_data(&self.user_data).await;
        }

        let list = self.user_lists.get(name).unwrap().clone();

        Ok(list)
    }

    pub async fn get_list_with_data(&mut self, name: &String) -> Result<Vec<UserInfo>, &'static str> {
        
        if USER_LISTS.contains(&name.as_str()) == false {
            return Err("invalid list");
        }

        if self.user_lists.contains_key(name) == false {
            // list is missing, get it
        }

        let list = self.user_lists.get(name).unwrap().clone();
        let mut data_list: Vec<UserInfo> = Vec::new();
        for id in list {
            if let Some(data) = self.user_data.get(&id) {
                data_list.push(data.to_owned());
            } else {
                println!("User data does not exist");
            }
        }

        Ok(data_list)
    }

    pub async fn get_data(&self, list: &Vec<i32>) -> Result<Vec<UserInfo>, &'static str> {

        let mut info_list: Vec<UserInfo> = Vec::new();
        for id in list {

            if let Some(data) = self.user_data.get(id) {
                info_list.push(data.clone());
            }
        }

        Ok(info_list)
    }

    pub async fn pull_updates(&mut self) -> Result<bool, &'static str> {
        
        if self.setting.username.is_empty() {
            return Err("No username");
        }

        match api_calls::get_updated_media_ids(self.setting.username.clone(), 0).await {
            Ok(list) => {
                GLOBAL_REFRESH_UI.lock().await.no_internet = false;
                
                for entry in list {
                    if self.user_data.contains_key(&entry.media_id) {
                        let client_updated_at = self.user_data.get(&entry.media_id).unwrap().updated_at;
                        if entry.updated_at > client_updated_at {
                            self.set_user_data(entry, false).await;
                        }
                    } else {
                        self.set_user_data(entry, false).await;
                    }
                }
            },
            Err(_error) => GLOBAL_REFRESH_UI.lock().await.no_internet = true,
        }
    
        Ok(true)
    }

    pub async fn increment_episode(&mut self, media_id: i32, length: i32) -> Result<bool, &'static str> {
        
        if let Some(mut media) = self.user_data.get(&media_id).cloned() {

            if media.progress == 0 && media.status == constants::USER_STATUSES[4] /* Planning */ {
                media.status = constants::USER_STATUSES[0].to_string(); // current
                GLOBAL_REFRESH_UI.lock().await.anime_list = true;
            }

            media.progress += length;

            if let Some(episodes) = self.max_episodes.get(&media_id).cloned() {
                if let Some(episodes_unwrapped) = episodes {

                    if media.progress >= episodes_unwrapped {
                        media.status = constants::USER_STATUSES[1].to_string(); // completed
                        GLOBAL_REFRESH_UI.lock().await.anime_list = true;
                    }
                }
            }

            match self.set_user_data(media, true).await {
                Ok(_result) => {},
                Err(error) => println!("{}", error),
            }

        } else {
            println!("increment_episode: userinfo missing");
        }

        Ok(true)
    }

    // takes a oauth code from the user and exchanges it for a oauth access token
    pub async fn anilist_oauth_token(&mut self, code: String) -> (bool, String) {
        
        let token = api_calls::anilist_get_access_token2(code).await;
        let combine = format!("{}\n{}", token.token_type, token.access_token);

        if token.access_token.len() == 0 {
            return (false, combine);
        }
        else {
            self.token = token;
        }

        file_operations::write_file_token_data(&self.token).await;
        
        (true, String::new())
    }

    pub fn get_user_settings(&self) -> UserSettings {
        return self.setting.clone();
    }

    pub async fn set_user_settings(&mut self, new_user_settings: UserSettings) -> (bool, Option<Vec<i32>>) {
        
        // check if the folders have changed
        let mut scan = false;
        if self.setting.folders.len() != new_user_settings.folders.len() {
            scan = true;
        } else {
            for i in 0..new_user_settings.folders.len() {
                if new_user_settings.folders[i] == "" {
                    continue;
                }
                if self.setting.folders[i] != new_user_settings.folders[i] {
                    scan = true;
                }
            }
        }

        if self.setting.show_adult != new_user_settings.show_adult ||
            self.setting.title_language != new_user_settings.title_language {
            GLOBAL_REFRESH_UI.lock().await.anime_list = true;
        }

        let old_current_tab = self.setting.current_tab.clone(); // don't change this value
        let score_format = self.setting.score_format.clone(); // don't change this value
        let old_username = self.setting.username.clone();
        self.setting = new_user_settings;
        self.setting.score_format = score_format;
        self.setting.current_tab = old_current_tab;

        // user is different, their list and score format will be different
        if old_username != self.setting.username {

            self.user_data.clear();
            self.user_lists.clear();

            match api_calls::get_user_score_format(self.setting.username.clone()).await {
                Ok(result) => {
                    self.setting.score_format = Some(result);
                },
                Err(_error) => self.setting.score_format = None,
            }
            
            let lists = vec!["CURRENT","COMPLETED","PAUSED","DROPPED","PLANNING"];
            let mut list_count = 0;
            for list in lists {
                list_count += 1;
                GLOBAL_REFRESH_UI.lock().await.loading_dialog = Some(format!("Downloading User Lists ({} of 5)", list_count));
                api_calls::anilist_get_list(self.setting.username.clone(), String::from(list), self.token.access_token.clone(), &mut self.user_data, &mut self.user_lists).await;
            }
            file_operations::write_file_user_data(&self.user_data).await;
            file_operations::write_file_user_lists(&self.user_lists).await;

            let user_media_ids: Vec<i32> = {
                let mut media_ids: Vec<i32> = Vec::new();
                self.user_lists.iter().for_each(|(_,list)|  media_ids.extend(list));
                media_ids
            };

            file_operations::write_file_user_settings(&self.setting).await;
            return (scan, Some(user_media_ids));
        } else {
            
            file_operations::write_file_user_settings(&self.setting).await;
            return (scan, None);
        }

    }

    pub fn set_current_tab(&mut self, tab: String) {
        self.setting.current_tab = tab;
        //file_operations::write_file_user_settings
    }

    pub fn get_update_delay(&self) -> i32 {

        self.setting.update_delay
    }

    pub fn set_highlight(&mut self, color: String) {
        self.setting.highlight_color = color;
    }

    pub fn get_highlight(&self) -> String {
        self.setting.highlight_color.clone()
    }

    pub fn get_scores_from_list(&self, list: String) -> HashMap<i32, f32> {

        if let Some(list_of_ids) = self.user_lists.get(&list) {

            let mut scores: HashMap<i32, f32> = HashMap::new();
            for id in list_of_ids {

                if let Some(info) = self.user_data.get(id) {
                    scores.insert(*id, info.score);
                }
            }
            return scores;
        } else {
            return HashMap::new();
        }
    }

    pub fn all_ids(&self) -> HashSet<i32> {

        let mut set: HashSet<i32> = HashSet::new();
        
        for id in self.user_data.keys() {
            set.insert(*id);
        }
        set
    }

    pub fn set_max_episodes(&mut self, max_episodes: HashMap<i32, Option<i32>>) {
        self.max_episodes = max_episodes;
    }

}