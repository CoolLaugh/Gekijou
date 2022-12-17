const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", () => {
  invoke("on_startup");
  get_user_settings();
  populate_year_dropdown();

  document.getElementById("information").style.display = "block";
  document.getElementById("underline_tab_0").style.visibility = "visible";
  invoke("anime_update_delay_loop");
  check_for_refresh_ui();
});

window.get_user_settings = get_user_settings;
async function get_user_settings() {
  
  var user_settings = await invoke("get_user_settings");
  
  document.getElementById("user_name").value = user_settings.username;
  document.getElementById("title_language").value = user_settings.title_language;
  document.getElementById("show_spoiler_tags").checked = user_settings.show_spoilers;
  document.getElementById("show_adult").checked = user_settings.show_adult;
  var folder_textarea = document.getElementById("folders");
  folder_textarea.value = "";
  for(var i = 0; i < user_settings.folders.length; i++){
    folder_textarea.value += user_settings.folders[i];
    if(i + 1 != user_settings.folders.length) {
      folder_textarea.value += "\n";
    }
  }
}

// add every year between next year and 1940 to the year dropdown
async function populate_year_dropdown() {

  let year =  new Date().getFullYear();
  year += 1;
  for(var i = year; i >= 1940; i--) {
    document.getElementById("year_select").insertAdjacentHTML("beforeend", "<option value=\"" + i + "\">" + i + "</option>");
  }
}

// check if rust has detected a episode and increased the users progress
async function check_for_refresh_ui() {

  while (true) {

    var refresh = await invoke("get_refresh_ui");
    if (refresh == true) {
      show_anime_list(current_tab);
    }
  }
}

// confirm the user wants to delete an anime and then delete it
window.confirm_delete_entry = confirm_delete_entry;
async function confirm_delete_entry(id, media_id) {

  // await warning is a lie, don't remove await
  if (await confirm('This will remove all data about this anime from your list. Are you sure?') == true) {
    
    var removed = await invoke("remove_anime", { id: id, mediaId: media_id});
    if (removed == true) {

      show_anime_list(current_tab);
      document.getElementById("status_select").value = "";
      document.getElementById("episode_number").value = 0;
      document.getElementById("score_0to5").value = "0";
      document.getElementById("started_date").value = "";
      document.getElementById("finished_date").value = "";
    }
  }
}

// open another window for the user to log in and get a code they can copy and paste
window.open_oauth_window = open_oauth_window;
async function open_oauth_window() {
  window.open("https://anilist.co/api/v2/oauth/authorize?client_id=9965&redirect_uri=https://anilist.co/api/v2/oauth/pin&response_type=code");
}

// takes the oauth code and uses it to get a access token for editing the users list
window.get_oauth_token = get_oauth_token;
async function get_oauth_token() {
  
  var input = document.getElementById("oauth_code")
  
  var success = await invoke("anilist_oauth_token", { code: document.getElementById("oauth_code").value});

  input.value = "";
  if(success[0] == true) {
    input.setAttribute("placeholder", "Success");
  } else {
    input.setAttribute("placeholder", "Failed");
    alert(success[1]);
  }
}

// show the users watching list
var current_tab = "";
window.show_watching_anime = show_watching_anime;
async function show_watching_anime() {
  current_tab = "CURRENT";
  show_anime_list(current_tab);
  exclusive_underline(0);
}

// show the users watching list
window.show_completed_anime = show_completed_anime;
async function show_completed_anime() {
  current_tab = "COMPLETED";
  show_anime_list(current_tab);
  exclusive_underline(1);
}

// show the users paused list
window.show_paused_anime = show_paused_anime;
async function show_paused_anime() {
  current_tab = "PAUSED";
  show_anime_list(current_tab);
  exclusive_underline(2);
}

// show the users dropped list
window.show_dropped_anime = show_dropped_anime;
async function show_dropped_anime() {
  current_tab = "DROPPED";
  show_anime_list(current_tab);
  exclusive_underline(3);
}

// show the users plan to watch list
window.show_planning_anime = show_planning_anime;
async function show_planning_anime() {
  current_tab = "PLANNING";
  show_anime_list(current_tab);
  exclusive_underline(4);
}

// show the controls to allow the user to look for anime based on year, season, genre, and format
window.show_browse_anime = show_browse_anime;
async function show_browse_anime() {
  current_tab = "BROWSE";
  exclusive_underline(5);
  document.getElementById("browse_filters").style.display = "block";
  removeChildren(document.getElementById("cover_panel_grid"));
}

