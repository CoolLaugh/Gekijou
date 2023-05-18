use std::{cmp::{Ordering, max}, collections::HashMap};


use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};


use crate::{secrets, GLOBAL_ANIME_DATA, file_operations};


// the structs below replicate the structure of data being returned by anilist api calls
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailerData {
    pub id: String,
    pub site: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>,
    pub user_preferred: Option<String>,
    pub custom: Option<String>
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
    pub media_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub relation_type: String,
    pub node: Node,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Relations {
    pub edges: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct  MediaRecommendation {
    pub id: i32,
    pub title: Title,
    pub cover_image: CoverImage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecNode {
    pub rating: i32,
    pub media_recommendation: Option<MediaRecommendation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Recommendations {
    pub nodes: Vec<RecNode>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Tag {
    pub name: String,
    pub is_general_spoiler: bool,
    pub is_media_spoiler: bool,
    pub description: String,
}

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
pub struct NextAiringEpisode {
    pub airing_at: i32,
    pub episode: i32,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub media: AnimeInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: Media
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub token_type: String,
    pub expires_in: i32,
    pub access_token: String,
    pub refresh_token: String
}

impl TokenData {
    pub const fn new() -> TokenData {
        TokenData { token_type: String::new(), expires_in: 0, access_token: String::new(), refresh_token: String::new() }
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
    pub show_spoilers: bool,
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
    pub updated_at: Option<u64>,
}

impl UserSettings {
    pub const fn new() -> UserSettings {
        UserSettings { username: String::new(), title_language: String::new(), show_spoilers: false, show_adult: false, folders: Vec::new(), update_delay: 0, score_format: None, highlight_color: String::new(), current_tab: String::new(), first_time_setup: true, show_airing_time: Some(true), theme: Some(0), user_id: None, updated_at: None }
    }
    
    pub fn clear(&mut self) {
        self.username.clear();
        self.title_language.clear();
        self.show_spoilers = false;
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
        self.updated_at = None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserAnimeInfo {
    pub id: i32,
    pub media_id: i32,
    pub status: String,
    pub score: f32,
    pub progress: i32,
    pub started_at: Option<AnilistDate>,
    pub completed_at: Option<AnilistDate>,
    pub notes: Option<String>,
}

impl UserAnimeInfo {
    pub const fn new() -> UserAnimeInfo {
        UserAnimeInfo { id: 0, media_id: 0, status: String::new(), score: 0.0, progress: 0, started_at: None, completed_at: None, notes: None }
    }
}

// request json for anilist api
const ANIME_INFO_QUERY: &str = "
query ($id: Int) {
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id title { english } coverImage { large } season seasonYear type format episodes duration isAdult genres averageScore popularity description trailer { id site } startDate { year month day } trending
    }
}";

// get every list from a user with all user data for each anime
const ANIME_LIST_QUERY: &str = "
query ($username: String) {
    MediaListCollection (userName: $username, type: ANIME) {
        lists {
            name entries { id mediaId status score progress startedAt { year month day } completedAt { year month day } } status
        }
    }
}";

// query for absolutely all data of a specific anime
/*
const ANIME_ALL_INFO_QUERY: &str = "
query ($id: Int) {
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id idMal title { romaji english native userPreferred } type format status description startDate { year month day } endDate { year month day }
        season seasonYear seasonInt episodes duration chapters volumes countryOfOrigin isLicensed source hashtag trailer { id site } updatedAt coverImage { large } bannerImage
		genres synonyms averageScore meanScore popularity isLocked trending favourites isFavourite isFavouriteBlocked isAdult
		tags { id name description category rank isGeneralSpoiler isMediaSpoiler isAdult userId }
        relations { edges { relationType node { id title { romaji english native userPreferred } coverImage { large } type } } nodes { id title { english romaji } } }
		nextAiringEpisode { id airingAt timeUntilAiring episode mediaId media { id } }
		externalLinks { id url site siteId type language color icon notes isDisabled }
		streamingEpisodes { title thumbnail url site }
		rankings { id rank type format year season allTime context }
		recommendations { nodes { id rating media { id title { romaji english native userPreferred } } mediaRecommendation { title { romaji english native userPreferred } } user { id name } } }
		stats { scoreDistribution { score amount } statusDistribution { status amount } }
		siteUrl autoCreateForumThread isRecommendationBlocked isReviewBlocked modNotes
    }
}";
*/

// query to return all data based on a criteria of year, season, format, and/or genre
const ANIME_BROWSE: &str = "
query($page: Int $type: MediaType $format: [MediaFormat] $season: MediaSeason $seasonYear: Int $genres: [String] $tags: [String] $search: String $sort: [MediaSort] = [POPULARITY_DESC, SCORE_DESC]) {
    Page(page: $page, perPage: 50) {
        pageInfo { total perPage currentPage lastPage hasNextPage }
        media(type: $type season: $season format_in: $format seasonYear: $seasonYear genre_in: $genres tag_in: $tags search: $search sort: $sort) {
            id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
            duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
            relations { edges { relationType node { id title { romaji english native userPreferred } coverImage { large } type } } }
            recommendations { nodes { rating mediaRecommendation { id title { romaji english native userPreferred } coverImage { large } } } }
            tags { name isGeneralSpoiler isMediaSpoiler description }
            studios(isMain: true) { nodes { name isAnimationStudio } }
            nextAiringEpisode { airingAt, episode }
        }
    }
}";

// retrieve a list of anime based on criteria like the year/season it was released, format, or genre
pub async fn anilist_browse_call(page: i32, year: String, season: String, genre: String, format: String, search: String, order: String) -> Result<serde_json::Value, &'static str> {

    let mut variables = json!({"page": page, "type": "ANIME"});
    if year.is_empty() == false {
        variables["seasonYear"] = Value::from(year);
    }
    if season.is_empty() == false {
        variables["season"] = Value::from(season);
    }
    if genre.is_empty() == false {
        variables["genres"] = Value::from(genre);
    }
    if format.is_empty() == false {
        variables["format"] = Value::from(format);
    }
    if order.is_empty() == false {
        variables["sort"] = Value::from(order);
    }
    if search.is_empty() == false {
        variables["search"] = Value::from(search);
    }

    let json = json!({"query": ANIME_BROWSE, "variables": variables});
    
    match post(&json, None).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);
            return Ok(serde_json::from_str(&response).unwrap());
        },
        Err(error) => return Err(error),
    }
}



const ANIME_DELETE_ENTRY: &str = "
mutation ($id: Int) { 
    DeleteMediaListEntry (id: $id) {
        deleted
    }
}";

// remove a anime from the users anime list
pub async fn anilist_remove_entry(id: i32, access_token: String) -> Result<bool, &'static str> {

    let json = json!({"query": ANIME_DELETE_ENTRY, "variables": {"id": id}});

    match post(&json, Some(&access_token)).await {
        Ok(result) => {
            let deleted = serde_json::from_str::<serde_json::Value>(&result).unwrap()["data"]["DeleteMediaListEntry"]["deleted"].as_bool().unwrap();
            return Ok(deleted);
        },
        Err(error) => return Err(error),
    }
}



// retrieve information on anime using it's anilist id
pub async fn anilist_api_call(id: i32) -> Result<AnimeInfo, &'static str> {
    
    // create client and query json
    let json = json!({"query": ANIME_INFO_QUERY, "variables": {"id": id}});

    match post(&json, None).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);
            let json: Data = serde_json::from_str(&response).unwrap();
            return Ok(json.data.media);
        },
        Err(error) => return Err(error),
    }
}



