use std::collections::HashMap;
use crate::{GLOBAL_ANIME_DATA, api_calls, file_operations, GLOBAL_REFRESH_UI};



// used to tally up the number of times a anime has been recommended
#[derive(Debug, Clone, Default)]
struct RecommendTally {
    pub id: i32,
    pub rating: f32,
}



// returns a list of anime ids of recommended shows generated from the users completed list, recommendation mode, and filters
pub async fn recommendations(mode: String, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) -> Vec<i32> {

    let mut recommend_tally = if mode == "user_recommended" {
        tally_recommendations().await
    } else {
        related_recommendations(mode).await
    };

    // if any filter is used
    filter_anime(&mut recommend_tally, genre_filter,year_min_filter, year_max_filter, format_filter).await;

    // reduce number of shows so the ui isn't overloaded with a large number of shows
    recommend_tally.truncate(100);
    // only keep anime ids, no other information is needed
    let recommend_list: Vec<_> = recommend_tally.iter().map(|anime| anime.id).collect();
    recommend_list
}



// return a list of anime recommended by users based on the users completed list
async fn tally_recommendations() -> Vec<RecommendTally> {

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await; // used to remove anime the user has already watched

    // use the completed list to grab user recommendations
    let mut list = GLOBAL_USER_ANIME_LISTS.lock().await;
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await; // uses the user score to modify the recommended rating

    if list.contains_key("COMPLETED") == false {

        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), 
            "COMPLETED".to_owned(), 
            GLOBAL_TOKEN.lock().await.access_token.clone(),
            &mut user_data,
            &mut list).await;

        if error_message.is_some() {
            println!("api_calls::anilist_get_list for completed list returned a error: {}", error_message.unwrap());
            return Vec::new();
        }

        file_operations::write_file_anime_info_cache(&anime_data);
    }
    let completed_list = list.get("COMPLETED").unwrap();

    let score_format = GLOBAL_USER_SETTINGS.lock().await.score_format.clone(); // used to properly convert score into a modifier

    // tally up all recommendations and combine the ratings for the same show
    let mut recommend_total: HashMap<i32, f32> = HashMap::new();
    for id in completed_list {

        if let Some(anime_entry) = anime_data.get(id) {

            if let Some(recommendations) = &anime_entry.recommendations {

                for rec in &recommendations.nodes {
        
                    // media might be null, it may have been removed after the recommendation was made
                    if let Some(recommendation) = &rec.media_recommendation {

                        let score_modifier = if user_data.contains_key(&id) == false {
                            // anime is in completed list but has no user data, this is a bug
                            println!("Missing: {}", id);
                            1.0
                        } else {
                            score_to_rating_modifier(user_data.get(&id).unwrap().score, &score_format)
                        };
            
                        // add the recommendation to the list or add the rating to the existing recommendation
                        recommend_total.entry(recommendation.id).and_modify(|r| { *r += (rec.rating as f32) * score_modifier }).or_insert((rec.rating as f32) * score_modifier);
                    }
                }
            }
        } else {
            println!("Anime({}) in completed list is missing from anime_data", id);
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
    match api_calls::anilist_api_call_multiple(unknown_ids, &mut anime_data).await {
        Ok(_result) => {
            file_operations::write_file_anime_info_cache(&anime_data);
            GLOBAL_REFRESH_UI.lock().await.no_internet = false;
        },
        Err(_error) => GLOBAL_REFRESH_UI.lock().await.no_internet = true,
    }

    recommendations
}


// remove anime from anime_list if it does not match the filters
async fn filter_anime(anime_list: &mut Vec<RecommendTally>, genre_filter: String, year_min_filter: i32, year_max_filter: i32, format_filter: String) {

    if genre_filter.is_empty() == false || year_min_filter != 0 || year_max_filter != 0 || format_filter.is_empty() == false {

        let anime_data = GLOBAL_ANIME_DATA.lock().await;

        // filter out any show which doesn't match the genre
        if genre_filter.is_empty() == false {

            anime_list.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().genres.contains(&genre_filter)
            })
        }

        // filter out any show which is too old
        if year_min_filter != 0 {

            anime_list.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().season_year.is_some() &&
                anime_data.get(&rec.id).unwrap().season_year.unwrap() >= year_min_filter
            })
        }

        // filter out any show which is too new
        if year_max_filter != 0 {

            anime_list.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().season_year.is_some() &&
                anime_data.get(&rec.id).unwrap().season_year.unwrap() <= year_max_filter
            })
        }

        // filter out any show which doesn't match the format
        if format_filter.is_empty() == false {

            anime_list.retain(|rec| { 
                anime_data.get(&rec.id).unwrap().format.is_some() &&
                anime_data.get(&rec.id).unwrap().format.as_ref().unwrap().eq(&format_filter)
            })
        }
    }
}