// shows the settings window
window.show_setting_window = show_setting_window;
async function show_setting_window() {
  document.getElementById("login_panel").style.visibility = "visible";
  document.getElementById("cover_panel_grid").style.opacity = 0.3;
}

// hide all underlines except one to show the current list being shown
function exclusive_underline(index) {

  for(var i = 0; i < 6; i++) {
    document.getElementById("underline" + i).style.visibility = "hidden";
  }
  document.getElementById("underline" + index).style.visibility = "visible";
}

// fill the UI with anime based on the list selected
window.show_anime_list = show_anime_list;
async function show_anime_list(name) {
  
  document.getElementById("browse_filters").style.display = "none";
  var watching = await invoke("get_list", { listName: name });
  console.log(watching);
  var user_data = await invoke("get_list_user_info", { listName: name });
  // get user data on anime
  console.log(user_data);
  var user_settings = await invoke("get_user_settings");

  // add anime to UI
  removeChildren(document.getElementById("cover_panel_grid"));

  for(var i = 0; i < watching.length; i++) {
    if(user_settings.show_adult == false && watching[i].is_adult == true) {
      continue;
    }
    add_anime(watching[i], user_data[i], i);
  }

  sort_anime();
}

// remove all html children of the current element.  used to clear the anime list on screen
const removeChildren = (parent) => {
  while (parent.lastChild) {
      parent.removeChild(parent.lastChild);
  }
};

// list of categories that can be searched by
// variables are field name, display name, and default sorting order
const sort_categories = [["name", "Alphabetical", true], ["score","Score", false], ["date","Date", true], ["popularity","Popularity", false]];
var sort_category_index = 0;
var sort_ascending = true;

// cycle through different ways of sorting shows
window.change_sort_type = change_sort_type;
async function change_sort_type() {

  sort_category_index = (sort_category_index + 1) % sort_categories.length;
  sort_ascending = sort_categories[sort_category_index][2];

  document.getElementById("sort_order_text").textContent = sort_categories[sort_category_index][1];

  change_ascending_indicator()

  console.log(sort_category_index);
  console.log(sort_categories[sort_category_index]);

  if (current_tab == "BROWSE") {
    browse_update();
  } else {
    sort_anime();
  }
}

// change between sorting ascending and descending
window.change_sort_ascending = change_sort_ascending;
async function change_sort_ascending() {
  sort_ascending = !sort_ascending;
  change_ascending_indicator()
  if (current_tab == "BROWSE") {
    browse_update();
  } else {
    sort_anime();
  }
}

// change the image to show if the list is being sorted ascending or descending
function change_ascending_indicator() {
  if(sort_ascending == true) {
    document.getElementById("sort_order_ascending").style.transform = 'rotate(180deg)';
    document.getElementById("sort_order_ascending").order = "AES";
  }
  else {
    document.getElementById("sort_order_ascending").style.transform = 'rotate(0deg)';
    document.getElementById("sort_order_ascending").order = "DESC";
  }
}

// sort covers according to the current category and order
window.sort_anime = sort_anime;
async function sort_anime() {

  var container = document.getElementById("cover_panel_grid");
  var elements = container.childNodes;
  var sortMe = [];

  for (var i=0; i<elements.length; i++) {
      
    if (elements[i].nodeType == 1) {

      switch(sort_category_index) {
        case 0:
          sortMe.push([ elements[i].getAttribute("title").toLowerCase() , elements[i] ]);
          break;
        case 1:
          sortMe.push([ parseInt(elements[i].getAttribute("score"), 10) , elements[i] ]);
          break;
        case 2:
          sortMe.push([ parseInt(elements[i].getAttribute("date"), 10) , elements[i] ]);
          break;
        case 3:
          sortMe.push([ parseInt(elements[i].getAttribute("popularity"), 10) , elements[i] ]);
          break;
      }
    }
  }

  switch(sort_category_index) {
    case 0:
      sortMe.sort();
      break;
    case 1: // intentional fall through
    case 2: // intentional fall through
    case 3: // intentional fall through
      sortMe.sort(function(a,b) {
        return a[0]-b[0];
      });
      break;
  }

  if (sort_ascending == false) {
    sortMe.reverse();
  }

  for (var i=0; i<sortMe.length; i++) {
      container.appendChild(sortMe[i][1]);
  }
}