const ANIME_INFO_QUERY_MULTIPLE: &str = "
query($page: Int $ids: [Int]) {
    Page(page: $page, perPage: 50) {
        pageInfo { total perPage currentPage lastPage hasNextPage }
        media(type: ANIME, id_in: $ids) {
          id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
          duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
          relations { edges { relationType node { id title { romaji english native userPreferred } coverImage { large } type } } }
          recommendations { nodes { rating mediaRecommendation { id title { romaji english native userPreferred } coverImage { large } } } }
          tags { name isGeneralSpoiler isMediaSpoiler description }
          studios(isMain: true) { nodes { name isAnimationStudio } }
          nextAiringEpisode { airingAt, episode }
        }
    }
}";

// get anime data from anilist for all ids
pub async fn anilist_api_call_multiple(ids: Vec<i32>, anime_data: &mut HashMap<i32, AnimeInfo>) -> Result<(), &'static str>  {

    let pages = ceiling_div(ids.len(), 50);
    println!("ids {} pages {}", ids.len(), pages);
    
    for i in 0..pages {

        println!("page {}", i);
        let start = i * 50;
        let end = 
        if start + 50 > ids.len() {
            ids.len()
        } else {
            start + 50
        };
        let sub_vec = &ids[start..end];
        let json = json!({"query": ANIME_INFO_QUERY_MULTIPLE, "variables": { "page": 0, "ids": sub_vec}});

        match post(&json, None).await {
            Ok(result) => {
                let response = anilist_to_snake_case(result);
                let mut anime_json: serde_json::Value = serde_json::from_str(&response).unwrap();
                let anime_vec: Vec<AnimeInfo> = serde_json::from_value(anime_json["data"]["Page"]["media"].take()).unwrap();
    
                for anime in anime_vec {
                    anime_data.insert(anime.id, anime);
                }
            },
            Err(error) => return Err(error),
        }
    }

    return Ok(());
}



