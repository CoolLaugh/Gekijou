


// open the info window to the edit user info tab
window.show_anime_info_window_edit = show_anime_info_window_edit;
async function show_anime_info_window_edit(anime_id) {
  await show_anime_info_window(anime_id);
  openTab('user_entry', 'underline_tab_1');
}



// open the info window to the edit user info tab
window.show_anime_info_window_trailer = show_anime_info_window_trailer;
async function show_anime_info_window_trailer(anime_id) {
  await show_anime_info_window(anime_id);
  openTab('trailer', 'underline_tab_2')
}


var list_ids = [];
var info_window_anime_id = 0;
// show information window populated with the shows info
window.show_anime_info_window = show_anime_info_window;
async function show_anime_info_window(anime_id) {
    
  info_window_anime_id = anime_id;

  // retrieve necessary information
  var user_settings = await invoke("get_user_settings");
  var info = await invoke("get_anime_info", {id: anime_id});
  var title = await determine_title(info.title, user_settings);

  // fill in info window with data
  add_anime_data(info, title, user_settings.show_spoilers);
  await add_user_data(anime_id, user_settings);
  add_trailer(info.trailer);
  add_related_anime(info.relations.edges, info.recommendations.nodes, user_settings.title_language);
  var table = document.getElementById("torrent_table");
  var rows = table.rows.length - 1;
  for(var i = 0; i < rows; i++) {
    table.deleteRow(1);
  }

  var index = list_ids.indexOf(anime_id);
  var index_previous = (index - 1) % list_ids.length;
  if (index == 0) {
    index_previous = list_ids.length - 1;
  }
  var index_next = (index + 1) % list_ids.length;

  document.getElementById("info_window_previous").setAttribute("onclick", "show_anime_info_window(" + list_ids[index_previous] + ")");
  document.getElementById("info_window_next").setAttribute("onclick", "show_anime_info_window(" + list_ids[index_next] + ")");

  // make the window visible
  openTab('information', 'underline_tab_0');
  document.getElementById("info_panel").style.display = "block";
  document.getElementById("cover_panel_grid").style.opacity = 0.3;
}



// fill in data about the selected anime into the info window
function add_anime_data(info, title, show_spoilers) {

    // text strings for parts that are more complicated than a simple assignment
    var studio_name = "";
    var anime_format = "";
    var episode_text = "";
    var date = "";

    // determine the name of the main studio
    if (info.studios.nodes.length == 0 || info.studios.nodes[0].name == null) {
        studio_name = "Unknown Studio";
    } else {
        studio_name = info.studios.nodes[0].name;
    }

    // determine the number of episodes and length of each episode
    if (info.episodes == null) {
        episode_text = "?? x "
    } else if (info.episodes > 1) {
        episode_text = info.episodes + " x "
    }
    episode_text += null_check(info.duration, info.duration + " Minutes", "?? Minutes");

    // determine the format of the anime(TV, Movie, OVA, etc)
    if (info.format == null) {
        anime_format = "Unknown Format";
    } else {
        if (info.format != "TV") {
            // capitalize the first letter
            anime_format = info.format.charAt(0) + info.format.toLowerCase().slice(1);
        } else {
            anime_format = info.format;
        }
    }

    // determine which season and year the show will air in
    if (info.season != null) {
        // capitalize the first letter
        date = info.season.charAt(0) + info.season.toLowerCase().slice(1) + " " + info.season_year; 
    } else {
        date = "Unknown Date";
    }

    // list all genres
    var genres_text = "";
    for (var i = 0; i < info.genres.length; i++) {
        genres_text += info.genres[i];
        if (i != info.genres.length - 1) {
        genres_text += ", ";
        }
    }
  
    // list all tags
    var tags = "";
    for (var i = 0; i < info.tags.length; i++) {
        // don't list tags that are marked as spoiler if the user doesn't want to see them
        if (show_spoilers == false && (info.tags[i].is_general_spoiler || info.tags[i].is_media_spoiler)) {
            continue;
        }
        tags += info.tags[i].name;
        if (i != info.tags.length - 1) {
            tags += ", ";
        }
    }
  
    // populate window with the anime's information
    document.getElementById("info_title").textContent = title;
    document.getElementById("info_cover").src = info.cover_image.large;
    document.getElementById("info_cover").setAttribute("onclick", "open_window(\"https://anilist.co/anime/" + info.id + "\")");
    document.getElementById("studio").innerHTML = studio_name;
    document.getElementById("info_description").innerHTML = info.description;
    document.getElementById("info_format").textContent = anime_format;
    document.getElementById("info_rating").textContent = null_check(info.average_score, info.average_score + "%", "No Score");
    document.getElementById("info_duration").textContent = episode_text;
    document.getElementById("info_season_year").textContent = date;
    document.getElementById("info_genres").textContent = "Genres: " + genres_text;
    document.getElementById("info_tags").textContent = "Tags: " + tags;
}