// add an anime to the ui
window.add_anime = add_anime;
async function add_anime(anime, user_data, cover_id) {

  var title = "No Title";
  if (anime.title != null) {
    if(anime.title.english != null){
      title = anime.title.english;
    } else if (anime.title.romaji != null) {
      title = anime.title.romaji;
    } else if (anime.title.native != null) {
      title = anime.title.native;
    }
  }

  var watch_percent = 0;
  var episode_text = "";
  // left side of episode text
  if (user_data == null) {
    episode_text = "0/";
  } else {
    episode_text = user_data.progress + "/";
  }
  // right side of episode text
  if (anime.episodes == null) {
    episode_text += "??";
  } else {
    episode_text += anime.episodes;
  }
  // progress bar length
  if (user_data != null) {
    if (anime.episodes != null) {
      watch_percent = (user_data.progress / anime.episodes);
    } else if (user_data.progress > 0) {
      watch_percent = 0.1;
    }
  }
  // protection for bad data
  if (watch_percent > 1.0) {
    watch_percent = 1.0;
  } else if (watch_percent < 0.0) {
    watch_percent = 0.0;
  }

  var start_date = 0;
  if (anime.start_date != null) {
    start_date = (anime.start_date.year * 10000 + anime.start_date.month * 100 + anime.start_date.day);
  }

  var cover_image = "./assets/no_cover_image.png";
  if (anime.cover_image != null) {
    cover_image = anime.cover_image.large;
  }

  var average_score = 0;
  if (anime.average_score != null) {
    average_score = anime.average_score;
  }

  add_cover_card(anime.id, cover_id, cover_image, start_date, anime.popularity, average_score, title, episode_text);

  draw_episode_canvas(anime.episodes, watch_percent, anime.id, cover_id);
}

// insert a card into the ui
window.add_cover_card = add_cover_card;
async function add_cover_card(anime_id, cover_id, cover_image, start_date, popularity, score, title, episode_text) {

  var html = "";
  html += "<div anime_id=" + anime_id + " class=\"cover_container\" date=" + start_date + " popularity=" + popularity + " score=" + score + " title=\"" + title + "\">";
  html += "<img alt=\"Cover Image\" class=\"image\" height=\"300\" id=\"" + cover_id + "\" src=" + cover_image + " width=\"200\">";
  if (current_tab == "BROWSE") {
    html += "<button class=\"add_planning_button\" onclick=\"add_to_list(" + anime_id + ", 'PLANNING')\" type=\"button\">Add to Planning</button>";
    html += "<button class=\"add_watching_button\" onclick=\"add_to_list(" + anime_id + ", 'CURRENT')\" type=\"button\">Add to Watching</button>";
  } else {
    html += "<button class=\"big_play_button\" onclick=\"play_next_episode(" + anime_id + ")\" type=\"button\"><img ,=\"\" height=\"80\" src=\"assets/play2.png\" width=\"80\"></button>";
  }
  html += "<div class=\"cover_nav\">";
  html +=   "<a href=\"#\" onclick=\"show_anime_info_window(" + anime_id + ")\" style=\"border-top-left-radius: 12px; border-bottom-left-radius:12px\">info</a>";
  html +=   "<a href=\"#\" onclick=\"decrease_episode(" + anime_id + ")\">-</a>";
  html +=   "<a href=\"#\" onclick=\"show_anime_info_window_edit(" + anime_id + ")\">" + episode_text + "</a>";
  html +=   "<a href=\"#\" onclick=\"increase_episode(" + anime_id + ")\" style=\"border-top-right-radius: 12px; border-bottom-right-radius:12px\">+</a>";
  html += "</div>";
  html += "<canvas class=\"episodes_exist\" height=\"5\" id=\"progress_episodes" + cover_id + "\" width=\"200\"></canvas>";
  html += "<div class=\"cover_title\">";
  html +=   "<p id=\"title" + anime_id + "\">" + title + "</p>";
  html += "</div>";
  html += "</div>";

  document.getElementById("cover_panel_grid").insertAdjacentHTML("beforeend", html);
}

// fills in the episode progress bar to show episodes available on disk and episodes watched
window.draw_episode_canvas = draw_episode_canvas;
async function draw_episode_canvas(episodes, watch_percent, media_id, cover_id) {
  
  var bar = document.getElementById("progress_episodes" + cover_id);
  var ctx = bar.getContext("2d"); 

  var width = bar.width / episodes;

  ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight-secondary');
  var episodes_exist = await invoke("episodes_exist_single", { id: media_id });

  // draw episodes on disk if there are any
  if(episodes_exist.length > 0){

    var start = 1;
    var length = 0;
    var last_episode = 0;
    episodes_exist.sort(function(a,b) {
      return a-b;
    });

    // cycle through episodes present on disk and draw rect to represent which episodes exist
    // consecutive episodes are drawn at the same time to eliminate ugly gaps
    for(var i = 0; i < episodes_exist.length; i++) {
  
      var episode = episodes_exist[i];
      if (episode == last_episode + 1) {
        // track consecutive episodes in order to draw one rect instead of multiple small ones that might have gaps
        last_episode = episode;
        length++;
      } else {
        // draw rect to represent episodes on disk
        ctx.fillRect((start - 1) * width, 0, width * length, 5);
  
        // reset consecutive tracking
        last_episode = episode;
        start = episode;
        length = 1;
      }
      // draw rect until end
      if (i == episodes_exist.length - 1) {
        
        ctx.fillRect((start - 1) * width, 0, width * length, 5);
      }
    }
  }

  ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight');
  ctx.fillRect(0, 0, watch_percent * 200, 5);
}