fn anilist_to_snake_case(anilist_json: String) -> String {

    anilist_json
        .replace("\"Media\"", "\"media\"")
        .replace("averageScore", "average_score")
        .replace("coverImage", "cover_image")
        .replace("isAdult", "is_adult")
        .replace("seasonYear", "season_year")
        .replace("type", "media_type")
        .replace("startDate", "start_date")
        .replace("userPreferred", "user_preferred")
        .replace("relationType", "relation_type")
        .replace("mediaRecommendation", "media_recommendation")
        .replace("isGeneralSpoiler", "is_general_spoiler")
        .replace("isMediaSpoiler", "is_media_spoiler")
        .replace("nextAiringEpisode", "next_airing_episode")
        .replace("airingAt", "airing_at")
        .replace("mediaId", "media_id")
        .replace("startedAt", "started_at")
        .replace("completedAt", "completed_at")
        .replace("isAnimationStudio", "is_animation_studio")
}

fn ceiling_div(x: usize, y: usize) -> usize {
    max(x / y, (x + y - 1) / y)
}



// query for a specific list along with all user data and media data for the anime on that list
const USER_LIST_WITH_MEDIA: &str = "
query($userName: String, $status: [MediaListStatus]) {
  MediaListCollection(userName: $userName, type:ANIME, status_in:$status) {
    lists {
      name 
      entries {
        id mediaId status score progress updatedAt startedAt { year month day } completedAt { year month day } notes
        media {
          id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
          duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
          relations { edges { relationType node { id title { romaji english native userPreferred } coverImage { large } type } } }
          recommendations { nodes { rating mediaRecommendation { id title { romaji english native userPreferred } coverImage { large } } } }
          tags { name isGeneralSpoiler isMediaSpoiler description }
          studios(isMain: true) { nodes { name isAnimationStudio } }
          nextAiringEpisode { airingAt, episode }
        }
      }
    }
  }
}";

// retrieve information on anime using it's anilist id
// returns a message if a error occurred
pub async fn anilist_get_list(username: String, status: String, access_token: String, anime_user_data: &mut HashMap<i32, UserAnimeInfo>, anime_user_list_lock: &mut HashMap<String, Vec<i32>>) -> Option<String> {

    // create query json
    let status_array = 
    if status == "CURRENT" { // rewatching is included in watching in UI but not in anilist api
        vec![status.clone(), String::from("REPEATING")]
    } else {
        vec![status.clone()]
    };

    let json = json!({"query": USER_LIST_WITH_MEDIA, "variables": {"userName": username, "status": status_array}});

    match post(&json, Some(&access_token)).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);

            let response_json: serde_json::Value = serde_json::from_str::<serde_json::Value>(&response).unwrap();

            if response_json.is_object() && response_json.get("errors").is_some() {
                let message = response_json["errors"][0]["message"].as_str().unwrap().to_string();
                return Some(message)
            }
            let lists: serde_json::Value = serde_json::from_str::<serde_json::Value>(&response).unwrap()["data"]["MediaListCollection"]["lists"].take();
        
            let anime_user_list = anime_user_list_lock.entry(status).or_default();
        
            anime_user_list.clear();
            for list in lists.as_array().unwrap() {
                
                for entry in list["entries"].as_array().unwrap() {
                    
                    let user_info: UserAnimeInfo = UserAnimeInfo { id: entry["id"].as_i64().unwrap() as i32, 
                                                                    media_id: entry["media_id"].as_i64().unwrap() as i32, 
                                                                    status: entry["status"].as_str().unwrap().to_string(), 
                                                                    score: entry["score"].as_f64().unwrap() as f32, 
                                                                    progress: entry["progress"].as_i64().unwrap() as i32, 
                                                                    started_at: serde_json::from_value(entry["started_at"].clone()).unwrap(), 
                                                                    completed_at: serde_json::from_value(entry["completed_at"].clone()).unwrap(),
                                                                    notes: serde_json::from_value(entry["notes"].clone()).unwrap()};
        
                    anime_user_data.insert(user_info.media_id, user_info);
                    let mut media: AnimeInfo = serde_json::from_value(entry["media"].clone()).unwrap();
                    media.studios.nodes.retain(|node| {node.is_animation_studio == true });
        
                    if anime_user_list.contains(&media.id) == false {
        
                        anime_user_list.push(media.id);
                    }
                }
            }
        },
        Err(error) => return Some(String::from(error)),
    }
    
    None
}



