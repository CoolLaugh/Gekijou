use std::collections::HashMap;

use crate::{GLOBAL_USER_ANIME_LISTS, GLOBAL_ANIME_DATA, api_calls, file_operations, GLOBAL_USER_ANIME_DATA, recommendation, GLOBAL_USER_SETTINGS};


#[derive(Debug, Clone, Default)]
struct RecommendTally {
    pub id: i32,
    pub rating: f32,
}

pub async fn tally_recommendations() -> Vec<i32> {
    
    let list = GLOBAL_USER_ANIME_LISTS.lock().await;
    let completed_list = list.get("COMPLETED").unwrap();

    let anime_data = GLOBAL_ANIME_DATA.lock().await;
    let user_data = GLOBAL_USER_ANIME_DATA.lock().await;
    let score_format = GLOBAL_USER_SETTINGS.lock().await.score_format.clone();
    let mut recommend_total: HashMap<i32, f32> = HashMap::new();
    for id in completed_list {

        let recommendations = anime_data.get(id).unwrap().recommendations.clone().unwrap().nodes;
        for rec in recommendations {

            if rec.media_recommendation.is_none() {
                continue;
            }

            let recommendation_id = rec.media_recommendation.unwrap().id;
            let score_modifier = if user_data.contains_key(&id) == false {
                println!("Missing: {}", id);
                1.0
            } else {
                score_to_rating_modifier(user_data.get(&id).unwrap().score, score_format.as_str())
            };

            recommend_total.entry(recommendation_id).and_modify(|r| { *r += (rec.rating as f32) * score_modifier }).or_insert((rec.rating as f32) * score_modifier);
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
        entry_b.rating.partial_cmp(&entry_a.rating).unwrap()
    });

    recommendations.truncate(100);
    
    let mut unknown_ids: Vec<i32> = Vec::new();
    for entry in recommendations.clone() {
        if anime_data.contains_key(&entry.id) == false {
            unknown_ids.push(entry.id);
        }
    }

    drop(anime_data);
    api_calls::anilist_api_call_multiple(unknown_ids).await;
    file_operations::write_file_anime_info_cache().await;

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

/// convert the score to a rating modifier. 
/// higher scores will produce a higher modifier so shows that a user liked will be worth more than a show the user disliked
fn score_to_rating_modifier(score: f32, score_format: &str) -> f32 {

    // show has no score
    if score == 0.0 {
        return 1.0;
    }

    match score_format {
        "POINT_100" => score / 50.0,
        "POINT_10_DECIMAL" => score / 5.0,
        "POINT_10" => score / 5.0,
        "POINT_5" => score / 2.5,
        "POINT_3" => score - 1.0,
        _ => 1.0,
    }
}
