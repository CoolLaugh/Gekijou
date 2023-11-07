use std::collections::HashMap;

use reqwest::Client;

use crate::{api_calls::TokenData, secrets::MAL_CLIENT_ID, constants::MAL_USER_STATUSES, user_data::UserInfo};




pub async fn mal_get_access_token(code: &str, code_verifier: &str) -> TokenData {

    let client = Client::new();

    let body = format!("client_id={}&grant_type=authorization_code&code={}&code_verifier={}", MAL_CLIENT_ID, code, code_verifier);
    
    let response = client.post("https://myanimelist.net/v1/oauth2/token")
        .basic_auth(MAL_CLIENT_ID, Some(""))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    let response_string = response.unwrap();

    println!("{}", response_string);

    let token: TokenData = serde_json::from_str(&response_string).unwrap();

    token
}

pub async fn mal_get_list(username: String, status: String, access_token: String, anime_user_data: &mut HashMap<i32, UserInfo>, anime_user_list_lock: &mut HashMap<String, Vec<i32>>) -> Option<String> {

    let client = Client::new();

    for status in MAL_USER_STATUSES {
        
        let response: Result<reqwest::Response, reqwest::Error> = client.get("https://api.myanimelist.net/v2/users/@me/animelist")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Authorization", format!("Bearer {}", access_token))
            .query(&[("fields", "my_list_status{status,score,num_episodes_watched,is_rewatching,start_date,finish_date,num_times_rewatched,comments,updated_at}"), 
                ("status", status)])
            .send().await;

        match response {
            Ok(result) => {
                //let lists: serde_json::Value = serde_json::from_str::<serde_json::Value>(&result.text().await).unwrap()["data"]["MediaListCollection"]["lists"].take();
            },
            Err(error) => { println!("{}", error)},
        }

    }




    None
}