// query for a specific list along with all user data and media data for the anime on that list
const MEDIA_INFO: &str = "query ($id: Int) {
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
        duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
        relations { edges { relationType node { id title { romaji english native userPreferred } coverImage { large } type } } }
        recommendations { nodes { rating mediaRecommendation { id title { romaji english native userPreferred } coverImage { large } } } }
        tags { name isGeneralSpoiler isMediaSpoiler description }
        studios(isMain: true) { nodes { name isAnimationStudio } }
        nextAiringEpisode { airingAt, episode }
    }
}";

pub async fn anilist_get_anime_info_single(anime_id: i32) -> Result<(), &'static str> {

    // create client and query json
    let json = json!({"query": MEDIA_INFO, "variables": {"id": anime_id}});

    // get media information from anilist api
    match post(&json, None).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);

            let mut anime_value: serde_json::Value = serde_json::from_str(&response).unwrap();
            let anime_data: AnimeInfo = serde_json::from_value(anime_value["data"]["media"].take()).unwrap();
            GLOBAL_ANIME_DATA.lock().await.insert(anime_data.id, anime_data);
            return Ok(())
        },
        Err(error) => return Err(error),
    }
}



// gets the users anime lists with all user data on each anime
pub async fn anilist_list_query_call(username: String, access_token: String) -> Result<String, &'static str> {

    // create client and query json
    let json = json!({"query": ANIME_LIST_QUERY, "variables": {"username": username}});

    // get media information from anilist api
    match post(&json, Some(&access_token)).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);
            return Ok(response)
        },
        Err(error) => return Err(error),
    }
}



// exchanges a code the user pastes in for a access token that is used to authorize access
pub async fn anilist_get_access_token(code: String) -> TokenData {

    let client = Client::new();

    let json = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": secrets::CLIENT_ID,
        "client_secret": secrets::CLIENT_SECRET,
        "redirect_uri": secrets::REDIRECT_URI,
        "code": code
    });

    let response = client.post("https://anilist.co/api/v2/oauth/token")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&json)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();

    if response_string.contains("\"error\"") {
        return TokenData { token_type: json.to_string(), expires_in: 0, access_token: response_string, refresh_token: String::new() };
    }
    
    return serde_json::from_str(&response_string).unwrap();
}



// query for a airing time for specific ids
const AIRING_INFO: &str = "query($page: Int $ids: [Int]) {
    Page(page: $page, perPage: 50) {
        pageInfo { total perPage currentPage lastPage hasNextPage }
        media(type: ANIME, id_in: $ids) {
          id nextAiringEpisode { airingAt, episode }
        }
    }
}";
// get data for the next airing episode for given ids
pub async fn anilist_airing_time(anime_ids: Vec<i32>, anime_data: &mut HashMap<i32, AnimeInfo>) -> Result<(), &'static str> {
    
    if anime_ids.is_empty() {
        return Ok(());
    }

    // create query json
    let json = json!({"query": AIRING_INFO, "variables": {"ids": anime_ids}});
    // get airing data from anilist
    match post(&json, None).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);
            // get list of media
            let airing_times: serde_json::Value = serde_json::from_str(&response).unwrap();
            let media = airing_times["data"]["Page"]["media"].as_array().unwrap();
            // change each anime's airing info
            for anime in media {
        
                let id = anime["id"].as_i64().unwrap() as i32;
                if anime["next_airing_episode"].is_null() == false {
        
                    let airing_at = anime["next_airing_episode"]["airing_at"].as_i64().unwrap() as i32;
                    let episode = anime["next_airing_episode"]["episode"].as_i64().unwrap() as i32;
        
                    if let Some(anime) = anime_data.get_mut(&id) {
                        anime.next_airing_episode = Some(NextAiringEpisode { airing_at, episode});
                    }
                } else {
                    if let Some(anime) = anime_data.get_mut(&id) {
                        anime.next_airing_episode = None;
                    }
                }
            }

            // anime data has been changed so save changes to disk
            file_operations::write_file_anime_info_cache(&anime_data);
            Ok(())
        },
        Err(error) => return Err(error),
    }
}