// fill in the ui with anime retrieved from anilist based on the categories selected
window.browse_update = browse_update;
async function browse_update() {

  var year = document.getElementById("year_select").value;
  var season = document.getElementById("season_select").value;
  var format = document.getElementById("format_select").value;
  var genre = document.getElementById("genre_select").value;

  var sort_value = "";
  switch(document.getElementById("sort_order_text").textContent) {
    case "Alphabetical":
      sort_value = "TITLE_ROMAJI";
      break;
    case "Score":
      sort_value = "SCORE";
      break;
    case "Date":
      sort_value = "START_DATE";
      break;
    case "Popularity":
      sort_value = "POPULARITY";
      break;
  }
  if (document.getElementById("sort_order_ascending").order == "DESC") {
    sort_value += "_DESC";
  }

  var list = await invoke("browse", {year: year, season: season, genre: genre, format: format, order: sort_value});

  var user_settings = await invoke("get_user_settings");
  removeChildren(document.getElementById("cover_panel_grid"));
  for(var i = 0; i < list.length; i++) {
    if(user_settings.show_adult == false && list[i].is_adult == true) {
      continue;
    }
    add_anime(list[i], null, i);
  }
  sort_anime();
}

// opens the file for the next episode in the default program
window.play_next_episode = play_next_episode;
async function play_next_episode(id) {
  await invoke("play_next_episode", { id: id });
}

// add a new anime to the users list
window.add_to_list = add_to_list;
async function add_to_list(id, list) {
  await invoke("add_to_list", { id: id, list: list});
}

// hide information window and return to cover grid
window.hide_anime_info_window = hide_anime_info_window;
async function hide_anime_info_window(anime_id) {
  document.getElementById("youtube_embed").src = "";
  document.getElementById("info_panel").style.display = "none";
  document.getElementById("cover_panel_grid").style.opacity = 1;
  if (anime_id != null) {
    var refresh = await update_user_entry(anime_id);
    if (refresh == true) {
      show_anime_list(current_tab);
    }
  }
}

// show information window populated with the shows info
window.show_anime_info_window = show_anime_info_window;
async function show_anime_info_window(anime_id) {
  
  var info = await invoke("get_anime_info", {id: anime_id});
  var title = "";
  if(info.title.english != null) {
    title = info.title.english;
  } else if(info.title.romaji != null) {
    title = info.title.romaji;
  } else {
    title = info.title.native;
  }

  document.getElementById("info_cover").src = info.cover_image.large;
  document.getElementById("info_description").innerHTML = info.description;
  if(title.length > 55) {
    document.getElementById("info_title").textContent = title.substring(0, 55) + "...";
  } else {
    document.getElementById("info_title").textContent = title;
  }
  if (info.format != "TV") {
    document.getElementById("info_format").textContent = info.format.charAt(0) + info.format.toLowerCase().slice(1);
  } else {
    document.getElementById("info_format").textContent = info.format;
  }
  document.getElementById("info_rating").textContent = info.average_score + "%";
  if (info.episodes == 1) {
    document.getElementById("info_duration").textContent = info.duration + " Minutes";
  } else if (info.episodes == null) {
    document.getElementById("info_duration").textContent = "?? x " + info.duration + " Minutes";
  } else {
    document.getElementById("info_duration").textContent = info.episodes + " x " + info.duration + " Minutes";
  }
  document.getElementById("info_season_year").textContent = info.season.charAt(0) + info.season.toLowerCase().slice(1) + " " + info.season_year;

  var genres_text = "";
  for (var i = 0; i < info.genres.length; i++) {
    genres_text += info.genres[i];
    if (i != info.genres.length - 1) {
      genres_text += ", ";
    }
  }
  document.getElementById("info_genres").textContent = genres_text;
  
  var user_settings = await invoke("get_user_settings");
  var tags = "";
  for (var i = 0; i < info.tags.length; i++) {
    if (user_settings.show_spoilers == false && (info.tags[i].is_general_spoiler || info.tags[i].is_media_spoiler)) {
      continue;
    }
    tags += info.tags[i].name + ", ";
  }
  tags = tags.substring(0, tags.length - 2);
  document.getElementById("info_tags").textContent = tags;

  if(info.trailer != null && info.trailer.site == "youtube") {
    document.getElementById("trailer_button").style.display = "block";
    document.getElementById("youtube_embed").src = "https://www.youtube.com/embed/" + info.trailer.id;
  } else {
    document.getElementById("trailer_button").style.display = "none";
  }

  var user_data = await invoke("get_user_info", {id: anime_id});

  document.getElementById("delete_anime").onclick = function() { confirm_delete_entry(user_data.id, user_data.media_id); }
  document.getElementById("status_select").value = user_data.status;
  document.getElementById("episode_number").value = user_data.progress;
  document.getElementById("score_0to5").value = user_data.score;
  if (user_data.started_at != null) {
    document.getElementById("started_date").value = user_data.started_at.year + "-" + String(user_data.started_at.month).padStart(2,'0') + "-" + String(user_data.started_at.day).padStart(2,'0');
  }
  if (user_data.completed_at != null) {
    document.getElementById("finished_date").value = user_data.completed_at.year + "-" + String(user_data.completed_at.month).padStart(2,'0') + "-" + String(user_data.completed_at.day).padStart(2,'0');
  }
  document.getElementById("info_close_button").onclick = function() { hide_anime_info_window(user_data.media_id)};

  openTab('information', 'underline_tab_0');
  document.getElementById("info_panel").style.display = "block";
  document.getElementById("cover_panel_grid").style.opacity = 0.3;
}

