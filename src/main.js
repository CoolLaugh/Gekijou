const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", async () => {

  await invoke("load_user_settings");

  var user_settings = await invoke("get_user_settings");
  document.styleSheets[0].cssRules[0].style.setProperty("--highlight", user_settings.highlight_color);

  await invoke("on_startup");

  populate_year_dropdown();

  document.getElementById("information").style.display = "block";
  document.getElementById("underline_tab_0").style.visibility = "visible";

  invoke("anime_update_delay_loop");

  check_for_refresh_ui();
});

window.set_color = set_color;
async function set_color(element) {

  var parent = document.getElementById("color_boxes");
  var elements = parent.childNodes;

  for (var i=0; i<elements.length; i++) {

    if(elements[i].nodeType == 1) {
      elements[i].style.setProperty("border-style", "hidden");
      elements[i].style.setProperty("margin", "2.5px");
    }
  }
  element.style.setProperty("border-style", "solid");
  element.style.setProperty("margin", "0px");

  document.styleSheets[0].cssRules[0].style.setProperty("--highlight", element.style.getPropertyValue("background"));
}

window.get_user_settings = get_user_settings;
async function get_user_settings() {
  
  var user_settings = await invoke("get_user_settings");
  
  document.getElementById("user_name").value = user_settings.username;
  document.getElementById("title_language").value = user_settings.title_language;
  document.getElementById("show_spoiler_tags").checked = user_settings.show_spoilers;
  document.getElementById("show_adult").checked = user_settings.show_adult;
  document.getElementById("update_delay").value = user_settings.update_delay;
  var folder_textarea = document.getElementById("folders");
  folder_textarea.value = "";
  for(var i = 0; i < user_settings.folders.length; i++){
    folder_textarea.value += user_settings.folders[i];
    if(i + 1 != user_settings.folders.length) {
      folder_textarea.value += "\n";
    }
  }
  
  document.styleSheets[0].cssRules[0].style.setProperty("--highlight", user_settings.highlight_color);
  var elements = document.getElementById("color_boxes").childNodes;
  for (var i=0; i<elements.length; i++) {

    if(elements[i].nodeType != 1) { 
      continue;
    }

    if (elements[i].style.getPropertyValue("background") == user_settings.highlight_color) {
      elements[i].style.setProperty("border-style", "solid");
      elements[i].style.setProperty("margin", "0px");
    } else {
      elements[i].style.setProperty("border-style", "hidden");
      elements[i].style.setProperty("margin", "2.5px");
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
    if (refresh.anime_list == true) {
      show_anime_list(current_tab);
    }
    
    draw_delay_progress();
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
      document.getElementById("score_dropdown").value = "0";
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
  expected_page = 0;
  current_page = 0;
  has_next_page = true;
  show_anime_list_paged(current_page);
  exclusive_underline(0);
  populate_sort_dropdown(false);
  var grid = document.getElementById("cover_panel_id");
  grid.onscroll = function(ev) {
    if (ev.target.offsetHeight + ev.target.scrollTop >= (ev.target.scrollHeight - 500)) {
      show_anime_list_paged(current_page);
    }
  };
}

// show the users watching list
window.show_completed_anime = show_completed_anime;
async function show_completed_anime() {
  current_tab = "COMPLETED";
  expected_page = 0;
  current_page = 0;
  has_next_page = true;
  show_anime_list_paged(current_page);
  exclusive_underline(1);
  populate_sort_dropdown(false);
  var grid = document.getElementById("cover_panel_id");
  grid.onscroll = function(ev) {
    if (ev.target.offsetHeight + ev.target.scrollTop >= (ev.target.scrollHeight - 500)) {
      show_anime_list_paged(current_page);
    }
  };
}

// show the users paused list
window.show_paused_anime = show_paused_anime;
async function show_paused_anime() {
  current_tab = "PAUSED";
  expected_page = 0;
  current_page = 0;
  has_next_page = true;
  show_anime_list_paged(current_page);
  exclusive_underline(2);
  populate_sort_dropdown(false);
  var grid = document.getElementById("cover_panel_id");
  grid.onscroll = function(ev) {
    if (ev.target.offsetHeight + ev.target.scrollTop >= (ev.target.scrollHeight - 500)) {
      show_anime_list_paged(current_page);
    }
  };
}

// show the users dropped list
window.show_dropped_anime = show_dropped_anime;
async function show_dropped_anime() {
  current_tab = "DROPPED";
  expected_page = 0;
  current_page = 0;
  has_next_page = true;
  show_anime_list_paged(current_page);
  exclusive_underline(3);
  populate_sort_dropdown(false);
  var grid = document.getElementById("cover_panel_id");
  grid.onscroll = function(ev) {
    if (ev.target.offsetHeight + ev.target.scrollTop >= (ev.target.scrollHeight - 500)) {
      show_anime_list_paged(current_page);
    }
  };
}

// show the users plan to watch list
window.show_planning_anime = show_planning_anime;
async function show_planning_anime() {
  current_tab = "PLANNING";
  expected_page = 0;
  current_page = 0;
  has_next_page = true;
  show_anime_list_paged(current_page);
  exclusive_underline(4);
  populate_sort_dropdown(false);
  var grid = document.getElementById("cover_panel_id");
  grid.onscroll = function(ev) {
    if (ev.target.offsetHeight + ev.target.scrollTop >= (ev.target.scrollHeight - 500)) {
      show_anime_list_paged(current_page);
    }
  };
}

// show the controls to allow the user to look for anime based on year, season, genre, and format
window.show_browse_anime = show_browse_anime;
async function show_browse_anime() {
  current_tab = "BROWSE";
  exclusive_underline(5);
  populate_sort_dropdown(true);
  document.getElementById("browse_filters").style.display = "block";
  removeChildren(document.getElementById("cover_panel_grid"));
  document.getElementById("cover_panel_id").onscroll = null;
}

function populate_sort_dropdown(browse) {

  var index = document.getElementById("sort_order").selectedIndex;

  removeChildren(document.getElementById("sort_order"));
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Alphabetical\">Alphabetical</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Score\">Score</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Date\">Date</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Popularity\">Popularity</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Trending\">Trending</option>");
  if (browse == false) {
    document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Started\">Started</option>");
    document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Completed\">Completed</option>");
  } else if (index >= 5) {
      index = 0;
  }

  document.getElementById("sort_order").selectedIndex = index;
}

