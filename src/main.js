const { invoke } = window.__TAURI__.tauri;

loadScript('/settings_window.js');
loadScript('/anime_info_window.js');

function loadScript(url)
{    
    var head = document.getElementsByTagName('head')[0];
    var script = document.createElement('script');
    script.type = 'text/javascript';
    script.src = url;
    head.appendChild(script);
}

window.addEventListener("DOMContentLoaded", async () => {

  populate_year_dropdown();

  document.getElementById("information").style.display = "block";
  document.getElementById("underline_tab_0").style.visibility = "visible";

  await invoke("load_user_settings");

  var user_settings = await invoke("get_user_settings");
  document.styleSheets[0].cssRules[0].style.setProperty("--highlight", user_settings.highlight_color);

  add_adult_genres(user_settings.show_adult);

  if (user_settings.first_time_setup == true) {

    show_setting_window();
    document.getElementById("login_panel").style.setProperty("left", "100%");
    document.getElementById("login_panel").style.setProperty("transform", "translate(-102%,-50%)");
    document.getElementById("first_time_setup").style.visibility = "visible";

  } else {
    
    await invoke("on_startup");

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
  invoke("close_splashscreen");
  check_for_refresh_ui();
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
async function check_for_refresh_ui() {

  while (true) {

    var refresh = await invoke("get_refresh_ui");
    if (refresh.anime_list == true) {
      show_anime_list(current_tab);
    }
    if (refresh.canvas == true && current_tab != "BROWSE") {
      redraw_episode_canvas();
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
  document.getElementById("browse_filters").style.display = "block";
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

  for(var i = 0; i < 8; i++) {
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

window.show_recommended_anime_list = show_recommended_anime_list;
async function show_recommended_anime_list() {

  if (current_tab == "RECOMMENDED") {
    return;
  }
  current_tab = "RECOMMENDED";
  exclusive_underline(6);
  document.getElementById("cover_panel_id").onscroll = null;

  var name = current_tab;
  
  document.getElementById("browse_filters").style.display = "none";

  document.getElementById("cover_panel_grid").innerHTML = "<a>Wait one moment<a>";

  var recommended_list = await invoke("recommend_anime");

  var user_settings = await invoke("get_user_settings");

  // user didn't change the tab while getting the list from anilist
  if (name == current_tab) {

    document.getElementById("cover_panel_grid").innerHTML = "";
    removeChildren(document.getElementById("cover_panel_grid"));
    
    for(var i = 0; i < recommended_list.length; i++) {
      if(user_settings.show_adult == false && recommended_list[i].is_adult == true) {
        continue;
      }
      await add_anime(recommended_list[i], null, i, user_settings.score_format, user_settings.show_airing_time);
    }
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
  for(var i = 0; i < list.length; i++) {
    if(user_settings.show_adult == false && list[i].is_adult == true) {
      continue;
    }
    add_anime(list[i], null, i, user_settings.score_format, user_settings.show_airing_time);
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

window.get_torrents = get_torrents;
async function get_torrents() {
  exclusive_underline(7);
  invoke("get_torrents", {search: ""});
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