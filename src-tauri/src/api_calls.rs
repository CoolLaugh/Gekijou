

use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::secrets;


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
    pub episodes: Option<i32>,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenData {
    pub token_type: String,
    pub expires_in: i32,
    pub access_token: String,
    pub refresh_token: String
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
query ($id: Int) {
    MediaList (userId: $id, status: CURRENT, type: ANIME) {
        media {
            id
            mediaListEntry {
                status
                score
                progress
            }
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



pub async fn anilist_list_quary_call(access_token: String) -> String {

    // create client and query json
    let client = Client::new();
    let json = json!({"query": ANIME_LIST_QUERY, "variables": {"id": 1}});
    print!("{}", json);

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
    print!("{}", response_string);
    response_string
}



pub async fn anilist_get_access_token(code: String) -> TokenData {

    let client = Client::new();

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



pub async fn test(id: i32) -> String {

    print!("\n{}\n", id);

    let client = Client::new();
    //let json = json!({"query": ANIME_MULTI_INFO_QUERY});
    let json = json!({"query": ANIME_MULTI_INFO_QUERY, "variables": {"id": 21, "id2": 17871}});
    print!("{}\n", json);

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
    print!("{}\n", response_string);
    response_string
}