// draw progress bar for recognizing anime being played by media players
window.draw_delay_progress = draw_delay_progress;
async function draw_delay_progress() {

  var percent = await invoke("get_delay_info");
  var ctx = document.getElementById("recognition_delay").getContext("2d");

  if (percent[0] == 0.0 || percent[0] >= 0.995) {
    // no anime being tracked or anime is about to update anyway so don't track it
    ctx.clearRect(0,0,52,52);
    document.getElementById("recognition_delay").title = "";
  } else {
    // format seconds remaining as minutes and seconds
    var time_remaining = "";
    if (percent[3] >= 60) {
      time_remaining = Math.floor(percent[3] / 60) + "m " + (percent[3] % 60) + "s";
    } else {
      time_remaining = percent[3] + "s";
    }
    // full description tooltip text
    document.getElementById("recognition_delay").title = "Updating " + percent[2] + " to episode " + percent[1] + " in " + time_remaining;

    ctx.clearRect(0,0,52,52);
    
    // progress bar background
    ctx.beginPath();
    ctx.arc(26,26,25,0, 2 * Math.PI, false);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight-secondary');
    ctx.fill();

    // progress bar
    ctx.beginPath();
    ctx.arc(26,26,25, 1.5 * Math.PI, (1.5 + (2 * percent[0])) * Math.PI, false);
    ctx.lineTo(26, 26);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight');
    ctx.fill();
    
    // hollow center
    ctx.beginPath();
    ctx.arc(26,26,21,0, 2 * Math.PI, false);
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--background-color2');
    ctx.fill();
  
    // center text
    var left = 14;
    if (percent[1] >= 10) {
      left -= 3;
    }
    var left2 = 19;
    if (percent[0] > 0.095) {
      left2 -= 4;
    }
    // timer text
    ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight');
    ctx.font = "12px Arial";
    ctx.fillText("EP " + percent[1], left, 25);
    ctx.fillText(Math.round(percent[0] * 100) + "%", left2, 37);
  }
}