// fill in the user's data into the info window
async function add_user_data(anime_id, user_settings) {

    var user_data = await invoke("get_user_info", {id: anime_id});

    document.getElementById("delete_anime").onclick = function() { confirm_delete_entry(user_data.id, user_data.media_id); }
    document.getElementById("status_select").value = user_data.status;
    document.getElementById("episode_number").value = user_data.progress;
    setup_score_dropdown(user_settings.score_format);
    document.getElementById("score_dropdown").value = user_data.score;
    document.getElementById("started_date").value = null_check_date_string(user_data.started_at, "");
    document.getElementById("finished_date").value = null_check_date_string(user_data.completed_at, "");
    document.getElementById("info_close_button").onclick = function() { hide_anime_info_window(anime_id)};
}

// add the trailer if it exists or hide the trailer tab if it doesn't
function add_trailer(trailer) {

    if(trailer != null && trailer.site == "youtube") {
        document.getElementById("trailer_button").style.display = "block";
        document.getElementById("youtube_embed").src = "https://www.youtube.com/embed/" + trailer.id;
    } else {
        // trailer does not exist, hide the tab
        document.getElementById("trailer_button").style.display = "none";
    }
}

// hide information window and return to cover grid
window.hide_anime_info_window = hide_anime_info_window;
async function hide_anime_info_window(anime_id) {
  document.getElementById("youtube_embed").src = "";
  document.getElementById("info_panel").style.display = "none";
  document.getElementById("cover_panel_grid").style.opacity = 1;
  if (anime_id != null) {
    var refresh = await update_user_entry(anime_id);
    if (refresh == true && current_tab != "BROWSE") {
      show_anime_list(current_tab);
    }
  }
}



// updates the entry for the current anime with new information from the info window
async function update_user_entry(anime_id) {

  var user_data = await invoke("get_user_info", {id: anime_id});

  // grab data from ui
  var user_entry = {
    'id': user_data.id,
    'media_id': anime_id,
    'status': document.getElementById("status_select").value,
    'score': parseFloat(document.getElementById("score_dropdown").value),
    'progress': parseInt(document.getElementById("episode_number").value)
  };

  switch(document.getElementById("score_dropdown").getAttribute("format")) {
    case "POINT_100":
      if (user_entry.score < 0) { user_entry.score = user_entry.score * -1 }
      if (user_entry.score > 100) { user_entry.score = 100 }
      break;
    case "POINT_10_DECIMAL":
      if (user_entry.score < 0) { user_entry.score = user_entry.score * -1 }
      if (user_entry.score > 10) { user_entry.score = 10 }
      break;
    case "POINT_10":
    case "POINT_5":
    case "POINT_3":
      break;
  }

  // fill in start date
  var started = document.getElementById("started_date").value.split("-");
  if (started.length == 3) {
    user_entry.started_at = {year: parseInt(started[0]), month: parseInt(started[1]), day: parseInt(started[2])};
  } else {
    user_entry.started_at = {year: null, month: null, day: null};
  }

  // fill in finished date
  var finished = document.getElementById("finished_date").value.split("-");
  if (finished.length == 3) {
    user_entry.completed_at = {year: parseInt(finished[0]), month: parseInt(finished[1]), day: parseInt(finished[2])};
  } else {
    user_entry.completed_at = {year: null, month: null, day: null};
  }

  // only update if something changed
  if (user_entry.status != user_data.status ||
    user_entry.score != user_data.score ||
    user_entry.progress != user_data.progress ||
    user_entry.started_at.year != user_data.started_at.year ||
    user_entry.started_at.month != user_data.started_at.month ||
    user_entry.started_at.day != user_data.started_at.day ||
    user_entry.completed_at.year != user_data.completed_at.year ||
    user_entry.completed_at.month != user_data.completed_at.month ||
    user_entry.completed_at.day != user_data.completed_at.day) {

      await invoke("update_user_entry", {anime: user_entry});
  }

  if (user_entry.progress != user_data.progress) {
    
    var text = document.getElementById("episode_text_"+ anime_id);
    var total = text.textContent.split('/')[1];

    text.textContent = user_entry.progress + "/" + total;

    draw_episode_canvas(user_entry.progress, total, anime_id);
  }

  // return true if the status has changed and the list needs to be refreshed
  return user_entry.status != user_data.status;
}



