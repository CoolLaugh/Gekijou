#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde_json::json;
use reqwest::Client;

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

const LARGE_IMAGE_QUARY: &str = "query ($id: Int) { Media (id: $id, type: ANIME) { coverImage { large } } }";

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn getanime(id: i32) -> String {

    let client = Client::new();
    let json = json!({"query": QUERY, "variables": {"id": id}});

    let resp = client.post("https://graphql.anilist.co/")
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .body(json.to_string())
    .send()
    .await
    .unwrap()
    .text()
    .await;

    let result: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
    println!("{}", result);

    let image_url = result["data"]["Media"]["coverImage"]["large"].to_string().replace("\"","");

    format!("{}", image_url)
}

#[tauri::command]
async fn get_cover_image(id: i32) -> String {
    let json = json!({"query": LARGE_IMAGE_QUARY, "variables": {"id": id}});
    let response = anilist_api_call(json.to_string()).await;
    let image_url = response["data"]["Media"]["coverImage"]["large"].to_string().replace("\"","");
    image_url
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![getanime])
        .invoke_handler(tauri::generate_handler![get_cover_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn anilist_api_call(query: String) -> serde_json::Value {

    let client = Client::new();

    let resp = client.post("https://graphql.anilist.co/")
    .header("Content-Type", "application/json")
    .header("Accept", "application/json")
    .body(query)
    .send()
    .await
    .unwrap()
    .text()
    .await;

    let result: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
    result
}