// shows the settings window
window.show_setting_window = show_setting_window;
async function show_setting_window() {
  get_user_settings();
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

  if (watching[1] != null) {
    alert(watching[1]);
  } else {

    var user_data = await invoke("get_list_user_info", { listName: name });

    // get user data on anime
    var user_settings = await invoke("get_user_settings");

    // user didn't change the tab while getting the list from anilist
    if (name == current_tab) {

      // add anime to UI
      removeChildren(document.getElementById("cover_panel_grid"));

      for(var i = 0; i < watching[0].length; i++) {
        if(user_settings.show_adult == false && watching[0][i].is_adult == true) {
          continue;
        }
        add_anime(watching[0][i], user_data[i], i);
      }

      sort_anime();
    }
  }
}


var current_page = 0;
var expected_page = 0;
var has_next_page = true;
window.show_anime_list_paged = show_anime_list_paged;
async function show_anime_list_paged(page) {

  if (has_next_page == false ||
    page != expected_page) {
    return;
  }
  expected_page++;

  var name = current_tab;
  
  document.getElementById("browse_filters").style.display = "none";

  var watching = await invoke("get_list_paged", { listName: current_tab, sort: document.getElementById("sort_order").value, ascending: sort_ascending, page: page});
  var user_settings = await invoke("get_user_settings");

  // user didn't change the tab while getting the list from anilist
  if (name == current_tab) {

      // add anime to UI
      if (page == 0) {
        removeChildren(document.getElementById("cover_panel_grid"));
      }
      if (watching.length < 50) {
        has_next_page = false;
      }

      for(var i = 0; i < watching.length; i++) {
        if(user_settings.show_adult == false && watching[i][0].is_adult == true) {
          continue;
        }
        await add_anime(watching[i][0], watching[i][1], i);
      }
      current_page++;
  }
}

// remove all html children of the current element.  used to clear the anime list on screen
const removeChildren = (parent) => {
  while (parent.lastChild) {
      parent.removeChild(parent.lastChild);
  }
};

// list of categories that can be searched by
// variables are field name, display name, and default sorting order
var sort_ascending = true;
const default_order = {"Alphabetical": true, "Score": false, "Date": true, "Popularity": false, "Trending": false, "Started": false, "Completed": false}
// cycle through different ways of sorting shows
window.change_sort_type = change_sort_type;
async function change_sort_type() {

  sort_ascending = default_order[document.getElementById("sort_order").value];
  change_ascending_indicator(sort_ascending);

  if (current_tab == "BROWSE") {
    browse_update();
  } else {
    expected_page = 0;
    current_page = 0;
    has_next_page = true;
    show_anime_list_paged(current_page);
  }
}

// change between sorting ascending and descending
window.change_sort_ascending = change_sort_ascending;
async function change_sort_ascending() {
  sort_ascending = !sort_ascending;
  change_ascending_indicator(sort_ascending)
  if (current_tab == "BROWSE") {
    browse_update();
  } else {
    expected_page = 0;
    current_page = 0;
    has_next_page = true;
    show_anime_list_paged(current_page);
  }
}

