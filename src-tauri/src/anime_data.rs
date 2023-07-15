use std::{cmp::Ordering, collections::{HashMap, HashSet}};

use serde::{Serialize, Deserialize};

use crate::{api_calls, user_data::UserInfo};



#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimeInfo {
    pub average_score: Option<i32>,
    pub cover_image: CoverImage,
    pub description: Option<String>,
    pub duration: Option<i32>,
    pub episodes: Option<i32>,
    pub format: Option<String>,
    pub genres: Vec<String>,
    pub id: i32,
    pub is_adult: bool,
    pub popularity: i32,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub start_date: AnilistDate,
    pub title: Title,
    pub trailer: Option<TrailerData>,
    pub media_type: String, // type is a rust keyword
    pub relations: Relations,
    pub recommendations: Option<Recommendations>,
    pub tags: Vec<Tag>,
    pub trending: i32,
    pub studios: Studio,
    pub next_airing_episode: Option<NextAiringEpisode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CoverImage {
    pub large: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnilistDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>
}

impl Ord for AnilistDate {
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

impl PartialOrd for AnilistDate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AnilistDate {
    fn eq(&self, other: &Self) -> bool {
        (self.year, &self.month, &self.day) == (other.year, &other.month, &other.day)
    }
}

impl Eq for AnilistDate { }

impl AnilistDate {
    pub const fn new() -> AnilistDate {
        AnilistDate { year: None, month: None, day: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
    pub custom: Option<String>
}

// the structs below replicate the structure of data being returned by anilist api calls
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailerData {
    pub id: String,
    pub site: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Relations {
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub relation_type: String,
    pub node: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recommendations {
    pub nodes: Vec<RecNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecNode {
    pub rating: i32,
    pub media_recommendation: Option<MediaRecommendation>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  MediaRecommendation {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Tag {
    pub name: String,
    pub is_general_spoiler: bool,
    pub is_media_spoiler: bool,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Studio {
    pub nodes: Vec<NodeName>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeName {
    pub name: String,
    pub is_animation_studio: bool
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NextAiringEpisode {
    pub airing_at: i32,
    pub episode: i32,
}

pub struct AnimeData {
    pub data : HashMap<i32, AnimeInfo>,
    pub nonexistent_ids: HashSet<i32>,
    pub needs_scan: Vec<i32>,
}

impl AnimeData {

    pub fn clear(&mut self) {
        self.data.clear();
        self.nonexistent_ids.clear();
    }

    pub async fn get_anime_data(&mut self, media_id: i32) -> Result<AnimeInfo, &'static str> {

        if self.nonexistent_ids.contains(&media_id) {
            return Err("Anime does not exist");
        }

        if let Some(anime) = self.data.get(&media_id) {
            return Ok(anime.clone());
        } else {
            match api_calls::anilist_get_anime_info_single2(media_id).await {
                Ok(result) => {
                    self.data.insert(result.id, result.clone());
                    return Ok(result);
                },
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn get_anime_list_data(&mut self, id_list: Vec<i32>) -> Result<Vec<AnimeInfo>, &'static str> {
        
        let mut valid_ids: Vec<i32> = id_list.iter().map(|id| *id).filter(|id| self.nonexistent_ids.contains(id) == false).collect();

        let missing_ids: Vec<i32> = valid_ids.iter().map(|id| *id).filter(|id| self.data.contains_key(id) == false).collect();

        if missing_ids.is_empty() == false {
            match api_calls::anilist_api_call_multiple2(missing_ids.clone()).await {
                Ok(result) => {
                    let missing_from_anilist_ids = self.find_missing_ids(&missing_ids, &result);
                    for anime in result {
                        self.data.insert(anime.id, anime);
                    }
                    for id in missing_from_anilist_ids {
                        valid_ids.remove(valid_ids.iter().position(|v_id| *v_id == id).unwrap());
                    }
                },
                Err(error) => return Err(error),
            }
        }

        let mut list_anime: Vec<AnimeInfo> = Vec::new();
        for id in valid_ids {
            if let Some(anime) = self.data.get(&id) {
                list_anime.push(anime.clone());
            } else {
                println!("anime is missing");
            }
        }

        Ok(list_anime)
    }

    // find anime missing from anilist and add them to the nonexistent list
    fn find_missing_ids(&mut self, ids: &Vec<i32>, data: &Vec<AnimeInfo>) -> Vec<i32> {

        let mut missing_ids: Vec<i32> = Vec::new();
        for id in ids {
            let mut found = false;
            for anime in data {
                if anime.id == *id {
                    found = true;
                    break;
                }
            }
            if found == false {
                self.nonexistent_ids.insert(*id);
                missing_ids.push(*id);
            }
        }

        missing_ids
    }

    pub fn sort_list(&self, anime_list: &mut Vec<(AnimeInfo,UserInfo)>, sorting: Option<String>) {
        if let Some(sort_category) = sorting {
            match sort_category.as_str() {
                "Alphabetical_native" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.native.clone() {
                        if let Some(second_title) = second.0.title.native.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Alphabetical_romaji" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.romaji.clone() {
                        if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Alphabetical_english" => anime_list.sort_by(|first, second| {
                    if let Some(first_title) = first.0.title.english.clone() {
                        if let Some(second_title) = second.0.title.english.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else if let Some(first_title) = first.0.title.romaji.clone() {
                        if let Some(second_title) = second.0.title.english.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else if let Some(second_title) = second.0.title.romaji.clone() {
                            first_title.to_lowercase().partial_cmp(&second_title.to_lowercase()).unwrap()
                        } else {
                            std::cmp::Ordering::Less
                        }
                    } else {
                        std::cmp::Ordering::Greater
                    }
                }),
                "Score" => anime_list.sort_by(|first, second| {
                    first.0.average_score.partial_cmp(&second.0.average_score).unwrap()
                }),
                "MyScore" => anime_list.sort_by(|first, second| {
                    first.1.score.partial_cmp(&second.1.score).unwrap()
                }),
                "Date" => anime_list.sort_by(|first, second| {
                    first.0.start_date.partial_cmp(&second.0.start_date).unwrap()
                }),
                "Popularity" => anime_list.sort_by(|first, second| {
                    first.0.popularity.partial_cmp(&second.0.popularity).unwrap()
                }),
                "Trending" => anime_list.sort_by(|first, second| {
                    first.0.trending.partial_cmp(&second.0.trending).unwrap()
                }),
                "Started" => anime_list.sort_by(|first, second| {
                    first.1.started_at.partial_cmp(&second.1.started_at).unwrap()
                }),
                "Completed" => anime_list.sort_by(|first, second| {
                    first.1.completed_at.partial_cmp(&second.1.completed_at).unwrap()
                }),
                &_ => (),
            }
        } else {

        }
    }

    pub fn identify_anime(filename: String) -> Option<(i32, i32)> {
        
    }

}