// determine which language to use for the title based on the users settings and which titles exist
async function determine_title(title_struct, user_settings) {

    // get the users language preference
    if (user_settings == null) {
        title_language = await invoke("get_user_settings").title_language;
    }

    var title = null;
    // try to use the language the user chose
    if (title_language == "romaji" && title_struct.romaji != null) {
        title = title_struct.romaji;
    } else if (title_language == "english" && title_struct.english != null) {
        title = title_struct.english;
    } else if (title_language == "native" && title_struct.native != null) {
        title = title_struct.native;
    }
    // if the preferred language does not exist use another language
    if (title == null) {
        title = null_check(title_struct.romaji, title_struct.romaji, null_check(title_struct.english, title_struct.english, title_struct.native));
    }

    return title;
}



// change the score input to match the user's score format
function setup_score_dropdown(format) {
  switch(format) {
    case "POINT_100":
      document.getElementById("score_cell").innerHTML = "<input id=\"score_dropdown\" format=\"" + format + "\" min=\"0\" max=\"100\" step=1 type=\"number\">";
      break;
    case "POINT_10_DECIMAL":
      document.getElementById("score_cell").innerHTML = "<input id=\"score_dropdown\" format=\"" + format + "\" min=\"0.0\" max=\"10.0\" step=0.1 type=\"number\">";
      break;
    case "POINT_10":
      document.getElementById("score_cell").innerHTML = "<select id=\"score_dropdown\" format=\"" + format + "\" name=\"score_select\"><option value=\"0\">No Score</option><option value=\"1\">1</option><option value=\"2\">2</option><option value=\"3\">3</option><option value=\"4\">4</option><option value=\"5\">5</option><option value=\"6\">6</option><option value=\"7\">7</option><option value=\"8\">8</option><option value=\"9\">9</option><option value=\"10\">10</option></select>";
      break;
    case "POINT_5":
      document.getElementById("score_cell").innerHTML = "<select id=\"score_dropdown\" format=\"" + format + "\" name=\"score_select\"><option value=\"0\">No Score</option><option value=\"1\">★☆☆☆☆</option><option value=\"2\">★★☆☆☆</option><option value=\"3\">★★★☆☆</option><option value=\"4\">★★★★☆</option><option value=\"5\">★★★★★</option></select>";
      break;
    case "POINT_3":
      document.getElementById("score_cell").innerHTML = "<select id=\"score_dropdown\" format=\"" + format + "\" name=\"score_select\"><option value=\"0\">No Score</option><option value=\"1\">🙁</option><option value=\"2\">😐</option><option value=\"3\">🙂</option></select>";
      break;
  }
}



