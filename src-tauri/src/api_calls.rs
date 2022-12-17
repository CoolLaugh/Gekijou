use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;


// structs that replacate the structure of returning data
#[derive(Serialize, Deserialize, Debug)]
pub struct TrailerData {
    pub id: String,
    pub site: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Title {
    pub english: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CoverImage {
    pub large: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnilistDate {
    pub year: i32,
    pub month: i32,
    pub day: i32
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimeInfo {
    pub average_score: i32,
    pub cover_image: CoverImage,
    pub description: String,
    pub duration: i32,
    pub episodes: i32,
    pub format: String,
    pub genres: Vec<String>,
    pub id: i32,
    pub is_adult: bool,
    pub popularity: i32,
    pub season: String,
    pub season_year: i32,
    pub title: Title,
    pub trailer: Option<TrailerData>,
    pub anime_type: String, // type is a rust keyword
    pub start_date: AnilistDate,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Media {
    pub Media: AnimeInfo
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: Media
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

const LARGE_IMAGE_QUARY: &str = "query ($id: Int) { Media (id: $id, type: ANIME) { coverImage { large } } }";


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

    // change json keys to snake case
    let average_score_replaced = response.unwrap().replace("averageScore", "average_score");
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