/// convert the score to a rating modifier. 
/// higher scores will produce a higher modifier so shows that a user liked will be worth more than a show the user disliked
fn score_to_rating_modifier(score: f32, score_format: &Option<String>) -> f32 {

    // show has no score
    if score == 0.0 {
        return 1.0;
    }

    if score_format.is_none() {
        return 1.0;
    }

    // convert to modifier
    match score_format.as_ref().unwrap().as_str() {
        "POINT_100" => score / 50.0,
        "POINT_10_DECIMAL" => score / 5.0,
        "POINT_10" => score / 5.0,
        "POINT_5" => score / 2.5,
        "POINT_3" => score - 1.0,
        _ => 1.0,
    }
}


// create a list of recommended anime based on relations to anime in the completed list
async fn related_recommendations(mode: String) -> Vec<RecommendTally> {

    let mut anime_data = GLOBAL_ANIME_DATA.lock().await; // used to find sequels
    // retrieve completed list, if it does not exist, retrieve it from anilist
    let mut user_data = GLOBAL_USER_ANIME_DATA.lock().await; // used to remove anime the user has already watched and uses the user score to modify the recommended rating
    let mut list = GLOBAL_USER_ANIME_LISTS.lock().await;
    if list.contains_key("COMPLETED") == false {
        let error_message = api_calls::anilist_get_list(GLOBAL_USER_SETTINGS.lock().await.username.clone(), "COMPLETED".to_owned(), GLOBAL_TOKEN.lock().await.access_token.clone(), &mut user_data, &mut list).await;
        if error_message.is_some() {
            println!("api_calls::anilist_get_list for completed list returned a error: {}", error_message.unwrap());
            return Vec::new();
        }
        file_operations::write_file_anime_info_cache(&anime_data);
        file_operations::write_file_user_info().await;
    }
    let completed_list = list.get("COMPLETED").unwrap();

    let score_format = GLOBAL_USER_SETTINGS.lock().await.score_format.clone(); // used to properly convert score into a modifier

    // create a list of all recommended anime from anime on the completed list, include score modifier so higher rated shows get recommended more highly
    let mut recommend_total: HashMap<i32, f32> = HashMap::new();
    for id in completed_list {

        if let Some(anime_entry) = &anime_data.get(id) {

            let score_modifier = if let Some(anime) = user_data.get(&id) {
                score_to_rating_modifier(anime.score, &score_format)
            } else {
                println!("Anime({}) in completed list is missing from user_data", id);
                1.0 // should never happen, anime in completed list should also be in user data
            };

            for related in &anime_entry.relations.edges {

                // filter by relation type
                if related.relation_type != mode {
                    continue;
                }

                // add anime id and score modifier or replace score modifier if the new modifier is higher
                if recommend_total.contains_key(&related.node.id) == false {

                    recommend_total.insert(related.node.id, score_modifier);
                } else if score_modifier > *recommend_total.get(&related.node.id).unwrap() {
                    
                    recommend_total.entry(related.node.id).and_modify(|entry| *entry = score_modifier);
                }
            }
        } else {
            println!("Anime({}) in completed list is missing from anime_data", id);
        }
    }
    
    // remove anime already in the users lists
    recommend_total.retain(|id, _| { user_data.contains_key(&id) == false || user_data.get(&id).unwrap().status == "" });

    // find shows that are missing data
    let unknown_ids: Vec<i32> = recommend_total.iter()
        .map(|(id,_)| *id)
        .filter(|id| anime_data.contains_key(id) == false)
        .collect();
    
    // get information on any show which is missing
    match api_calls::anilist_api_call_multiple(unknown_ids, &mut anime_data).await {
        Ok(_result) => {
            file_operations::write_file_anime_info_cache(&anime_data);
            GLOBAL_REFRESH_UI.lock().await.no_internet = false;
        },
        Err(_error) => GLOBAL_REFRESH_UI.lock().await.no_internet = true,
    }

    // some ids lead to 404 pages, these ids won't be in anime_data, remove them
    recommend_total.retain(|anime_id, _| { anime_data.contains_key(anime_id) == true });
    
    // sum up the number of recommendations for the anime and apply score modifier
    let mut recommendations: Vec<RecommendTally> = Vec::new();
    for (anime_id, score_modifier) in recommend_total {
        
        if let Some(anime_entry) = anime_data.get(&anime_id) {
    
            if let Some(anime_recommendations) = &anime_entry.recommendations {
    
                let score: i32 = anime_recommendations.nodes.iter().map(|r| r.rating).sum();

                recommendations.push(RecommendTally { id: anime_id, rating: score as f32 * score_modifier });
            }
        }
    };

    // sort by most recommended
    recommendations.sort_by(| entry_a, entry_b | {
        entry_b.rating.partial_cmp(&entry_a.rating).unwrap()
    });

    recommendations
}