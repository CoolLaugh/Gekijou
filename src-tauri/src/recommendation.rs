use std::collections::HashMap;

use crate::{GLOBAL_USER_ANIME_LISTS, GLOBAL_ANIME_DATA, api_calls, file_operations, GLOBAL_USER_ANIME_DATA, recommendation};


#[derive(Debug, Clone, Default)]
struct RecommendTally {
    pub id: i32,
    pub rating: i32,
}

pub async fn tally_recommendations() -> Vec<i32> {
    
    let list = GLOBAL_USER_ANIME_LISTS.lock().await;
    let completed_list = list.get("COMPLETED").unwrap();

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let mut recommend_total: HashMap<i32, i32> = HashMap::new();
    for id in completed_list {

        let recommendations = anime_data.get(id).unwrap().recommendations.clone().unwrap().nodes;
        for rec in recommendations {

            if rec.media_recommendation.is_none() {
                continue;
            }

            let id = rec.media_recommendation.unwrap().id;
            recommend_total.entry(id).and_modify(|r| { *r += rec.rating }).or_insert(rec.rating);
        }
    }
    drop(list);

    let mut recommendations: Vec<RecommendTally> = Vec::new();
    for entry in recommend_total {
        recommendations.push(RecommendTally {
            id: entry.0,
            rating: entry.1
        });
    }

    let user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    recommendations.retain(|entry| { user_data.contains_key(&entry.id) == false });
    drop(user_data);

    recommendations.sort_by(| entry_a, entry_b | {
        entry_b.rating.cmp(&entry_a.rating)
    });

    recommendations.truncate(100);
    
    let mut unknown_ids: Vec<i32> = Vec::new();
    for entry in recommendations.clone() {
        if anime_data.contains_key(&entry.id) == false {
            unknown_ids.push(entry.id);
        }
    }

    drop(anime_data);
    println!("before anilist_api_call_multiple");
    api_calls::anilist_api_call_multiple(unknown_ids).await;
    println!("before write_file_anime_info_cache");
    file_operations::write_file_anime_info_cache().await;
    println!("before GLOBAL_ANIME_DATA.lock().await");

    //let anime_data2 = GLOBAL_ANIME_DATA.lock().await;

    let mut recommend_list: Vec<i32> = Vec::new();
    for entry in recommendations {

        // let title = if anime_data2.contains_key(&entry.id) {
        //     anime_data2.get(&entry.id).unwrap().title.romaji.clone().unwrap()
        // } else {
        //     entry.id.to_string()
        // };

        // print!("{} {}|", title, entry.rating);
        recommend_list.push(entry.id);
    }
    recommend_list
}