// decrease the users progress by 1
window.decrease_episode = decrease_episode;
async function decrease_episode(anime_id) {
  
  await invoke("increment_decrement_episode", {animeId: anime_id, change: -1});
  show_anime_list(current_tab);
}

// open the info window to the edit user info tab
window.show_anime_info_window_edit = show_anime_info_window_edit;
async function show_anime_info_window_edit(anime_id) {
  await show_anime_info_window(anime_id);
  openTab('user_entry', 'underline_tab_1');
}

// increases the users progress by 1
window.increase_episode = increase_episode;
async function increase_episode(anime_id) {

  await invoke("increment_decrement_episode", {animeId: anime_id, change: 1});
  show_anime_list(current_tab);
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
  document.getElementById(tab_name).style.display = "block";
  document.getElementById(underline_name).style.visibility = "visible";
}

// updates the entry for the current anime with new information from the info window
async function update_user_entry(anime_id) {

  var user_data = await invoke("get_user_info", {id: anime_id});

  var user_entry = {
    'id': user_data.id,
    'media_id': anime_id,
    'status': document.getElementById("status_select").value,
    'score': parseInt(document.getElementById("score_0to5").value),
    'progress': parseInt(document.getElementById("episode_number").value)
  };


  var started = document.getElementById("started_date").value.split("-");
  if (started.length == 3) {
    user_entry.started_at = {year: parseInt(started[0]), month: parseInt(started[1]), day: parseInt(started[2])};
  } else {
    user_entry.started_at = {year: null, month: null, day: null};
  }

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

  // return true if the status has changed and the list needs to be refreshed
  return user_entry.status != user_data.status;
}

// clears the date next the the button that has been pushed
window.clearDate = clearDate;
async function clearDate(date_id) {
  document.getElementById(date_id).value = "";
}

// hide the settings window and set the settings in rust
window.hide_setting_window = hide_setting_window;
async function hide_setting_window() {
  document.getElementById("login_panel").style.visibility = "hidden";
  document.getElementById("cover_panel_grid").style.opacity = 1;

  var username = document.getElementById("user_name").value;
  var language = document.getElementById("title_language").value;
  var show_spoiler = document.getElementById("show_spoiler_tags").checked;
  var show_adult = document.getElementById("show_adult").checked;
  var folders = document.getElementById("folders").value.split('\n');

  invoke("set_user_settings", { username: username, titleLanguage: language, showSpoilers: show_spoiler, showAdult: show_adult, folders: folders});
}

// close the window
window.exitWindow = exitWindow;
async function exitWindow() {
  window.close();
}

// minimize the window
window.minimizeWindow = minimizeWindow;
async function minimizeWindow() {
  window.minimize();
}

// maximize the window
window.toggleMaximizeWindow = toggleMaximizeWindow;
async function toggleMaximizeWindow() {
  window.toggleMaximizeWindow();
}
