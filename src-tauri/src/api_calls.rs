


use std::collections::HashMap;

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::{json};

use crate::{secrets, GLOBAL_ANIME_DATA};


// structs that replacate the structure of returning data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrailerData {
    pub id: String,
    pub site: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub english: Option<String>,
    pub native: Option<String>,
    pub romaji: Option<String>
}

impl Title {
    pub const fn new() -> Title {
        Title { english: None, native: None, romaji: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoverImage {
    pub large: String
}

impl CoverImage {
    pub const fn new() -> CoverImage {
        CoverImage { large: String::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnilistDate {
    pub year: i32,
    pub month: i32,
    pub day: i32
}

impl AnilistDate {
    pub const fn new() -> AnilistDate {
        AnilistDate { year: 0, month: 0, day: 0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnimeInfo {
    pub average_score: i32,
    pub cover_image: CoverImage,
    pub description: String,
    pub duration: i32,
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
}

impl AnimeInfo {
    pub const fn new() -> AnimeInfo {
        AnimeInfo { average_score: 0, cover_image: CoverImage::new(), description: String::new(), duration: 0, 
            episodes: Option::None, format: String::new(), genres: Vec::new(), id: 0, is_adult: false, popularity: 0, 
            season: Option::None, season_year: Option::None, title: Title::new(), trailer: Option::None, anime_type: String::new(), start_date: AnilistDate::new()}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub Media: AnimeInfo
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
}

impl UserSettings {
    pub const fn new() -> UserSettings {
        UserSettings { username: String::new(), title_language: String::new() }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UserAnimeInfo {
    pub id: i32,
    pub media_id: i32,
    pub status: String,
    pub score: i32,
    pub progress: i32,
    pub started_at: Option<FuzzyDate>,
    pub completed_at: Option<FuzzyDate>,
}

impl UserAnimeInfo {
    pub const fn new() -> UserAnimeInfo {
        UserAnimeInfo { id: 0, media_id: 0, status: String::new(), score: 0, progress: 0, started_at: None, completed_at: None }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

impl FuzzyDate {
    pub const fn new() -> FuzzyDate {
        FuzzyDate { year: None, month: None, day: None }
    }
}


const QUERY: &str = "
query ($id: Int) { # Define which variables will be used in the query (id)
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id
        title {
            romaji
            english
            native
        }
        coverImage {
            extraLarge
            large
            medium
            color
        }
    }
}
";

// request json for anilist api
const ANIME_INFO_QUERY: &str = "
query ($id: Int) { # Define which variables will be used in the query (id)
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id
        title {
            english
        }
        coverImage {
            large
        }
        season
        seasonYear
        type
        format
        episodes
        duration
        isAdult
        genres
        averageScore
        popularity
        description
        trailer {
            id
            site
        }
        startDate {
            year
            month
            day
        }
    }
}
";

const ANIME_LIST_QUERY: &str = "
query ($username: String) {
    MediaListCollection (userName: $username, type: ANIME) {
        lists {
            name
            entries {
                id
                mediaId
                status
                score
                progress
                startedAt {
                    year
                    month
                    day
                }
                completedAt {
                    year
                    month
                    day
                }
            }
            status
        }
    }
}
";

const LARGE_IMAGE_QUARY: &str = "query ($id: Int) { Media (id: $id, type: ANIME) { coverImage { large } } }";

const ANIME_ALLINFO_QUERY: &str = "
query ($id: Int) { # Define which variables will be used in the query (id)
    Media (id: $id, type: ANIME) { # Insert our variables into the query arguments (id) (type: ANIME is hard-coded in the query)
        id
		idMal
        title {
			romaji
            english
			native
			userPreferred
        }
		type
        format
		status
        description
        startDate {
            year
            month
            day
        }
        endDate {
            year
            month
            day
        }
        season
        seasonYear
		seasonInt
        episodes
        duration
		chapters
		volumes
		countryOfOrigin
		isLicensed
		source
		hashtag
        trailer {
            id
            site
        }
		updatedAt
        coverImage {
            large
        }
		bannerImage
		genres
		synonyms
        averageScore
		meanScore
		popularity
		isLocked
		trending
		favourites
		tags {
			id
			name
			description
			category
			rank
			isGeneralSpoiler
			isMediaSpoiler
			isAdult
			userId
		}
        relations {
            edges {
                id
                relationType
                node {
                    title {
                        english
                        romaji
                    }
                }
            }
            nodes {
                id
                title {
                    english
                    romaji
                }
            }
        }
		isFavourite
		isFavouriteBlocked
        isAdult
		nextAiringEpisode {
			id
			airingAt
			timeUntilAiring
			episode
			mediaId
			media {
				id
			}
		}
		externalLinks {
			id
			url
			site
			siteId
			type
			language
			color
			icon
			notes
			isDisabled
		}
		streamingEpisodes {
			title
			thumbnail
			url
			site
		}
		rankings {
			id
			rank
			type
			format
			year
			season
			allTime
			context
		}
		recommendations {
			nodes {
				id
				rating
				media {
					id
					title {
						romaji
						english
						native
						userPreferred
					}
				}
				mediaRecommendation {
					title {
						romaji
						english
						native
						userPreferred
					}
				}
				user {
					id
					name
				}
			}
		}
		stats {
			scoreDistribution {
				score
				amount
			}
			statusDistribution {
				status
				amount
			}
		}
		siteUrl
		autoCreateForumThread
		isRecommendationBlocked
		isReviewBlocked
		modNotes
    }
}
";

const ANIME_MULTI_INFO_QUERY: &str = "
query ($id: Int, $id2: Int) {
    A:Media (id: $id, type: ANIME) {
        id
        title {
            english
        }
    }
    B:Media (id: $id2, type: ANIME) {
        id
        title {
            english
        }
    }
}
";

const ANIME_UPDATE_PROGRESS_ENTRY: &str = "
mutation ($id: Int, $progress: Int) {
    SaveMediaListEntry (id: $id, progress: $progress) {
    }
}
";

const ANIME_UPDATE_ENTRY: &str = "mutation ($id: Int, $status: MediaListStatus, $score: Float, $progress: Int, $syear: Int, $smonth: Int, $sday: Int, $eyear: Int, $emonth: Int, $eday: Int) { 
    SaveMediaListEntry (id: $id, status: $status, score: $score, progress: $progress, startedAt: {year: $syear, month: $smonth, day: $sday}, completedAt: {year: $eyear, month: $emonth, day: $eday}) {
        id
        mediaId
        status
        score
        progress
        startedAt {
            year
            month
            day
        }
        completedAt {
            year
            month
            day
        }
    }
}";


// retrive information on anime using it's anilist id
pub async fn anilist_api_call(id: i32) -> AnimeInfo {

    // create client and query json
    let client = Client::new();
    let json = json!({"query": ANIME_INFO_QUERY, "variables": {"id": id}});

    // get media information from anilist api
    let response = client.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();

    print!("{}", response_string);

    // change json keys to snake case
    let average_score_replaced = response_string.replace("averageScore", "average_score");
    let cover_image_replaced = average_score_replaced.replace("coverImage", "cover_image");
    let is_adult_replaced = cover_image_replaced.replace("isAdult", "is_adult");
    let season_year_replaced = is_adult_replaced.replace("seasonYear", "season_year");
    // type is already snake case but it is a rust keyword
    let type_replaced = season_year_replaced.replace("type", "anime_type");
    let start_date_replaced = type_replaced.replace("startDate", "start_date");

    // return struct with media information
    let json: Data = serde_json::from_str(&start_date_replaced).unwrap();
    json.data.Media
}



// split requests for anime info to avoid the complexity limit of 500
pub async fn anilist_get_anime_info_split(anime: Vec<i32>) {

    // each entry has 26 complexity
    // max extries is 19 (19 x 26 = 494)
    let vec_length = 19;
    let number_of_splits = (anime.len() + vec_length - 1) / (vec_length); // to ceil the value
    let mut split_anime: Vec<Vec<i32>> = Vec::new();
    split_anime.resize(number_of_splits, Vec::new());

    for i in 0..anime.len() {
        split_anime[i / vec_length].push(anime[i]);
        //print!("\n{} {} {}", i, i % vec_length, split_anime[i % vec_length].len());
    }

    for list in split_anime {
        if anilist_get_anime_info(list).await == false{
            break; // too many requests
        }
    }
}


pub async fn anilist_get_anime_info(anime: Vec<i32>) -> bool {

    if anime.len() == 0 {
        return true;
    }
    //print!("\n{}", anime.len());

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
    let client = Client::new();
    let json = json!({"query": query, "variables": ids});

    // get media information from anilist api
    let response = client.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    //print!("\n{}", response_string);
    
    // change json keys to snake case
    let average_score_replaced = response_string.replace("averageScore", "average_score");
    let cover_image_replaced = average_score_replaced.replace("coverImage", "cover_image");
    let is_adult_replaced = cover_image_replaced.replace("isAdult", "is_adult");
    let season_year_replaced = is_adult_replaced.replace("seasonYear", "season_year");
    // type is already snake case but it is a rust keyword
    let type_replaced = season_year_replaced.replace("type", "anime_type");
    let start_date_replaced = type_replaced.replace("startDate", "start_date");


    let anime_data: serde_json::Value = serde_json::from_str(&start_date_replaced).unwrap();
    let anime_list = &mut *GLOBAL_ANIME_DATA.lock().await;

    //print!("\narray: {} object: {} string: {} hr: {} null: {} number: {}", anime_data["data"].is_array(), anime_data["data"].is_object(), anime_data["data"].is_string(), anime_data["data"].is_human_readable(), anime_data["data"].is_null(), anime_data["data"].is_number());

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



pub async fn anilist_list_quary_call(username: String, access_token: String) -> String {

    // create client and query json
    let client = Client::new();
    let json = json!({"query": ANIME_LIST_QUERY, "variables": {"username": username}});

    // get media information from anilist api
    let response = client.post("https://graphql.anilist.co/")
        .header("Authorization", String::from("Bearer ") + &access_token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    
    let media_id_replaced = response_string.replace("mediaId", "media_id");
    let started_at_replaced = media_id_replaced.replace("startedAt", "started_at");
    let completed_at_replaced = started_at_replaced.replace("completedAt", "completed_at");

    completed_at_replaced
}



pub async fn anilist_get_access_token(code: String) -> TokenData {

    let client = Client::new();
    print!("{}\n\n", code);

    let response = client.post("https://anilist.co/api/v2/oauth/token")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "client_id": secrets::CLIENT_ID,
            "client_secret": secrets::CLIENT_SECRET,
            "redirect_uri": secrets::REDIRECT_URI,
            "code": code
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    print!("{}\n\n", response_string);

    if response_string.contains("\"error\"") {
        return TokenData { token_type: String::from("error"), expires_in: 0, access_token: String::new(), refresh_token: String::new() };
    }
    
    return serde_json::from_str(&response_string).unwrap();
}



pub async fn anilist_oauth_call() -> String {

    let username = "";

    // create client and query json
    let client = Client::new();
    let json = json!({"query": ANIME_LIST_QUERY, "variables": {"userName": username}});
    print!("{}", json);

    // get media information from anilist api
    let response = client.post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
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

pub async fn update_user_entry(access_token: String, anime: UserAnimeInfo) -> String {

    let mut mutation: String = ANIME_UPDATE_ENTRY.to_string();
    let mut variables = json!({"id": anime.id, "status": anime.status, "score": anime.score, "progress": anime.progress});

    if anime.started_at.is_none() {
        mutation = mutation.replace(", $syear: Int, $smonth: Int, $sday: Int", "");
        mutation = mutation.replace(", startedAt: {year: $syear, month: $smonth, day: $sday}", "");
    }
    else {
        let started = anime.started_at.unwrap();
        variables["syear"] = json!(started.year);
        variables["smonth"] = json!(started.month);
        variables["sday"] = json!(started.day);
    }

    if anime.completed_at.is_none() {
        mutation = mutation.replace(", $eyear: Int, $emonth: Int, $eday: Int", "");
        mutation = mutation.replace(", completedAt: {year: $eyear, month: $emonth, day: $eday}", "");
    }
    else {
        let completed = anime.completed_at.unwrap();
        variables["eyear"] = json!(completed.year);
        variables["emonth"] = json!(completed.month);
        variables["eday"] = json!(completed.day);
    }

    let json = json!({"query": mutation, "variables": variables});
    print!("{}\n", json);

    let client = Client::new();
    let response = client.post("https://graphql.anilist.co/")
        .header("Authorization", String::from("Bearer ") + &access_token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    
    let media_id_replaced = response_string.replace("mediaId", "media_id");
    let started_at_replaced = media_id_replaced.replace("startedAt", "started_at");
    let completed_at_replaced = started_at_replaced.replace("completedAt", "completed_at");

    print!("{}\n", completed_at_replaced);
    completed_at_replaced
}

pub async fn test(id: i32, access_token: String) -> String {

    print!("\n{}\n", id);

    let client = Client::new();
    //let json = json!({"query": ANIME_MULTI_INFO_QUERY});
    let json = json!({"query": ANIME_ALLINFO_QUERY, "variables": {"id": 5081 }});
    print!("{}\n", json);

    let response = client.post("https://graphql.anilist.co/")
        .header("Authorization", String::from("Bearer ") + &access_token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();
    print!("{}\n", response_string);
    response_string
}

