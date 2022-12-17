const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", () => {
  invoke("on_startup");
  get_user_settings();

  document.getElementById("information").style.display = "block";
  document.getElementById("underline_tab_0").style.visibility = "visible";
  invoke("anime_update_delay_loop");
  check_for_refresh_ui();
});

async function check_for_refresh_ui() {

  while (true) {

    var refresh = await invoke("get_refresh_ui");
    if (refresh == true) {
      show_anime_list(current_tab);
    }
    sleep(1000);
  }
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function get_user_settings() {
  
  var user_settings = await invoke("get_user_settings");
  
  document.getElementById("user_name").value = user_settings.username;
  document.getElementById("title_language").value = user_settings.title_language;
  document.getElementById("show_spoiler_tags").checked = user_settings.show_spoilers;
  document.getElementById("show_adult").checked = user_settings.show_adult;
}

// add anime for testing
async function test_add_anime() {
  
  //var anime_ids = [5114,9253,21202,17074,2904,114745,7311,437,109190,21366,21860,21,17871,19221];

  //for (let i = 0; i < anime_ids.length; i++) {
  //  add_anime(anime_ids[i], i);
  //}
}

async function open_oauth_window() {
  window.open("https://anilist.co/api/v2/oauth/authorize?client_id=9965&redirect_uri=https://anilist.co/api/v2/oauth/pin&response_type=code");
}

async function get_oauth_token() {
  var input = document.getElementById("oauth_code")
  console.log(input.value);
  var success = await invoke("anilist_oauth_token", { code: document.getElementById("oauth_code").value});

  input.value = "";
  if(success == true) {
    input.setAttribute("placeholder", "Success");
  } else {
    input.setAttribute("placeholder", "Failed");
  }
}

async function hide_setting_window() {
  document.getElementById("login_panal").style.visibility = "hidden";
  document.getElementById("cover_panal_grid").style.opacity = 1;

  var username = document.getElementById("user_name").value;
  var language = document.getElementById("title_language").value;
  var show_spoiler = document.getElementById("show_spoiler_tags").checked
  var show_adult = document.getElementById("show_adult").checked

  invoke("set_user_settings", { username: username, titleLanguage: language, showSpoilers: show_spoiler, showAdult: show_adult});
}

async function show_setting_window() {
  document.getElementById("login_panal").style.visibility = "visible";
  document.getElementById("cover_panal_grid").style.opacity = 0.3;
}

var current_tab = "";
async function show_watching_anime() {
  current_tab = "CURRENT";
  show_anime_list(current_tab);
  exclusive_underline(0);
}

async function show_completed_anime() {
  current_tab = "COMPLETED";
  show_anime_list(current_tab);
  exclusive_underline(1);
}

async function show_paused_anime() {
  current_tab = "PAUSED";
  show_anime_list(current_tab);
  exclusive_underline(2);
}

async function show_dropped_anime() {
  current_tab = "DROPPED";
  show_anime_list(current_tab);
  exclusive_underline(3);
}

async function show_planning_anime() {
  current_tab = "PLANNING";
  show_anime_list(current_tab);
  exclusive_underline(4);
}

function exclusive_underline(index) {

  for(var i = 0; i < 6; i++) {
    document.getElementById("underline" + i).style.visibility = "hidden";
  }
  document.getElementById("underline" + index).style.visibility = "visible";
}

async function show_anime_list(name) {

  var watching = await invoke("get_list", { listName: name });
  console.log(watching);
  var user_data = await invoke("get_list_user_info", { listName: name });
  // get userdata on anime
  console.log(user_data);
  // add anime to UI
  removeChilds(document.getElementById("cover_panal_grid"));

  for(var i = 0; i < watching.length; i++) {
    add_anime(watching[i], user_data[i], i);
  }
}

const removeChilds = (parent) => {
  while (parent.lastChild) {
      parent.removeChild(parent.lastChild);
  }
};

async function test() {

  // var bars = document.getElementsByClassName("episodes_exist");
  // for(var i = 0; i < bars.length; i++) {
  //   var ctx = bars[i].getContext("2d");
  //   ctx.fillStyle = "#808080";
  //   ctx.fillRect(0, 0, 46, 5);
  //   ctx.fillRect(77, 0, 46, 5);
  //   ctx.fillRect(154, 0, 30, 5);
  // }

  //console.log("test fn started");
  var response = await invoke("test");
  //console.log(response);
}

// add an anime to the ui
async function add_anime(anime, user_data, cover_id) {

  var title = "No Title";
  if(anime.title.english != null){
    title = anime.title.english;
  } else if (anime.title.romaji != null) {
    title = anime.title.romaji;
  } else if (anime.title.native != null) {
    title = anime.title.native;
  }

  var watch_percent = (user_data.progress / anime.episodes) * 100;
  if (watch_percent > 100) {
    watch_percent = 100;
  } else if (watch_percent < 0) {
    watch_percent = 0;
  }

  document.getElementById("cover_panal_grid").insertAdjacentHTML("beforeend", 
  "<div anime_id=" + anime.id + " class=\"cover_container\" date=" + (anime.start_date.year * 10000 + anime.start_date.month * 100 + anime.start_date.day) + " popularity=" + anime.popularity + " score=" + anime.average_score + " title=\"" + title + "\">" +
    "<img alt=\"Cover Image\" class=\"image\" height=\"300\" id=\"" + cover_id + "\" src=" + anime.cover_image.large + " width=\"200\">" +
    "<button class=\"big_play_button\" onclick=\"play_next_episode(" + anime.id + ")\" type=\"button\"><img ,=\"\" height=\"80\" src=\"assets/play2.png\" width=\"80\"></button>" +
    "<div class=\"cover_nav\">" +
      "<a href=\"#\" onclick=\"show_anime_info_window(" + anime.id + ")\" style=\"border-top-left-radius: 12px; border-bottom-left-radius:12px\">info</a>" +
      "<a href=\"#\" onclick=\"decrease_episode(" + anime.id + ")\">-</a>" +
      "<a href=\"#\" onclick=\"\">" + user_data.progress + "/" + anime.episodes + "</a>" +
      "<a href=\"#\" onclick=\"increase_episode(" + anime.id + ")\" style=\"border-top-right-radius: 12px; border-bottom-right-radius:12px\">+</a>" +
    "</div>" +
    "<div class=\"myProgress\">" +
      "<div class=\"myBar\" id=\"Bar" + cover_id + "\" style=\"width: " + watch_percent + "%;\"></div>" +
      "<canvas class=\"episodes_exist\" height=\"5\" id=\"progress_episodes" + cover_id + "\" width=\"200\"></canvas>" +
    "</div>" +
    "<div class=\"cover_title\">" +
      "<p id=\"title" + anime.id + "\">" + title + "</p>" +
    "</div>" +
  "</div>");

  sort_anime();
}

async function play_next_episode(id) {
  await invoke("play_next_episode", { id: id });
}

// hide information window and return to cover grid
async function hide_anime_info_window(anime_id) {
  document.getElementById("youtube_embed").src = "";
  document.getElementById("info_panal").style.display = "none";
  document.getElementById("cover_panal_grid").style.opacity = 1;
  var refresh = await update_user_entry(anime_id);
  if (refresh == true) {
    show_anime_list(current_tab);
  }
}

// show information window populated with the shows info
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
  console.log(user_data);
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
  document.getElementById("info_panal").style.display = "block";
  document.getElementById("cover_panal_grid").style.opacity = 0.3;
}

