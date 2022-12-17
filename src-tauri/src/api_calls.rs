


use std::{collections::HashMap, cmp::Ordering};

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};

use crate::{secrets, GLOBAL_ANIME_DATA, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_ANIME_LISTS};


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
    pub user_preferred: Option<String>
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
        if other.year.is_none() && self.year.is_some() { return Ordering::Greater }
        if self.year.unwrap() < other.year.unwrap() { return Ordering::Less }
        if self.year.unwrap() > other.year.unwrap()  { return Ordering::Greater }

        if self.month.is_none() && other.month.is_some() { return Ordering::Less }
        if other.month.is_none() && self.month.is_some() { return Ordering::Greater } 
        if self.month.unwrap() < other.month.unwrap() { return Ordering::Less } 
        if self.month.unwrap() > other.month.unwrap()  { return Ordering::Greater }

        if self.day.is_none() && other.day.is_some() { return Ordering::Less } 
        if other.day.is_none() && self.day.is_some() { return Ordering::Greater } 
        if self.day.unwrap() < other.day.unwrap() { return Ordering::Less } 
        if self.day.unwrap() > other.day.unwrap()  { return Ordering::Greater }

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
    pub title: Title,
    pub cover_image: CoverImage
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub id: i32,
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
    pub format: String,
    pub genres: Vec<String>,
    pub id: i32,
    pub is_adult: bool,
    pub popularity: i32,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub start_date: AnilistDate,
    pub title: Title,
    pub trailer: Option<TrailerData>,
    pub anime_type: String, // type is a rust keyword
    pub relations: Relations,
    pub recommendations: Option<Recommendations>,
    pub tags: Vec<Tag>,
    pub trending: i32,
    pub studios: Studio,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Studio {
    pub nodes: Vec<NodeName>
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeName {
    pub name: String
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSettings {
    pub username: String,
    pub title_language: String,
    pub show_spoilers: bool,
    pub show_adult: bool,
    pub folders: Vec<String>,
    pub update_delay: i32,
    pub score_format: String,
    pub highlight_color: String,
}

impl UserSettings {
    pub const fn new() -> UserSettings {
        UserSettings { username: String::new(), title_language: String::new(), show_spoilers: false, show_adult: true, folders: Vec::new(), update_delay: 0, score_format: String::new(), highlight_color: String::new() }
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
}

impl UserAnimeInfo {
    pub const fn new() -> UserAnimeInfo {
        UserAnimeInfo { id: 0, media_id: 0, status: String::new(), score: 0.0, progress: 0, started_at: None, completed_at: None }
    }
}

// request json for anilist api
const ANIME_INFO_QUERY: &str = "
query ($id: Int) { # Define which variables will be used in the query (id)
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
query ($id: Int) { # Define which variables will be used in the query (id)
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id idMal title { romaji english native userPreferred } type format status description startDate { year month day } endDate { year month day }
        season seasonYear seasonInt episodes duration chapters volumes countryOfOrigin isLicensed source hashtag trailer { id site } updatedAt coverImage { large } bannerImage
		genres synonyms averageScore meanScore popularity isLocked trending favourites isFavourite isFavouriteBlocked isAdult
		tags { id name description category rank isGeneralSpoiler isMediaSpoiler isAdult userId }
        relations { edges { id relationType node { title { userPreferred } coverImage { large } } } nodes { id title { english romaji } } }
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
query($page: Int $type: MediaType $format: [MediaFormat] $season: MediaSeason $seasonYear: Int $genres: [String]$tags: [String] $sort: [MediaSort] = [POPULARITY_DESC, SCORE_DESC]) {
    Page(page: $page, perPage: 50) {
        pageInfo { total perPage currentPage lastPage hasNextPage }
        media(type: $type season: $season format_in: $format seasonYear: $seasonYear genre_in: $genres tag_in: $tags sort: $sort) {
            id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
            duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
            relations { edges { id relationType node { title { userPreferred } coverImage { large } } } }
            recommendations { nodes { rating mediaRecommendation { id title { userPreferred } coverImage { large } } } }
            tags { name isGeneralSpoiler isMediaSpoiler description }
            studios(isMain: true) { nodes { name } }
        }
    }
}";

// query to change the users data for a specific anime
const ANIME_UPDATE_ENTRY: &str = "
mutation ($id: Int, $media_id: Int, $status: MediaListStatus, $score: Float, $progress: Int, $start_year: Int, $start_month: Int, $start_day: Int, $end_year: Int, $end_month: Int, $end_day: Int) { 
    SaveMediaListEntry (id: $id, mediaId: $media_id, status: $status, score: $score, progress: $progress, startedAt: {year: $start_year, month: $start_month, day: $start_day}, completedAt: {year: $end_year, month: $end_month, day: $end_day}) {
        id mediaId status score progress startedAt { year month day } completedAt { year month day }
    }
}";

// query for a specific list along with all user data and media data for the anime on that list
const USER_LIST_WITH_MEDIA: &str = "query($userName: String, $status: MediaListStatus) {
    MediaListCollection(userName: $userName, type:ANIME, status:$status) {
      lists {
        name 
        entries {
          id mediaId status score progress updatedAt startedAt { year month day } completedAt { year month day }
          media {
            id title { userPreferred romaji english native } coverImage { large } season seasonYear type format episodes trending
            duration isAdult genres averageScore popularity description status trailer { id site } startDate { year month day }
            relations { edges { id relationType node { title { userPreferred } coverImage { large } } } }
            recommendations { nodes { rating mediaRecommendation { id title { userPreferred } coverImage { large } } } }
            tags { name isGeneralSpoiler isMediaSpoiler description }
            studios(isMain: true) { nodes { name } }
          }
        }
      }
    }
  }";

// retrieve a list of anime based on criteria like the year/season it was released, format, or genre
pub async fn anilist_browse_call(page: i32, year: String, season: String, genre: String, format: String, order: String) -> serde_json::Value {

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

    let json = json!({"query": ANIME_BROWSE, "variables": variables});

    let mut response = post(&json, None).await;

    response = response.replace("averageScore", "average_score");
    response = response.replace("coverImage", "cover_image");
    response = response.replace("isAdult", "is_adult");
    response = response.replace("seasonYear", "season_year");
    response = response.replace("type", "anime_type");
    response = response.replace("startDate", "start_date");
    response = response.replace("userPreferred", "user_preferred");
    response = response.replace("relationType", "relation_type");
    response = response.replace("mediaRecommendation", "media_recommendation");
    response = response.replace("isGeneralSpoiler", "is_general_spoiler");
    response = response.replace("isMediaSpoiler", "is_media_spoiler");

    serde_json::from_str(&response).unwrap()
}


const ANIME_DELETE_ENTRY: &str = "
mutation ($id: Int) { 
    DeleteMediaListEntry (id: $id) {
        deleted
    }
}";

// remove a anime from the users anime list
pub async fn anilist_remove_entry(id: i32, access_token: String) -> bool {

    let json = json!({"query": ANIME_DELETE_ENTRY, "variables": {"id": id}});

    let response = post(&json, Some(&access_token)).await;

    let deleted = serde_json::from_str::<serde_json::Value>(&response).unwrap()["data"]["DeleteMediaListEntry"]["deleted"].as_bool().unwrap();
    
    deleted
}

// retrieve information on anime using it's anilist id
pub async fn anilist_api_call(id: i32) -> AnimeInfo {
    
    // create client and query json
    let json = json!({"query": ANIME_INFO_QUERY, "variables": {"id": id}});

    let mut response = post(&json, None).await;

    // change json keys to snake case
    response = response.replace("\"Media\"", "\"media\"")
        .replace("averageScore", "average_score")
        .replace("coverImage", "cover_image")
        .replace("isAdult", "is_adult")
        .replace("seasonYear", "season_year")
        .replace("type", "anime_type") // type is already snake case but it is a rust keyword
        .replace("startDate", "start_date");

    // return struct with media information
    let json: Data = serde_json::from_str(&response).unwrap();
    json.data.media
}


// retrieve information on anime using it's anilist id
// returns a message if a error occurred
pub async fn anilist_get_list(username: String, status: String, access_token: String) -> Option<String> {

    // create query json
    let json = json!({"query": USER_LIST_WITH_MEDIA, "variables": {"userName": username, "status": status}});

    let mut response = post(&json, Some(&access_token)).await;

    response = response.replace("\"Media\"", "\"media\"")
        .replace("averageScore", "average_score")
        .replace("coverImage", "cover_image")
        .replace("isAdult", "is_adult")
        .replace("seasonYear", "season_year")
        .replace("type", "anime_type")
        .replace("startDate", "start_date")
        .replace("userPreferred", "user_preferred")
        .replace("relationType", "relation_type")
        .replace("mediaRecommendation", "media_recommendation")
        .replace("isGeneralSpoiler", "is_general_spoiler")
        .replace("isMediaSpoiler", "is_media_spoiler");
    
    let response_json: serde_json::Value = serde_json::from_str::<serde_json::Value>(&response).unwrap();

    if response_json.is_object() && response_json.get("errors").is_some() {
        let message = response_json["errors"][0]["message"].as_str().unwrap().to_string();
        return Some(message)
    }
    let list: serde_json::Value = serde_json::from_str::<serde_json::Value>(&response).unwrap()["data"]["MediaListCollection"]["lists"][0]["entries"].take();

    let mut anime_user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    let mut anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut anime_user_list_lock = GLOBAL_USER_ANIME_LISTS.lock().await;
    let anime_user_list = anime_user_list_lock.entry(status).or_default();

    anime_user_list.clear();
    for entry in list.as_array().unwrap() {
        
        let user_info: UserAnimeInfo = UserAnimeInfo { id: entry["id"].as_i64().unwrap() as i32, 
                                                        media_id: entry["mediaId"].as_i64().unwrap() as i32, 
                                                        status: entry["status"].as_str().unwrap().to_string(), 
                                                        score: entry["score"].as_f64().unwrap() as f32, 
                                                        progress: entry["progress"].as_i64().unwrap() as i32, 
                                                        started_at: serde_json::from_value(entry["startedAt"].clone()).unwrap(), 
                                                        completed_at: serde_json::from_value(entry["completedAt"].clone()).unwrap() };

        anime_user_data.insert(user_info.media_id, user_info);

        let media: AnimeInfo = serde_json::from_value(entry["media"].clone()).unwrap();

        anime_user_list.push(media.id);
        anime_data.insert(media.id, media);
    }
    
    None
}



// split requests for anime info to avoid the complexity limit of 500
pub async fn anilist_get_anime_info_split(anime: Vec<i32>) {

    // each entry has 26 complexity
    // max entries is 19 (19 x 26 = 494)
    let vec_length = 19;
    let number_of_splits = (anime.len() + vec_length - 1) / (vec_length); // to ceil the value
    let mut split_anime: Vec<Vec<i32>> = Vec::new();
    split_anime.resize(number_of_splits, Vec::new());

    for i in 0..anime.len() {
        split_anime[i / vec_length].push(anime[i]);
    }

    for list in split_anime {
        if anilist_get_anime_info(list).await == false{
            break; // too many requests
        }
    }
}

// retrieves information for specific anime based on its id
pub async fn anilist_get_anime_info(anime: Vec<i32>) -> bool {

    if anime.len() == 0 {
        return true;
    }

    const MULTI_ANIME_INFO_QUERY_START_FRONT: &str = "query ($id0: Int";
    const MULTI_ANIME_INFO_QUERY_START_BACK: &str = ") { ";
    const MULTI_ANIME_INFO_QUERY: &str = "R0:Media (id: $id0, type: ANIME) { id title { romaji english native } coverImage { large } season seasonYear type format episodes duration isAdult genres averageScore popularity description trailer { id site } startDate { year month day } } ";
    const MULTI_ANIME_INFO_QUERY_END: &str = "}";

    let mut query: String = MULTI_ANIME_INFO_QUERY_START_FRONT.to_string();
    for i in 1..anime.len() {
        query.push_str(", $id");
        query.push_str(&i.to_string());
        query.push_str(": Int");
    }
    query.push_str(MULTI_ANIME_INFO_QUERY_START_BACK);
    for i in 0..anime.len() {
        query.push_str(&MULTI_ANIME_INFO_QUERY.replace("0", &i.to_string()));
    }
    query.push_str(MULTI_ANIME_INFO_QUERY_END);



    let mut ids: HashMap<String, i32> = HashMap::new();

    for i in 0..anime.len() {
        let mut id_name = String::from("id");
        id_name.push_str(&i.to_string());
        ids.insert(id_name, *anime.get(i).unwrap());
    }



    // create client and query json
    let json = json!({"query": query, "variables": ids});

    // get media information from anilist api
    let mut response = post(&json, None).await;
    
    // change json keys to snake case
    response = response.replace("\"Media\"", "\"media\"")
        .replace("averageScore", "average_score")
        .replace("coverImage", "cover_image")
        .replace("isAdult", "is_adult")
        .replace("seasonYear", "season_year")
        .replace("type", "anime_type") // type is already snake case but it is a rust keyword
        .replace("startDate", "start_date");


    let anime_data: serde_json::Value = serde_json::from_str(&response).unwrap();
    let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;

    if anime_data["data"].as_object().is_none() {
        if anime_data["errors"].is_array() {
            return false;
        }
    }

    for item in anime_data["data"].as_object().unwrap() {

        let anime: AnimeInfo = match serde_json::from_str(&item.1.to_string()) {
            Err(why) => panic!("error, {}, {}", why, item.1.to_string()),
            Ok(anime) => anime,
        };

        anime_list.insert(anime.id, anime);
    }
    true
}


// gets the users anime lists with all user data on each anime
pub async fn anilist_list_query_call(username: String, access_token: String) -> String {

    // create client and query json
    let json = json!({"query": ANIME_LIST_QUERY, "variables": {"username": username}});

    // get media information from anilist api
    let mut response = post(&json, Some(&access_token)).await;
    
    response = response.replace("mediaId", "media_id")
        .replace("startedAt", "started_at")
        .replace("completedAt", "completed_at");

    response
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


// change the users entry data on anilist with the current data
pub async fn update_user_entry(access_token: String, anime: UserAnimeInfo) -> String {

    let mut mutation: String = ANIME_UPDATE_ENTRY.to_string();
    let mut variables = json!({"media_id": anime.media_id, "status": anime.status, "score": anime.score, "progress": anime.progress});

    if anime.id != 0 {
        variables["id"] = json!(anime.id);
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
    
    let mut response = post(&json, Some(&access_token)).await;
    
    response = response.replace("mediaId", "media_id")
        .replace("startedAt", "started_at")
        .replace("completedAt", "completed_at");
    
    response
}


const USER_SCORE_FORMAT: &str = "
query($username: String) {
    User(name: $username) {
        mediaListOptions {
            scoreFormat
        }
    }
}";
pub async fn get_user_score_format(username: String) -> String {

    let json = json!({"query": USER_SCORE_FORMAT, "variables": {"username": username}});

    let response = post(&json, None).await;

    let format = serde_json::from_str::<serde_json::Value>(&response).unwrap()["data"]["User"]["mediaListOptions"]["scoreFormat"].to_string().replace("\"", "");
    
    format
}

// send post json to https://graphql.anilist.co/ and return its response as a string
// access token in necessary for creating, updating, deleting, and reading private data
pub async fn post(json: &Value, access_token: Option<&String>) -> String {

    let client = Client::new();
    let mut request_builder = client.post("https://graphql.anilist.co/");
    if access_token.is_some() {
        request_builder = request_builder.header("Authorization", String::from("Bearer ") + access_token.unwrap());
    }
    let response = request_builder.header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    response_string
}