// add related(sequel, prequel, etc) and recommended titles to the related tab
function add_related_anime(related, recommendations, title_language) {

    // the element that will house related shows
    var related_grid = document.getElementById("related_grid");
    // cleanup from the last show that used the info window
    removeChildren(related_grid);

    // add each related anime
    for(var i = 0; i < related.length; i++) {

        // determine which title to use
        var title = related[i].node.title.romaji;
        if (title_language == "english" && related[i].node.title.english != null) {
            title = related[i].node.title.english;
        } else if (title_language == "native") {
            title = related[i].node.title.native;
        }
        // capitalize the first letter
        var relation_type = related[i].relation_type.charAt(0) + related[i].relation_type.toLowerCase().slice(1);
        relation_type.replace("_", " ");

        var href = " href=\"#\"";
        var onclick = " onclick=\"show_anime_info_window(" + related[i].node.id + ")\"";
        // don't allow clicking for manga sources, the info window is only designed for anime
        if (relation_type == "Adaptation") {
            onclick = "";
            href = "";
        }

        // add the show to the grid
        var html = "";
        html +=  "<div style=\"width: 116px; text-align: center; background: var(--background-color1); position: relative;\">"
        html +=    "<a" + href + "><img class=image href=\"#\" height=\"174px\" src=\"" + related[i].node.cover_image.large + "\" width=\"116px\"" + onclick + "></a>"
        html +=    "<div style=\"height: 49px; overflow: hidden; margin-top: -5px;\"><a" + href + "><p" + onclick + ">" + title + "</p></a></div>"
        html +=    "<div class=\"related_category\"><p style=\"color: #f6f6f6;\">" + relation_type + "</p></div>"
        html +=  "</div>"

        related_grid.innerHTML += html;
    }

    // the element that will house recommended shows
    var recommended_grid = document.getElementById("recommended_grid");
    // cleanup from the last show that used the info window
    removeChildren(recommended_grid);

    // sort by number of people recommending the show
    recommendations.sort(function(a,b) {
        return b.rating-a.rating;
    });

    // add each recommendation
    for(var i = 0; i < recommendations.length; i++) {

        // show might have been removed?
        if (recommendations[i].media_recommendation == null) {
            continue;
        }

        // determine title language
        var title = recommendations[i].media_recommendation.title.romaji;
        if (title_language == "english" && recommendations[i].media_recommendation.title.english != null) {
        title = recommendations[i].media_recommendation.title.english;
        } else if (title_language == "native") {
        title = recommendations[i].media_recommendation.title.native;
        }

        // add the show to the grid
        var html = "";
        html +=  "<div style=\"width: 116px; text-align: center; background: var(--background-color1);\">"
        html +=    "<a href=\"#\"><img class=image height=\"174px\" src=\"" + recommendations[i].media_recommendation.cover_image.large + "\" width=\"116px\" onclick=\"show_anime_info_window(" + recommendations[i].media_recommendation.id + ")\"></a>"
        html +=    "<div style=\"height: 49px; overflow: hidden; margin-top: -5px;\"><a href=\"#\"><p onclick=\"show_anime_info_window(" + recommendations[i].media_recommendation.id + ")\">" + title + "</p></a></div>"
        html +=  "</div>"

        recommended_grid.innerHTML += html;
    }
}


var rss_data;
async function add_torrent_data(anime_id) {

  size_sorted = false;
  downloads_sorted = false;

  rss_data = await invoke("get_torrents", {id: anime_id});

  var sub_groups = [];
  var resolutions = [];

  for(var i = 0; i < rss_data.length; i++) {

    if (sub_groups.includes(rss_data[i].derived_values.sub_group) == false) {
      sub_groups.push(rss_data[i].derived_values.sub_group);
    }

    if (resolutions.includes(rss_data[i].derived_values.resolution) == false) {
      resolutions.push(rss_data[i].derived_values.resolution);
    }
  }

  sub_groups.sort();
  resolutions.sort(function(a,b) {
    return b-a;
  });

  var sub_group_filter_select = document.getElementById("sub_group_filter");
  removeChildren(sub_group_filter_select);
  sub_group_filter_select.insertAdjacentHTML("beforeend", "<option>Any</option>");
  for(var i = 0; i < sub_groups.length; i++) {
    sub_group_filter_select.insertAdjacentHTML("beforeend", "<option>" + sub_groups[i] + "</option>");
  }

  var resolution_filter_select = document.getElementById("resolution_filter");
  removeChildren(resolution_filter_select);
  resolution_filter_select.insertAdjacentHTML("beforeend", "<option>Any</option>");
  for(var i = 0; i < resolutions.length; i++) {
    if(resolutions[i] == 0) {
      resolution_filter_select.insertAdjacentHTML("beforeend", "<option>Unknown</option>");
    } else {
      resolution_filter_select.insertAdjacentHTML("beforeend", "<option value=" + resolutions[i] + ">" + resolutions[i] + "p</option>");
    }
  }

  document.getElementById("episode_filter").value = "Any";

  var table = document.getElementById("torrent_table");
  for(var i = 0; i < rss_data.length; i++) {

    add_torrent_row(table,rss_data[i]);
  }
}



var size_sorted = false;
var downloads_sorted = false;
window.filter_sort_torrents = filter_sort_torrents;
function filter_sort_torrents(sort_category) {

  var table = document.getElementById("torrent_table");

  var rows = table.rows.length - 1;
  for(var i = 0; i < rows; i++) {
    table.deleteRow(1);
  }

  var sub_group_filter = document.getElementById("sub_group_filter").value;
  var resolution_filter = document.getElementById("resolution_filter").value;
  var episode_filter = document.getElementById("episode_filter").value;

  if (sort_category == 1) {
    downloads_sorted = false;
    if (size_sorted == true) {
      rss_data.sort(function(a,b) {
        return b.size-a.size;
      });
    } else {
      rss_data.sort(function(a,b) {
        return a.size-b.size;
      });
    }
    size_sorted = !size_sorted;
  } else if (sort_category == 2) {
    size_sorted = false;
    if (downloads_sorted == true) {
      rss_data.sort(function(a,b) {
        return b.downloads-a.downloads;
      });
    } else {
      rss_data.sort(function(a,b) {
        return a.downloads-b.downloads;
      });
    }
    downloads_sorted = !downloads_sorted;
  }

  for(var i = 0; i < rss_data.length; i++) {

    if (sub_group_filter != "Any") {
      if (sub_group_filter != rss_data[i].derived_values.sub_group){
        continue;
      }
    }

    if (resolution_filter != "Any") {
      if (resolution_filter != rss_data[i].derived_values.resolution){
        continue;
      }
    }

    if (episode_filter != "Any") {
      if (rss_data[i].derived_values.batch == false) {
        continue;
      }
    }

    add_torrent_row(table, rss_data[i]);
  }
}

