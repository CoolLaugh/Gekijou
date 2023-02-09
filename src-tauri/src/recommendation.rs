use std::collections::HashMap;

use crate::{GLOBAL_USER_ANIME_LISTS, GLOBAL_ANIME_DATA, api_calls, file_operations, GLOBAL_USER_ANIME_DATA, GLOBAL_USER_SETTINGS};



#[derive(Debug, Clone, Default)]
struct RecommendTally {
    pub id: i32,
    pub rating: f32,
}



pub async fn tally_recommendations(genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) -> Vec<i32> {
    
    // use the completed list to grab user recommendations
    let list = GLOBAL_USER_ANIME_LISTS.lock().await;
    let completed_list = list.get("COMPLETED").unwrap();


    let anime_data = GLOBAL_ANIME_DATA.lock().await; // used to remove anime the user has already watched
    let user_data = GLOBAL_USER_ANIME_DATA.lock().await; // uses the user score to modify the recommended rating
    let score_format = GLOBAL_USER_SETTINGS.lock().await.score_format.clone(); // used to properly convert score into a modifier

    // tally up all recommendations and combine the ratings for the same show
    let mut recommend_total: HashMap<i32, f32> = HashMap::new();
    for id in completed_list {

        let recommendations = anime_data.get(id).unwrap().recommendations.clone().unwrap().nodes;
        for rec in recommendations {

            // media is null, it may have been removed after the recommendation was made
            if rec.media_recommendation.is_none() {
                continue;
            }

            let recommendation_id = rec.media_recommendation.unwrap().id;
            let score_modifier = if user_data.contains_key(&id) == false {
                // anime is in completed list but has no user data, this is a bug
                println!("Missing: {}", id);
                1.0
            } else {
                score_to_rating_modifier(user_data.get(&id).unwrap().score, score_format.as_str())
            };

            // add the recommendation to the list or add the rating to the existing recommendation
            recommend_total.entry(recommendation_id).and_modify(|r| { *r += (rec.rating as f32) * score_modifier }).or_insert((rec.rating as f32) * score_modifier);
        }
    }
    drop(list);

    // remove anime already in the users lists
    recommend_total.retain(|id, _| { user_data.contains_key(&id) == false });
    drop(user_data);

    // move to a vector so the entries can be sorted
    let mut recommendations: Vec<RecommendTally> = Vec::new();
    for entry in recommend_total {
        recommendations.push(RecommendTally {
            id: entry.0,
            rating: entry.1
        });
    }

    // sort by most recommended
    recommendations.sort_by(| entry_a, entry_b | {
        entry_b.rating.partial_cmp(&entry_a.rating).unwrap()
    });

    // remove the least recommended shows but not too many in case it will be further filtered
    recommendations.truncate(1000);
    
    // find shows that are missing data
    let mut unknown_ids: Vec<i32> = Vec::new();
    for entry in recommendations.clone() {
        if anime_data.contains_key(&entry.id) == false {
            unknown_ids.push(entry.id);
        }
    }

    // get information on any show which is missing
    drop(anime_data);
    api_calls::anilist_api_call_multiple(unknown_ids).await;
    file_operations::write_file_anime_info_cache().await;

    // if any filter is used
    if genre_filter.is_empty() == false || year_min_filter != 0 || year_max_filter != 0 || format_filter.is_empty() == false {

        let anime_data = GLOBAL_ANIME_DATA.lock().await;

        // filter out any show which doesn't match the genre
        if genre_filter.is_empty() == false {

            recommendations.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().genres.contains(&genre_filter)
            })
        }

        // filter out any show which is too old
        if year_min_filter != 0 {

            recommendations.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().season_year.is_some() &&
                anime_data.get(&rec.id).unwrap().season_year.unwrap() >= year_min_filter
            })
        }

        // filter out any show which is too new
        if year_max_filter != 0 {

            recommendations.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().season_year.is_some() &&
                anime_data.get(&rec.id).unwrap().season_year.unwrap() <= year_max_filter
            })
        }

        // filter out any show which doesn't match the format
        if format_filter.is_empty() == false {

            recommendations.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().format.is_some() &&
                anime_data.get(&rec.id).unwrap().format.as_ref().unwrap().eq(&format_filter)
            })
        }
    }

    // reduce number of shows so the ui loads faster
    recommendations.truncate(100);
    // only keep anime ids, no other information is needed
    let mut recommend_list: Vec<i32> = Vec::new();
    for entry in recommendations {

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

    // convert to modifier
    match score_format {
        "POINT_100" => score / 50.0,
        "POINT_10_DECIMAL" => score / 5.0,
        "POINT_10" => score / 5.0,
        "POINT_5" => score / 2.5,
        "POINT_3" => score - 1.0,
        _ => 1.0,
    }
}