// query to change the users data for a specific anime
const ANIME_UPDATE_ENTRY: &str = "
mutation ($id: Int, $media_id: Int, $status: MediaListStatus, $score: Float, $progress: Int, $start_year: Int, $start_month: Int, $start_day: Int, $end_year: Int, $end_month: Int, $end_day: Int, $notes: String) { 
    SaveMediaListEntry (id: $id, mediaId: $media_id, status: $status, score: $score, progress: $progress, startedAt: {year: $start_year, month: $start_month, day: $start_day}, completedAt: {year: $end_year, month: $end_month, day: $end_day}, notes: $notes) {
        id mediaId status score progress startedAt { year month day } completedAt { year month day } notes
    }
}";

// change the users entry data on anilist with the current data
pub async fn update_user_entry(access_token: String, anime: UserAnimeInfo) -> Result<String, &'static str> {

    let mut mutation: String = ANIME_UPDATE_ENTRY.to_string();
    let mut variables = json!({"media_id": anime.media_id, "status": anime.status, "score": anime.score, "progress": anime.progress});

    if anime.id != 0 {
        variables["id"] = json!(anime.id);
    }

    if anime.notes.is_none() {
        mutation = mutation.replace(", $notes: String", "");
        mutation = mutation.replace(", notes: $notes", "");
    } else {
        variables["notes"] = json!(anime.notes.unwrap());
    }

    if anime.started_at.is_none() {
        mutation = mutation.replace(", $start_year: Int, $start_month: Int, $start_day: Int", "");
        mutation = mutation.replace(", startedAt: {year: $start_year, month: $start_month, day: $start_day}", "");
    }
    else {
        let started = anime.started_at.unwrap();
        variables["start_year"] = json!(started.year);
        variables["start_month"] = json!(started.month);
        variables["start_day"] = json!(started.day);
    }

    if anime.completed_at.is_none() {
        mutation = mutation.replace(", $end_year: Int, $end_month: Int, $end_day: Int", "");
        mutation = mutation.replace(", completedAt: {year: $end_year, month: $end_month, day: $end_day}", "");
    }
    else {
        let completed = anime.completed_at.unwrap();
        variables["end_year"] = json!(completed.year);
        variables["end_month"] = json!(completed.month);
        variables["end_day"] = json!(completed.day);
    }

    let json = json!({"query": mutation, "variables": variables});
    
    match post(&json, Some(&access_token)).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);
            Ok(response)
        },
        Err(error) => return Err(error),
    }
}



const USER_SCORE_FORMAT: &str = " query($username: String) { User(name: $username) { mediaListOptions { scoreFormat } } }";
// get the number the users score is out of (1/3, 1/5, 1/10, 1/100, etc)
pub async fn get_user_score_format(username: String) -> Result<String, &'static str> {

    let json = json!({"query": USER_SCORE_FORMAT, "variables": {"username": username}});

    match post(&json, None).await {
        Ok(result) => {
            let format = serde_json::from_str::<serde_json::Value>(&result).unwrap()["data"]["User"]["mediaListOptions"]["scoreFormat"].to_string().replace("\"", "");
            Ok(format)
        },
        Err(error) => return Err(error),
    }
}



const USER_MEDIA_DATA: &str = "query($ids: [Int]) { Page(page: 0, perPage: 50) { media(id_in: $ids) { mediaListEntry { id mediaId status score progress updatedAt startedAt { year month day } completedAt { year month day } } } } }";
// get user data for the requested ids
pub async fn get_media_user_data(ids: Vec<i32>, access_token: String) -> Result<Vec<UserAnimeInfo>, &'static str> {
    
    let json = json!({"query": USER_MEDIA_DATA, "variables": {"ids": ids}});

    match post(&json, Some(&access_token)).await {
        Ok(result) => {
            let response = anilist_to_snake_case(result);

            let response_json = serde_json::from_str::<serde_json::Value>(&response).unwrap();

            let mut user_anime_data: Vec<UserAnimeInfo> = Vec::new();
            if let Some(media) = response_json["data"]["Page"]["media"].as_array(){
        
                for entry in media {
                    let updated_anime: UserAnimeInfo = serde_json::from_value(entry["mediaListEntry"].clone()).unwrap();
                    user_anime_data.push(updated_anime);
                }
            }
            Ok(user_anime_data)
        },
        Err(error) => return Err(error),
    }
}