async function decrease_episode(anime_id) {

}

async function increase_episode(anime_id) {

}

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



function openTab(tab_name, underline_name) {

  // Get all elements with class="tabcontent" and hide them
  var tabcontent = document.getElementsByClassName("tabcontent");
  for (var i = 0; i < tabcontent.length; i++) {
    tabcontent[i].style.display = "none";
  }

  // Get all elements with class="tab_underline" and hide them
  var tabcontent = document.getElementsByClassName("tab_underline");
  for (var i = 0; i < tabcontent.length; i++) {
    tabcontent[i].style.visibility = "hidden";
  }

  // Show the current tab, and an underline to the button that opened the tab
  document.getElementById(tab_name).style.display = "block";
  document.getElementById(underline_name).style.visibility = "visible";
}

async function clearDate(date_id) {
  document.getElementById(date_id).value = "";
}

// list of categories that can be searched by
// variables are field name, display name, and default sorting order
const sort_categories = [["name", "Alphabetical", true], ["score","Score", false], ["date","Date", true], ["populariry","Populariry", false]];
var sort_categorie_index = 0;
var sort_ascending = true;

// cycle through different ways of sorting shows
async function change_sort_type() {

  sort_categorie_index = (sort_categorie_index + 1) % sort_categories.length;
  sort_ascending = sort_categories[sort_categorie_index][2];

  document.getElementById("sort_order_text").textContent = sort_categories[sort_categorie_index][1];

  change_ascending_indicator()

  console.log(sort_categorie_index);
  console.log(sort_categories[sort_categorie_index]);

  sort_anime();
}

// change between sorting ascending and decending
async function change_sort_ascending() {
  sort_ascending = !sort_ascending;
  change_ascending_indicator()
  sort_anime();
}

// change the image to show if the list is being sorted ascending or decending
function change_ascending_indicator() {
  if(sort_ascending == true) {
    document.getElementById("sort_order_ascending").style.transform = 'rotate(180deg)';
  }
  else {
    document.getElementById("sort_order_ascending").style.transform = 'rotate(0deg)';
  }
}

// sort covers according to the current category and order
async function sort_anime() {

  var container = document.getElementById("cover_panal_grid");
  var elements = container.childNodes;
  var sortMe = [];

  for (var i=0; i<elements.length; i++) {
      
    if (elements[i].nodeType == 1) {

      var id = parseInt(elements[i].getAttribute("anime_id"), 10);

      switch(sort_categorie_index) {
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

  sortMe.sort();
  if (sort_ascending == false) {
    sortMe.reverse();
  }

  for (var i=0; i<sortMe.length; i++) {
      container.appendChild(sortMe[i][1]);
  }
}

async function exitWindow() {
  window.close();
}

async function minimizeWindow() {
  window.minimize();
}

async function toggleMaximizeWindow() {
  window.toggleMaximizeWindow();
}

window.increase_episode = increase_episode;
window.decrease_episode = decrease_episode;
window.play_next_episode = play_next_episode;
window.clearDate = clearDate;
window.openTab = openTab;
window.show_anime_list = show_anime_list;
window.show_watching_anime = show_watching_anime;
window.show_completed_anime = show_completed_anime;
window.show_paused_anime = show_paused_anime;
window.show_dropped_anime = show_dropped_anime;
window.show_planning_anime = show_planning_anime;
window.get_user_settings = get_user_settings;
window.hide_setting_window = hide_setting_window;
window.show_setting_window = show_setting_window;
window.get_oauth_token = get_oauth_token;
window.open_oauth_window = open_oauth_window;
window.test = test;
window.change_sort_ascending = change_sort_ascending;
window.change_sort_type = change_sort_type;
window.sort_anime = sort_anime;
window.show_anime_info_window = show_anime_info_window;
window.hide_anime_info_window = hide_anime_info_window;
window.add_anime = add_anime;
window.exitWindow = exitWindow;
window.minimizeWindow = minimizeWindow;
window.toggleMaximizeWindow = toggleMaximizeWindow;