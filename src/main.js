const { invoke } = window.__TAURI__.tauri;



window.addEventListener("DOMContentLoaded", async () => {

  populate_year_dropdown();

  document.getElementById("information").style.display = "block";
  document.getElementById("underline_tab_0").style.visibility = "visible";

  var user_settings = await invoke("get_user_settings");
  document.styleSheets[0].cssRules[0].style.setProperty("--highlight", user_settings.highlight_color);

  add_adult_genres(user_settings.show_adult);

  if (user_settings.first_time_setup == true) {

    show_setting_window();
    document.getElementById("login_panel").style.setProperty("left", "100%");
    document.getElementById("login_panel").style.setProperty("transform", "translate(-102%,-50%)");
    document.getElementById("first_time_setup").style.visibility = "visible";

  } else {
    
    if (user_settings.current_tab != "") {
      switch(user_settings.current_tab) {
        case "CURRENT":
          show_watching_anime();
          break;
        case "COMPLETED":
          show_completed_anime();
          break;
        case "PAUSED":
          show_paused_anime();
          break;
        case "DROPPED":
          show_dropped_anime();
          break;
        case "PLANNING":
          show_planning_anime();
          break;
        case "BROWSE":
          show_browse_anime();
          break;
      }
    }
  }

  invoke("anime_update_delay_loop");
});