const USER_ID_QUERY: &str = "query($userName: String) { User(name: $userName) { id } }";
// get the user id for the requested username
pub async fn get_user_id(username: String) -> Option<i32> {
    
    let json = json!({"query": USER_ID_QUERY, "variables": {"userName": username}});
    println!("{}", json.to_string());

    match post(&json, None).await {
        Ok(result) => {

            let result_value = serde_json::from_str::<serde_json::Value>(&result).unwrap();
            let result_map = result_value.as_object().unwrap();
            if result_map.contains_key("errors") {
                println!("Result: {}", result);
                return None;
            } else {
                let user_id = result_map["data"]["User"]["id"].as_i64().unwrap() as i32;
                return Some(user_id);
            }
        },
        Err(_error) => return None,
    }
}



const USER_UPDATED_AT: &str = "query($userName: String) { User(name: $userName) { updatedAt } }";
// get the last time the user's data was changed
pub async fn get_user_updated_at(username: String) -> Option<u64> {
    
    let json = json!({"query": USER_UPDATED_AT, "variables": {"username": username}});

    match post(&json, None).await {
        Ok(result) => {
            let updated_at = serde_json::from_str::<serde_json::Value>(&result).unwrap()["data"]["User"]["updatedAt"].as_u64();
            return updated_at;
        },
        Err(_error) => return None,
    }
}



// returns a list of media that has been updated after the supplied time
const USER_MEDIA_UPDATED: &str = "query($userId: Int, $page: Int, $time: Int) { Page(page: $page, perPage: 50) { pageInfo { hasNextPage } activities(userId: $userId, type: ANIME_LIST, createdAt_greater: $time) { ... on ListActivity { media { id } } } } }";
pub async fn get_updated_media_id(user_id: i32, time: u64) -> Result<Vec<i32>, &'static str> {

    let mut has_next_page = true;
    let mut page_number = 0;
    let mut media_ids: Vec<i32> = Vec::new();
    
    while has_next_page {

        let json = json!({"query": USER_MEDIA_UPDATED, "variables": {"userId": user_id,"page": page_number,"time": time}});
    
        match post(&json, None).await {
            Ok(result) => {
                let response_value = serde_json::from_str::<serde_json::Value>(&result).unwrap();
        
                if let Some(activities_list) = response_value["data"]["Page"]["activities"].as_array() {
                    for activity in activities_list {
                        if let Some(media_id) = activity["media"]["id"].as_i64() {
                            if media_ids.contains(&(media_id as i32)) == false {
                                media_ids.push(media_id as i32);
                            }
                        }
                    }
                }
    
                has_next_page = response_value["data"]["Page"]["pageInfo"]["hasNextPage"].as_bool().unwrap();
                page_number += 1;
            },
            Err(error) => return Err(error),
        }
    }
    
    Ok(media_ids)
}



// send post json to https://graphql.anilist.co/ and return its response as a string
// access token in necessary for creating, updating, deleting, and reading private data
pub async fn post(json: &Value, access_token: Option<&String>) -> Result<String, &'static str> {

    let client = Client::new();
    let mut request_builder = client.post("https://graphql.anilist.co/");
    if access_token.is_some() {
        request_builder = request_builder.header("Authorization", String::from("Bearer ") + access_token.unwrap());
    }

    let response: Result<reqwest::Response, reqwest::Error> = request_builder.header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await;

    if response.is_err() {
        let error = response.err().unwrap();
        if error.is_connect() { // no internet
            println!("connect error");
            return Err("no connection")
        }
        else if error.is_request() { // no internet
            println!("request error");
            return Err("bad request")
        }
        else {
            println!("{:?}", error);
            return Err("error")
        }
    }
    else {
        let response_string = response.unwrap().text().await.unwrap();
        return Ok(response_string)
    }
}