// change the image to show if the list is being sorted ascending or descending
function change_ascending_indicator(ascending) {
  if(ascending == true) {
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
  var sort_me = [];
  
  for (var i=0; i<elements.length; i++) {
      
    if (elements[i].nodeType == 1) {

      switch(document.getElementById("sort_order").value) {
        case "Alphabetical":
          sort_me.push([ elements[i].getAttribute("title").toLowerCase() , elements[i] ]);
          break;
        case "Score":
          sort_me.push([ parseInt(elements[i].getAttribute("score"), 10) , elements[i] ]);
          break;
        case "Date":
          sort_me.push([ parseInt(elements[i].getAttribute("date"), 10) , elements[i] ]);
          break;
        case "Popularity":
          sort_me.push([ parseInt(elements[i].getAttribute("popularity"), 10) , elements[i] ]);
          break;
        case "Trending":
          sort_me.push([ parseInt(elements[i].getAttribute("trending"), 10) , elements[i] ]);
          break;
        case "Started":
          sort_me.push([ parseInt(elements[i].getAttribute("started"), 10) , elements[i] ]);
          break;
        case "Completed":
          sort_me.push([ parseInt(elements[i].getAttribute("completed"), 10) , elements[i] ]);
          break;
      }
    }
  }

  switch(document.getElementById("sort_order").value) {
    case "Alphabetical":
      sort_me.sort();
      break;
    case "Score": // intentional fall through
    case "Date": // intentional fall through
    case "Popularity":
    case "Trending":
    case "Started":
    case "Completed":
      sort_me.sort(function(a,b) {
        return a[0]-b[0];
      });
      break;
  }

  if (sort_ascending == false) {
    sort_me.reverse();
  }

  for (var i=0; i<sort_me.length; i++) {
      container.appendChild(sort_me[i][1]);
  }
}

// add an anime to the ui
window.add_anime = add_anime;
async function add_anime(anime, user_data, cover_id) {

  var title = await determine_title(anime.title, null);

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

  var started_date = 0;
  if (anime.started_at != null) {
    started_date = (user_data.started_at.year * 10000 + user_data.started_at.month * 100 + user_data.started_at.day);
  }

  var completed_date = 0;
  if (anime.completed_at != null) {
    completed_date = (user_data.completed_at.year * 10000 + user_data.completed_at.month * 100 + user_data.completed_at.day);
  }

  var cover_image = "./assets/no_cover_image.png";
  if (anime.cover_image != null) {
    cover_image = anime.cover_image.large;
  }

  var average_score = 0;
  if (anime.average_score != null) {
    average_score = anime.average_score;
  }

  var display_browse = "none";
  var display_not_browse = "none";
  if (current_tab == "BROWSE") {
    display_browse = "block";
  } else {
    display_not_browse = "block";
  }

  var display_trailer = "none";
  if (current_tab == "BROWSE" && anime.trailer != null) {
    display_trailer = "block";
  }

  var sort_value = determine_sort_value(anime, user_data);
  var display_sort_value = "none";
  if (sort_value.length > 0) {
    display_sort_value = "block";
  }

  var html = "";

  html += "<div id=\"" + anime.id + "\" class=\"cover_container\" date=\"" + start_date + "\" popularity=\"" + anime.popularity + "\" score=\"" + average_score + "\" title=\"" + title + "\" trending=\"" + anime.trending + "\" started=\"" + started_date + "\" completed=\"" + completed_date + "\">"
  html +=   "<img alt=\"Cover Image\" class=\"image\" height=\"300\" id=\"" + cover_id + "\" src=\"" + cover_image + "\" width=\"200\">"
  html +=   "<div class=\"sort_value_display\" style=\"display: " + display_sort_value + ";\"><p id=\"sort_value\">" + sort_value + "</p></div>"
  html +=   "<canvas class=\"episodes_exist\" height=\"5\" id=\"progress_episodes_" + anime.id + "\" width=\"200\"></canvas>"
  html +=   "<div class=\"cover_title\"><p id=\"title" + anime.id + "\">" + title + "</p></div>"
  html +=   "<div class=\"overlay\">"
  html +=     "<div class=\"add_buttons\"><a href=\"#\" onclick=\"show_anime_info_window(" + anime.id + ")\" title=\"See the description, score, episodes, etc\">Information</a></div>"
  html +=     "<div class=\"add_buttons\" style=\"top: 93px; display: " + display_browse + ";\"><a href=\"#\" onclick=\"add_to_list(" + anime.id + ", 'PLANNING')\" title=\"Add this anime to your plan to watch list\">Add to Planning</a></div>"
  html +=     "<div class=\"add_buttons\" style=\"top: 163px; display: " + display_browse + ";\"><a href=\"#\" onclick=\"add_to_list(" + anime.id + ", 'CURRENT')\" title=\"Add this anime to your watching list\">Add to Watching</a></div>"
  html +=     "<div class=\"add_buttons\" style=\"top: 232px; display: " + display_trailer + ";\"><a href=\"#\" onclick=\"show_anime_info_window_trailer(" + anime.id + ")\" title=\"Watch the trailer\">Watch Trailer</a></div>"
  html +=     "<button class=\"big_play_button\" onclick=\"play_next_episode(" + anime.id + ")\" type=\"button\" style=\"display: " + display_not_browse + ";\" title=\"Play Next Episode\"><img ,=\"\" height=\"80\" src=\"assets/play2.png\" width=\"80\"></button>"
  html +=     "<div class=\"cover_nav\" style=\"display: " + display_not_browse + ";\">"
  html +=       "<a href=\"#\" onclick=\"decrease_episode(" + anime.id + ")\" style=\"border-top-left-radius: 12px; border-bottom-left-radius:12px; font-size: 24px;\" title=\"Decrease episode progress\">-</a>"
  html +=       "<a href=\"#\" onclick=\"show_anime_info_window_edit(" + anime.id + ")\" id=\"episode_text_" + anime.id + "\" title=\"Edit episode and other data\">" + episode_text + "</a>"
  html +=       "<a href=\"#\" onclick=\"increase_episode(" + anime.id + ")\" style=\"border-top-right-radius: 12px; border-bottom-right-radius:12px; font-size: 24px;\" title=\"Increase episode progress\">+</a>"
  html +=     "</div>"
  html +=   "</div>"
  html += "</div>"

  document.getElementById("cover_panel_grid").insertAdjacentHTML("beforeend", html);

  if (user_data != null) {
    draw_episode_canvas(user_data.progress, anime.episodes, anime.id);
  }
}

function determine_sort_value(anime, user_data) {

  switch(document.getElementById("sort_order").value) {
    case "Alphabetical":
      return "";
      break;
    case "Score":
      return anime.average_score + "%";
      break;
    case "Date":
      if (anime.start_date != null) {
        return anime.start_date.year + "-" + anime.start_date.month + "-" + anime.start_date.day;
      } else {
        return "????-??-??";
      }
      break;
    case "Popularity":
      return anime.popularity + "";
      break;
    case "Trending":
      return anime.trending + "";
      break;
    case "Started":
      if (user_data.started_at != null) {
        if (user_data.started_at.year == null && user_data.started_at.month == null && user_data.started_at.day == null){
          return "No Date";
        }
        return user_data.started_at.year + "-" + user_data.started_at.month + "-" + user_data.started_at.day;
      } else {
        return "????-??-??";
      }
      break;
    case "Completed":
      if (user_data.completed_at != null) {
        if (user_data.started_at.year == null && user_data.started_at.month == null && user_data.started_at.day == null){
          return "No Date";
        }
        return user_data.completed_at.year + "-" + user_data.completed_at.month + "-" + user_data.completed_at.day;
      } else {
        return "????-??-??";
      }
      break;
  }
}

// fills in the episode progress bar to show episodes available on disk and episodes watched
window.draw_episode_canvas = draw_episode_canvas;
async function draw_episode_canvas(episode, total_episodes, media_id) {
  
  var watch_percent = 0.0;
  if (episode != null) {
    watch_percent = (episode / total_episodes);
  } else if (episode > 0) {
    watch_percent = 0.1;
  }
  
  // protection for bad data
  if (watch_percent > 1.0) {
    watch_percent = 1.0;
  } else if (watch_percent < 0.0) {
    watch_percent = 0.0;
  }

  var bar = document.getElementById("progress_episodes_" + media_id);
  bar.title = "Watched: " + episode + " / " + total_episodes;

  var ctx = bar.getContext("2d");
  ctx.clearRect(0,0,200,5);

  var width = bar.width / total_episodes;

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

    bar.title += "\nEpisodes on disk: ";

    var comma = false;
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
        if (comma == true) { bar.title += ", "}
        bar.title += start + "-" + (start + length - 1);
        comma = true;
  
        // reset consecutive tracking
        last_episode = episode;
        start = episode;
        length = 1;
      }
      if (i == episodes_exist.length - 1) {
        // draw rect until end
        ctx.fillRect((start - 1) * width, 0, width * length, 5);
        if (comma == true) { bar.title += ", "}
        bar.title += start + "-" + (start + length - 1);
        comma = true;
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
  var user_settings = await invoke("get_user_settings");

  var sort_value = "";
  switch(document.getElementById("sort_order").value) {
    case "Alphabetical":
      switch(user_settings.title_language) {
        case "romaji":
          sort_value = "TITLE_ROMAJI";
          break;
        case "english":
          sort_value = "TITLE_ENGLISH";
          break;
        case "native":
          sort_value = "TITLE_NATIVE";
          break;
      }
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
    case "Trending":
      sort_value = "TRENDING";
      break;
  }
  if (document.getElementById("sort_order_ascending").order == "DESC") {
    sort_value += "_DESC";
  }

  var list = await invoke("browse", {year: year, season: season, genre: genre, format: format, order: sort_value});

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

async function determine_title(title_struct, user_settings) {

  if (user_settings == null) {
    user_settings = await invoke("get_user_settings");
  }

  var title = null;
  // try to use the language the user chose
  if (user_settings.title_language == "romaji" && title_struct.romaji != null) {
    title = title_struct.romaji;
  } else if (user_settings.title_language == "english" && title_struct.english != null) {
    title = title_struct.english;
  } else if (user_settings.title_language == "native" && title_struct.native != null) {
    title = title_struct.native;
  }
  // if the preferred language does not exist use another language
  if (title == null) {
    if(title_struct.romaji != null) {
      title = title_struct.romaji;
    } else if(title_struct.english != null) {
      title = title_struct.english;
    } else {
      title = title_struct.native;
    }
  }

  return title;
}

// show information window populated with the shows info
window.show_anime_info_window = show_anime_info_window;
async function show_anime_info_window(anime_id) {
  
  var user_settings = await invoke("get_user_settings");
  var info = await invoke("get_anime_info", {id: anime_id});
  var title = await determine_title(info.title, user_settings);

  var episode_text = "";
  if (info.episodes == null) {
    episode_text = "?? x "
  } else if (info.episodes > 1) {
    episode_text = info.episodes + " x "
  }
  if (info.duration == null) {
    episode_text += "?? Minutes"
  } else {
    episode_text += info.duration + " Minutes"
  }

  document.getElementById("info_cover").src = info.cover_image.large;
  document.getElementById("studio").innerHTML = info.studios.nodes[0].name;
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
  if (info.average_score == null) {
    document.getElementById("info_rating").textContent = "No Score";
  } else {
    document.getElementById("info_rating").textContent = info.average_score + "%";
  }
  document.getElementById("info_duration").textContent = episode_text;
  document.getElementById("info_season_year").textContent = info.season.charAt(0) + info.season.toLowerCase().slice(1) + " " + info.season_year;

  var genres_text = "";
  for (var i = 0; i < info.genres.length; i++) {
    genres_text += info.genres[i];
    if (i != info.genres.length - 1) {
      genres_text += ", ";
    }
  }
  document.getElementById("info_genres").textContent = genres_text;
  
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
  setup_score_dropdown(user_settings.score_format);
  document.getElementById("score_dropdown").value = user_data.score;
  if (user_data.started_at != null) {
    document.getElementById("started_date").value = user_data.started_at.year + "-" + String(user_data.started_at.month).padStart(2,'0') + "-" + String(user_data.started_at.day).padStart(2,'0');
  } else {
    document.getElementById("started_date").value = "";
  }
  if (user_data.completed_at != null) {
    document.getElementById("finished_date").value = user_data.completed_at.year + "-" + String(user_data.completed_at.month).padStart(2,'0') + "-" + String(user_data.completed_at.day).padStart(2,'0');
  } else {
    document.getElementById("finished_date").value = "";
  }
  document.getElementById("info_close_button").onclick = function() { hide_anime_info_window(user_data.media_id)};

  add_related_anime(info.relations.edges, info.recommendations.nodes, user_settings.title_language);

  openTab('information', 'underline_tab_0');
  document.getElementById("info_panel").style.display = "block";
  document.getElementById("cover_panel_grid").style.opacity = 0.3;
}

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

function add_related_anime(related, recommendations, title_language) {

  var related_grid = document.getElementById("related_grid");
  removeChildren(related_grid);
  for(var i = 0; i < related.length; i++) {

    var title = related[i].node.title.romaji;
    if (title_language == "english" && related[i].node.title != null) {
      title = related[i].node.title.english;
    } else if (title_language == "native") {
      title = related[i].node.title.native;
    }
    var relation_type = related[i].relation_type.charAt(0) + related[i].relation_type.toLowerCase().slice(1);
    relation_type.replace("_", " ");

    var html = "";
    html +=  "<div style=\"width: 116px; text-align: center; background: var(--background-color1);\">"
    //html +=    "<div><p>" + relation_type + "</p></div>"
    html +=    "<a href=\"#\"><img class=image href=\"#\" height=\"174px\" src=\"" + related[i].node.cover_image.large + "\" width=\"116px\" onclick=\"show_anime_info_window(" + related[i].node.id + ")\"></a>"
    html +=    "<div style=\"height: 44px; overflow: hidden;\"><a href=\"#\"><p onclick=\"show_anime_info_window(" + recommendations[i].media_recommendation.id + ")\">" + title + "</p></a></div>"
    html +=  "</div>"

    related_grid.innerHTML += html;
  }

  var recommended_grid = document.getElementById("recommended_grid");
  removeChildren(recommended_grid);

  recommendations.sort(function(a,b) {
    return b.rating-a.rating;
  });

  for(var i = 0; i < recommendations.length; i++) {

    var title = recommendations[i].media_recommendation.title.romaji;
    if (title_language == "english" && recommendations[i].media_recommendation.title != null) {
      title = recommendations[i].media_recommendation.title.english;
    } else if (title_language == "native") {
      title = recommendations[i].media_recommendation.title.native;
    }
    var rating = recommendations[i].rating;

    var html = "";
    html +=  "<div style=\"width: 116px; text-align: center; background: var(--background-color1);\">"
    //html +=    "<div><p>" + rating + "</p></div>"
    html +=    "<a href=\"#\"><img class=image height=\"174px\" src=\"" + recommendations[i].media_recommendation.cover_image.large + "\" width=\"116px\" onclick=\"show_anime_info_window(" + recommendations[i].media_recommendation.id + ")\"></a>"
    html +=    "<div style=\"height: 44px; overflow: hidden;\"><a href=\"#\"><p onclick=\"show_anime_info_window(" + recommendations[i].media_recommendation.id + ")\">" + title + "</p></a></div>"
    html +=  "</div>"

    recommended_grid.innerHTML += html;
  }

}

// decrease the users progress by 1
window.decrease_episode = decrease_episode;
async function decrease_episode(anime_id) {
  
  await invoke("increment_decrement_episode", {animeId: anime_id, change: -1});

  var text = document.getElementById("episode_text_"+ anime_id);
  var episodes = text.textContent.split('/');
  var progress = parseInt(episodes[0]) - 1;
  var total = parseInt(episodes[1]);
  if (progress > -1) {

    text.textContent = progress + "/" + total;

    draw_episode_canvas(progress, total, anime_id);
  }
}

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

// increases the users progress by 1
window.increase_episode = increase_episode;
async function increase_episode(anime_id) {

  await invoke("increment_decrement_episode", {animeId: anime_id, change: 1});

  var text = document.getElementById("episode_text_"+ anime_id);
  var episodes = text.textContent.split('/');
  var progress = parseInt(episodes[0]) + 1;
  var total = parseInt(episodes[1]);
  if (progress <= total) {

    text.textContent = progress + "/" + total;

    draw_episode_canvas(progress, total, anime_id);
  
    if (progress == total) {
      show_anime_list(current_tab);
    }
  }
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

  var elements = document.getElementById("color_boxes").childNodes;
  var highlight_color = "";
  for (var i=0; i<elements.length; i++) {
    
    if(elements[i].nodeType != 1) { 
      continue;
    }

    if (elements[i].style.getPropertyValue("border-style") == "solid") {
      highlight_color = elements[i].style.getPropertyValue("background");
      break;
    } 
  }

  var settings = {
    username: document.getElementById("user_name").value,
    title_language: document.getElementById("title_language").value,
    show_spoilers: document.getElementById("show_spoiler_tags").checked,
    show_adult: document.getElementById("show_adult").checked,
    folders: document.getElementById("folders").value.split('\n'),
    update_delay: parseInt(document.getElementById("update_delay").value),
    score_format: "",
    highlight_color: highlight_color,
  }

  invoke("set_user_settings", { settings: settings});
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