async function add_adult_genres(show_adult) {

  if (show_adult == true) {

    var hentai_option = document.getElementById("hentai_option");

    if (hentai_option == null) {

      hentai_option = document.createElement("option");
      hentai_option.value = "Hentai";
      hentai_option.innerHTML = "Hentai";
      hentai_option.id = "hentai_option"
      
      document.getElementById("horror_option").insertAdjacentElement("beforebegin", hentai_option);
    }
  } else {
    
    var hentai_option = document.getElementById("hentai_option");

    if (hentai_option != null) {

      var drop_down = document.getElementById("genre_select");
      drop_down.removeChild(hentai_option);
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
var refresh_ui_interval = setInterval(refresh_ui, 1000);

async function refresh_ui() {

  clearInterval(refresh_ui_interval);

  var refresh = await invoke("get_refresh_ui");
  if (refresh.anime_list == true) {
    show_anime_list(current_tab);
  }
  if (refresh.canvas == true && current_tab != "BROWSE") {
    redraw_episode_canvas();
  }
  
  draw_delay_progress();

  refresh_ui_interval = setInterval(refresh_ui, 1000);
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


// show the users watching list
var current_tab = "";
window.show_watching_anime = show_watching_anime;
async function show_watching_anime() {
  if (current_tab == "CURRENT") {
    return;
  }
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
  invoke("set_current_tab", {currentTab: current_tab});
}

// show the users watching list
window.show_completed_anime = show_completed_anime;
async function show_completed_anime() {
  if (current_tab == "COMPLETED") {
    return;
  }
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
  invoke("set_current_tab", {currentTab: current_tab});
}

// show the users paused list
window.show_paused_anime = show_paused_anime;
async function show_paused_anime() {
  if (current_tab == "PAUSED") {
    return;
  }
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
  invoke("set_current_tab", {currentTab: current_tab});
}

// show the users dropped list
window.show_dropped_anime = show_dropped_anime;
async function show_dropped_anime() {
  if (current_tab == "DROPPED") {
    return;
  }
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
  invoke("set_current_tab", {currentTab: current_tab});
}

// show the users plan to watch list
window.show_planning_anime = show_planning_anime;
async function show_planning_anime() {
  if (current_tab == "PLANNING") {
    return;
  }
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
  invoke("set_current_tab", {currentTab: current_tab});
}

// show the controls to allow the user to look for anime based on year, season, genre, and format
window.show_browse_anime = show_browse_anime;
async function show_browse_anime() {
  if (current_tab == "BROWSE") {
    return;
  }
  current_tab = "BROWSE";
  exclusive_underline(5);
  populate_sort_dropdown(true);
  var user_settings = await invoke("get_user_settings");
  add_adult_genres(user_settings.show_adult);
  document.getElementById("browse_filters").style.display = "block";
  document.getElementById("recommended_filters").style.display = "none";
  document.getElementById("sort_area").style.display = "block";
  removeChildren(document.getElementById("cover_panel_grid"));
  document.getElementById("cover_panel_id").onscroll = null;
  invoke("set_current_tab", {currentTab: current_tab});
}

function populate_sort_dropdown(browse) {

  var index = document.getElementById("sort_order").selectedIndex;

  removeChildren(document.getElementById("sort_order"));
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Alphabetical\">Alphabetical</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"Score\">Score</option>");
  document.getElementById("sort_order").insertAdjacentHTML("beforeend", "<option value=\"MyScore\">My Score</option>");
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


// hide all underlines except one to show the current list being shown
function exclusive_underline(index) {

  for(var i = 0; i < 7; i++) {
    document.getElementById("underline" + i).style.visibility = "hidden";
  }
  document.getElementById("underline" + index).style.visibility = "visible";
}

// fill the UI with anime based on the list selected
window.show_anime_list = show_anime_list;
async function show_anime_list(name) {

  document.getElementById("browse_filters").style.display = "none";
  document.getElementById("recommended_filters").style.display = "none";
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
        add_anime(watching[0][i], user_data[i], i, user_settings.score_format, user_settings.show_airing_time);
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
  document.getElementById("recommended_filters").style.display = "none";
  document.getElementById("sort_area").style.display = "block";

  var get_list_response = await invoke("get_list_paged", { listName: current_tab, sort: document.getElementById("sort_order").value, ascending: sort_ascending, page: page});
  if (get_list_response[1] != null) {
    alert(get_list_response[1]);
  }
  var watching = get_list_response[0];

  var user_settings = await invoke("get_user_settings");

  // user didn't change the tab while getting the list from anilist
  if (name == current_tab) {

    // add anime to UI
    if (page == 0) {
      removeChildren(document.getElementById("cover_panel_grid"));
      list_ids = await invoke("get_list_ids", { list: current_tab });
    }
    if (watching.length < 50) {
      has_next_page = false;
    }

    for(var i = 0; i < watching.length; i++) {
      if(user_settings.show_adult == false && watching[i][0].is_adult == true) {
        continue;
      }
      await add_anime(watching[i][0], watching[i][1], i, user_settings.score_format, user_settings.show_airing_time);
    }
    current_page++;
  }
}

window.show_recommended_anime_list_tab = show_recommended_anime_list_tab;
async function show_recommended_anime_list_tab() {

  if (current_tab == "RECOMMENDED") {
    return;
  }
  current_tab = "RECOMMENDED";
  exclusive_underline(6);
  document.getElementById("cover_panel_id").onscroll = null;
  document.getElementById("sort_area").style.display = "none";
  document.getElementById("browse_filters").style.display = "none";
  document.getElementById("recommended_filters").style.display = "block";
  removeChildren(document.getElementById("cover_panel_grid"));

  show_recommended_anime_list();
}

window.show_recommended_anime_list = show_recommended_anime_list;
async function show_recommended_anime_list() {

  document.getElementById("loader_recommended").style.display = "inline-block";
  var genre = document.getElementById("genre_select_recommended").value;
  var format = document.getElementById("format_select_recommended").value;
  var year_split = document.getElementById("year_select_recommended").value.split("|");
  var year_start = 0;
  var year_end = 0;
  if (year_split != null && year_split.length == 2) {
    year_start = parseInt(year_split[0]);
    year_end = parseInt(year_split[1]);
  }

  var recommended_list = await invoke("recommend_anime", { genreFilter: genre, yearMinFilter: year_start, yearMaxFilter: year_end, formatFilter: format });
  var user_settings = await invoke("get_user_settings");

  if (current_tab == "RECOMMENDED") {

    document.getElementById("cover_panel_grid").innerHTML = "";
    removeChildren(document.getElementById("cover_panel_grid"));
    list_ids = [];
    
    for(var i = 0; i < recommended_list.length; i++) {
      if(user_settings.show_adult == false && recommended_list[i].is_adult == true) {
        continue;
      }
      await add_anime(recommended_list[i], null, i, user_settings.score_format, user_settings.show_airing_time);
      list_ids.push(recommended_list[i].id);
    }
  }
  document.getElementById("loader_recommended").style.display = "none";
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
const default_order = {"Alphabetical": true, "Score": false, "MyScore": false, "Date": false, "Popularity": false, "Trending": false, "Started": false, "Completed": false}
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
async function add_anime(anime, user_data, cover_id, score_format, show_airing) {

  var title = await determine_title(anime.title, null);

  // left side of episode text
  var episode_text = "";
  if (user_data != null) {
    episode_text = null_check(user_data.progress, user_data.progress + "/", "0/");
  } else {
    episode_text = "0/";
  }
  // right side of episode text
  episode_text += null_check(anime.episodes, anime.episodes, "??");
  // progress bar length
  var watch_percent = 0;
  if (user_data != null) {
    watch_percent = null_check(anime.episodes, user_data.progress / anime.episodes, 0.1);
  }
  // protection for bad data
  if (watch_percent > 1.0) {
    watch_percent = 1.0;
  } else if (watch_percent < 0.0) {
    watch_percent = 0.0;
  }

  var start_date = null_check(anime.start_date, null_check_date(anime.start_date), 0);
  var started_date = null_check(anime.started_at, null_check_date(anime.started_at), 0);
  var completed_date = null_check(anime.completed_at, null_check_date(anime.completed_at), 0);
  var cover_image = null_check(anime.cover_image, anime.cover_image.large, "./assets/no_cover_image.png");
  var average_score = null_check(anime.average_score, anime.average_score, 0);

  var display_browse = "none";
  var display_not_browse = "none";
  if (current_tab == "BROWSE" || current_tab == "RECOMMENDED") {
    display_browse = "block";
  } else {
    display_not_browse = "block";
  }

  var display_trailer = "none";
  if ((current_tab == "BROWSE" || current_tab == "RECOMMENDED") && anime.trailer != null) {
    display_trailer = "block";
  }

  var sort_value = await determine_sort_value(anime, user_data, score_format);
  var display_sort_value = "none";
  if (sort_value.length > 0) {
    display_sort_value = "block";
  }
  
  var display_airing_value = "none";
  var airing_value = "";
  var airing_at = 0;
  var airing_ep = 0;
  if (anime.next_airing_episode != null && show_airing == true) {
    display_airing_value = "block";
    airing_at = anime.next_airing_episode.airing_at * 1000;
    airing_ep = anime.next_airing_episode.episode;
  }

  var html = "";

  html += "<div id=\"" + anime.id + "\" class=\"cover_container\" date=\"" + start_date + "\" popularity=\"" + anime.popularity + "\" score=\"" + average_score + "\" title=\"" + title + "\" trending=\"" + anime.trending + "\" started=\"" + started_date + "\" completed=\"" + completed_date + "\">"
  html +=   "<img alt=\"Cover Image\" class=\"image\" height=\"300\" id=\"" + cover_id + "\" src=\"" + cover_image + "\" width=\"200\">"
  html +=   "<div class=\"airing_value_display\" style=\"display: " + display_airing_value + "; color: #f6f6f6;\"><p id=\"airing_value\" airing_at=\"" + airing_at + "\" airing_ep=\"" + airing_ep + "\">" + airing_value + "</p></div>"
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

var x = setInterval(function() {
  var elements = document.getElementsByClassName("airing_value_display");

  for(var i = 0; i < elements.length; i++) {

    var airing_at = parseInt(elements[i].childNodes[0].getAttribute("airing_at"));
    if (airing_at == 0) {
      continue;
    }
    var airing_ep = parseInt(elements[i].childNodes[0].getAttribute("airing_ep"));
    var date = new Date(airing_at);
    var now = new Date().getTime();
    var distance = date - now;

    if (distance < 0) {

      elements[i].childNodes[0].innerText = "Ep " + airing_ep + ": Aired"

    } else {

      var days = Math.floor(distance / (1000 * 60 * 60 * 24));
      var hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
      var minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));
      //var seconds = Math.floor((distance % (1000 * 60)) / 1000);

      elements[i].childNodes[0].innerText = "Ep " + airing_ep + ":";
      if (days > 0) { elements[i].childNodes[0].innerText += " " + days + "d"; }
      if (hours > 0) { elements[i].childNodes[0].innerText += " " + hours + "h"; }
      elements[i].childNodes[0].innerText += " " + minutes + "m";
    }
  }
}, 1000);

async function determine_sort_value(anime, user_data, score_format) {

  switch(document.getElementById("sort_order").value) {
    case "Alphabetical":
    case "Popularity":
    case "Trending":
      return "";
    case "Score":
      return null_check(anime.average_score, anime.average_score + "%", "??%");
    case "MyScore":
      if (user_data.score == 0) {
        return "No Score";
      }
      switch(score_format) {
        case "POINT_100":
          return user_data.score + "";
        case "POINT_10_DECIMAL":
          return user_data.score + "";
        case "POINT_10":
          return user_data.score + "";
        case "POINT_5":
          var text = "";
          for(var i = 0; i < user_data.score; i++) {
            text += "★";
          }
          for(var i = 0; i < (5 - user_data.score); i++) {
            text += "☆";
          }
          return text;
        case "POINT_3":
          switch(user_data.score) {
            case 1:
              return "🙁";
            case 2:
              return "😐";
            case 3:
              return "🙂";
          }
      }
      return "";
    case "Date":
      return null_check(anime.start_date, 
        null_check(anime.start_date.year, anime.start_date.year, "????") + 
        null_check(anime.start_date.month, "-" + anime.start_date.month, "-??") + 
        null_check(anime.start_date.day, "-" + anime.start_date.day, "-??"), 
        "????-??-??");
    case "Started":
      return null_check(user_data.started_at, 
        null_check(user_data.started_at.year, user_data.started_at.year, "????") + 
        null_check(user_data.started_at.month, "-" + user_data.started_at.month, "-??") + 
        null_check(user_data.started_at.day, "-" + user_data.started_at.day, "-??"), 
        "????-??-??");
    case "Completed":
      return null_check(user_data.completed_at, 
        null_check(user_data.completed_at.year, user_data.completed_at.year, "????") + 
        null_check(user_data.completed_at.month, "-" + user_data.completed_at.month, "-??") + 
        null_check(user_data.completed_at.day, "-" + user_data.completed_at.day, "-??"), 
        "????-??-??");
  }
}

function null_check(null_check, not_null_value, null_value) {
  if (null_check == null) {
    return null_value;
  } else {
    return not_null_value;
  }
}

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

async function redraw_episode_canvas() {

  var grid_children = document.getElementById("cover_panel_grid").childNodes;

  for(var i = 0; i < grid_children.length; i++) {

    if (grid_children[i].nodeType == 1) {

      var id = parseInt(grid_children[i].getAttribute("id"));
      var anime = await invoke("get_anime_info", {id: id});
      var user_data = await invoke("get_user_info", {id: id});
      draw_episode_canvas(user_data.progress, anime.episodes, id);

      // left side of episode text
      var episode_text = "";
      if (user_data != null) {
        episode_text = null_check(user_data.progress, user_data.progress + "/", "0/");
      } else {
        episode_text = "0/";
      }
      // right side of episode text
      episode_text += null_check(anime.episodes, anime.episodes, "??");
      document.getElementById("episode_text_" + anime.id).innerText = episode_text;
    }
  }
}

// fills in the episode progress bar to show episodes available on disk and episodes watched
window.draw_episode_canvas = draw_episode_canvas;
async function draw_episode_canvas(episode, total_episodes, media_id) {
  
  var watch_percent = 0.0;
  if (episode != null && Number.isNaN(total_episodes) == false && total_episodes != null) {
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
  if (Number.isNaN(total_episodes) == false && total_episodes != null) {
    bar.title = "Watched: " + episode + " / " + total_episodes;
  } else {
    bar.title = "Watched: " + episode + " / ??";
  }

  var ctx = bar.getContext("2d");
  ctx.clearRect(0,0,200,5);

  var width = bar.width / total_episodes;

  ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight-secondary');
  var episodes_exist = await invoke("episodes_exist_single", { id: media_id });

  if (Number.isNaN(total_episodes)) {
    var last_episode = Math.max(...episodes_exist);
    width = bar.width / last_episode;
  }

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
      if (i == episodes_exist.length - 1) {
        // draw rect until end
        ctx.fillRect((start - 1) * width, 0, width * length, 5);
      }
    }

    bar.title += episodes_on_disk_string(episodes_exist);
  }


  ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--highlight');
  ctx.fillRect(0, 0, watch_percent * 200, 5);
}

function episodes_on_disk_string(episodes_exist) {

  if (episodes_exist.length == 0) {
    return "";
  } else if (episodes_exist.length == 1) {
    return "\nEpisodes on disk: " + episodes_exist[0];
  }

  var start = -1;
  var end = -1;
  var text = "";

  for(var i = 0; i < episodes_exist.length; i++) {

    if (start == -1) {
      start = episodes_exist[i];
    } else if ((i + 1) == episodes_exist.length) {
      end = episodes_exist[i];
    } else if (episodes_exist[i] != episodes_exist[i + 1] - 1) {
      end = episodes_exist[i];
    }

    if (end != -1) {

      if (text.length != 0) {
        text += ", ";
      }

      if (start == end) {
        text += start;
      } else {
        text += start + "-" + end;
      }

      if (i + 1 < episodes_exist.length) {
        start = episodes_exist[i + 1];
      } else {
        start = -1;
      }
      end = -1;
    }
  }

  return "\nEpisodes on disk: " + text;
}

// enter key on search text
document.querySelector("#search_text").addEventListener("keyup", event => {
  if(event.key !== "Enter") return;
  browse_update();
  event.preventDefault();
});

// fill in the ui with anime retrieved from anilist based on the categories selected
window.browse_update = browse_update;
async function browse_update() {

  document.getElementById("loader").style.display = "inline-block";
  var year = document.getElementById("year_select").value;
  var season = document.getElementById("season_select").value;
  var format = document.getElementById("format_select").value;
  var genre = document.getElementById("genre_select").value;
  var search = document.getElementById("search_text").value;
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

  var list = await invoke("browse", {year: year, season: season, genre: genre, format: format, search: search, order: sort_value});

  removeChildren(document.getElementById("cover_panel_grid"));
  list_ids = [];
  for(var i = 0; i < list.length; i++) {
    if(user_settings.show_adult == false && list[i].is_adult == true) {
      continue;
    }
    add_anime(list[i], null, i, user_settings.score_format, user_settings.show_airing_time);
    list_ids.push(list[i].id);
  }
  //sort_anime();
  document.getElementById("loader").style.display = "none";
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
    if (Number.isNaN(total)) {
      text.textContent = progress + "/??";
    }

    draw_episode_canvas(progress, total, anime_id);
  }
}


// increases the users progress by 1
window.increase_episode = increase_episode;
async function increase_episode(anime_id) {

  await invoke("increment_decrement_episode", {animeId: anime_id, change: 1});

  var text = document.getElementById("episode_text_"+ anime_id);
  var episodes = text.textContent.split('/');
  var progress = parseInt(episodes[0]) + 1;
  var total = parseInt(episodes[1]);
  if (progress <= total || Number.isNaN(total)) {

    text.textContent = progress + "/" + total;
    if (Number.isNaN(total)){
      text.textContent = progress + "/??";
    }

    draw_episode_canvas(progress, total, anime_id);
  
    if (progress == total) {
      show_anime_list(current_tab);
    }
  }
}

// clears the date next the the button that has been pushed
window.clearDate = clearDate;
async function clearDate(date_id) {
  document.getElementById(date_id).value = "";
}

window.get_torrents = get_torrents;
async function get_torrents(anime_id) {
  exclusive_underline(7);
  invoke("get_torrents", {id: anime_id});
}

window.open_window = open_window;
async function open_window(url) {
  invoke("open_url", { url: url});
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

window.run_tests = run_tests;
async function run_tests() {

  var results = await invoke("run_filename_tests");
  console.log(results);
  removeChildren(document.getElementById("cover_panel_grid"));

  document.getElementById("cover_panel_grid").innerHTML = 
  "<table id=\"tests_table\" style=\"width:1600px;\">" + 
    "<tbody>" + 
      "<tr>" + 
        "<th>Filename</th>" + 
        "<th>Similarity Score</th>" + 
        "<th>Processed Title</th>" + 
        "<th>Title</th>" + 
        "<th>anime id</th>" + 
        "<th>Expected anime id</th>" + 
        "<th>Episode</th>" + 
        "<th>Expected Episode</th>" + 
        "<th>Resolution</th>" + 
        "<th>Expected Resolution</th>" + 
      "</tr>" + 
    "</tbody>" + 
  "</table>";

  var table = document.getElementById("tests_table");

  for(var i = 0; i < results.length; i++) {
    
    var score_color = "red";
    var episode_color = "red";
    var id_color = "red";
    var resolution_color = "red";
    if (results[i].similarity_score >= 0.65){ score_color = "lightgreen"; }
    if (results[i].episode == results[i].expected_episode){ episode_color = "lightgreen"; }
    if (results[i].anime_id == results[i].expected_anime_id){ id_color = "lightgreen"; }
    if (results[i].resolution == results[i].expected_resolution){ resolution_color = "lightgreen"; }

    var row = table.insertRow(i + 1);
    row.insertCell(0).innerHTML = results[i].filename;
    row.insertCell(1).innerHTML = "<p style=\"color:" + score_color + ";\">" + results[i].similarity_score.toFixed(3) + "</p>";
    row.insertCell(2).innerHTML = "<p>" + results[i].title + "</p>";
    row.insertCell(3).innerHTML = "<p>" + results[i].id_title + "</p>";
    row.insertCell(4).innerHTML = "<p style=\"color:" + id_color + ";\">" + results[i].anime_id + "</p>";
    row.insertCell(5).innerHTML = "<p style=\"color:" + id_color + ";\">" + results[i].expected_anime_id + "</p>";
    row.insertCell(6).innerHTML = "<p style=\"color:" + episode_color + ";\">" + results[i].episode + "</p>";
    row.insertCell(7).innerHTML = "<p style=\"color:" + episode_color + ";\">" + results[i].expected_episode + "</p>";
    row.insertCell(8).innerHTML = "<p style=\"color:" + resolution_color + ";\">" + results[i].resolution + "</p>";
    row.insertCell(9).innerHTML = "<p style=\"color:" + resolution_color + ";\">" + results[i].expected_resolution + "</p>";
  }
}



//// settings window


// shows the settings window
window.show_setting_window = show_setting_window;
async function show_setting_window() {
  get_user_settings();
  document.getElementById("login_panel").style.visibility = "visible";
  document.getElementById("cover_panel_grid").style.opacity = 0.3;
}



window.get_user_settings = get_user_settings;
async function get_user_settings() {
  
  var user_settings = await invoke("get_user_settings");
  
  document.getElementById("user_name").value = user_settings.username;
  document.getElementById("title_language").value = user_settings.title_language;
  document.getElementById("show_spoiler_tags").checked = user_settings.show_spoilers;
  document.getElementById("show_adult").checked = user_settings.show_adult;
  document.getElementById("show_airing").checked = user_settings.show_airing_time;
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



// hide the settings window and set the settings in rust
window.hide_setting_window = hide_setting_window;
async function hide_setting_window() {
  document.getElementById("login_panel").style.visibility = "hidden";
  document.getElementById("cover_panel_grid").style.opacity = 1;
  document.getElementById("first_time_setup").style.visibility = "hidden";
  document.getElementById("login_panel").style.setProperty("left", "50%");
  document.getElementById("login_panel").style.setProperty("transform", "translate(-50%,-50%)");

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
    show_airing_time: document.getElementById("show_airing").checked,
    folders: document.getElementById("folders").value.split('\n'),
    update_delay: parseInt(document.getElementById("update_delay").value),
    score_format: "",
    highlight_color: highlight_color,
    current_tab: "",
    first_time_setup: false,
  }

  await invoke("set_user_settings", { settings: settings});
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
  
  redraw_episode_canvas()
}



var background_color_1 = ["#1f2122", "#0e0e10", "#ffffff", "#f7f9fa", "#eaeded", "#eef2fe", "#feffef", "#edeeee" ];
var background_color_2 = ["#27292a", "#1f1f23", "#e5e5e5", "#edeeee", "#141921", "#d6dbef", "#ede0d7", "#dcdddd" ];
var text_color = ["#f6f6f6", "#f6f6f6", "#1c0000", "#000000", "#f6f6f6", "#000000", "#000000", "#000000"];
window.set_theme = set_theme;
async function set_theme(element, index) {

  var parent = document.getElementById("theme_boxes");
  var elements = parent.childNodes;

  for (var i=0; i<elements.length; i++) {

    if(elements[i].nodeType == 1) {
      elements[i].style.setProperty("border-style", "hidden");
      elements[i].style.setProperty("margin", "2.5px");
    }
  }
  element.style.setProperty("border-style", "solid");
  element.style.setProperty("margin", "0px");

  document.styleSheets[0].cssRules[0].style.setProperty("--background-color1", background_color_1[index]);
  document.styleSheets[0].cssRules[0].style.setProperty("--background-color2", background_color_2[index]);
  document.styleSheets[0].cssRules[0].style.setProperty("--text-color", text_color[index]);
}



//// anime info window



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
      user_settings = await invoke("get_user_settings");
    }
    var title_language = user_settings.title_language;
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

window.open_torrents_tab = open_torrents_tab;
function open_torrents_tab(tab_name, underline_name){
  openTab(tab_name, underline_name);
  add_torrent_data(info_window_anime_id);
}