function add_torrent_row(table, rss_entry) {

  var row = table.insertRow(1);

  var download_link_cell = row.insertCell(0);
  download_link_cell.innerHTML = "<a title=\"" + rss_entry.title + "\" href=\"magnet:?xt=urn:btih:" + rss_entry.info_hash + "&dn=" + rss_entry.title + "&tr=http%3A%2F%2Fnyaa.tracker.wf%3A7777%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337%2Fannounce&tr=udp%3A%2F%2Fexodus.desync.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce\">⤓</a>";

  var sub_group_cell = row.insertCell(1);
  sub_group_cell.innerHTML = rss_entry.derived_values.sub_group;

  var title_cell = row.insertCell(2);
  title_cell.innerHTML = rss_entry.derived_values.title;

  var episode_cell = row.insertCell(3);
  if (rss_entry.derived_values.batch == false) {
    episode_cell.innerHTML = "Ep " + rss_entry.derived_values.episode;
  } else {
    episode_cell.innerHTML = "Batch";
  }

  var resolution_cell = row.insertCell(4);
  if (rss_entry.derived_values.resolution == 0) {
    resolution_cell.innerHTML = "Unknown";
  } else {
    resolution_cell.innerHTML = rss_entry.derived_values.resolution + "p";
  }

  var size_cell = row.insertCell(5);
  size_cell.innerHTML = rss_entry.size_string;

  var downloads_cell = row.insertCell(6);
  downloads_cell.innerHTML = rss_entry.downloads + " dl";

}


// changes the opened tab in the anime info window
window.openTab = openTab;
function openTab(tab_name, underline_name) {

  // Get all elements with class="tab_content" and hide them
  var tab_content = document.getElementsByClassName("tab_content");
  for (var i = 0; i < tab_content.length; i++) {
    tab_content[i].style.display = "none";
  }

  // Get all elements with class="tab_underline" and hide them
  var tab_content = document.getElementsByClassName("tab_underline");
  for (var i = 0; i < tab_content.length; i++) {
    tab_content[i].style.visibility = "hidden";
  }

  // Show the current tab, and an underline to the button that opened the tab
  if (tab_name == "related"){
    document.getElementById(tab_name).style.display = "grid"; 
  } else {
    document.getElementById(tab_name).style.display = "block";
  }
  document.getElementById(underline_name).style.visibility = "visible";
}

window.openTab = openTab;
function open_torrents_tab(tab_name, underline_name){
  openTab(tab_name, underline_name);
  add_torrent_data(info_window_anime_id);
}



// determine which value to use based on if the first value is null or not
function null_check(null_check, not_null_value, null_value) {
  if (null_check == null) {
    return null_value;
  } else {
    return not_null_value;
  }
}



// turn the date into a sortable number while skipping null values
function null_check_date(null_check_date) {
  var date = 0;
  if (null_check_date != null) {
    if(null_check_date.year != null) {
      date += null_check_date.year * 10000;
    }
    if(null_check_date.month != null) {
      date += null_check_date.month * 100;
    }
    if(null_check_date.day != null) {
      date += null_check_date.day;
    }
  }
  return date;
}



// create a date string while handling null values
function null_check_date_string(date, null_value) {
  if (date == null) { 
    return null_value;
  }
  var date_string = "";
  if (date.year != null) {
    date_string += date.year;
  }
  if (date.month != null) {
    date_string +=  "-" + String(date.month).padStart(2,'0');
  }
  if (date.day != null) {
    date_string +=  "-" + String(date.day).padStart(2,'0');
  }
  return date_string;
}



// remove all html children of the current element.  used to clear the anime list on screen
const removeChildren = (parent) => {
  while (parent.lastChild) {
      parent.removeChild(parent.lastChild);
  }
};