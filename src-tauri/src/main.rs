#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]



pub mod api_calls;



#[tauri::command]
async fn get_anime_info_query(id: i32) -> api_calls::AnimeInfo {

    let response = api_calls::anilist_api_call(id).await;    
    print!("{}", response.id);
    response
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_anime_info_query])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
