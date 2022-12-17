const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", () => {
  invoke("read_token_data");
  get_user_settings();
});

async function get_user_settings() {
  
  var user_settings = await invoke("get_user_settings");
  
  document.getElementById("user_name").value = user_settings.username;
  document.getElementById("title_language").value = user_settings.title_language;
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

  var un = document.getElementById("user_name").value;
  var lang = document.getElementById("title_language").value;

  console.log(un + " " + lang + "\n");

  invoke("set_user_settings", { username: un, titleLanguage: lang});
}

async function show_setting_window() {
  document.getElementById("login_panal").style.visibility = "visible";
  document.getElementById("cover_panal_grid").style.opacity = 0.3;
}

async function show_watching_anime() {

  var watching = await invoke("get_watching_list", { listName: "Watching" });
  console.log(watching);
  // get userdata on anime

  // add anime to UI
  var cover_id = 0;
  watching.forEach(function(anime) {
    console.log(anime);
    add_anime(anime, cover_id);
    cover_id += 1;
  });
}

async function show_completed_anime() {

  var watching = await invoke("get_watching_list", { listName: "Completed" });
  console.log(watching);
  // get userdata on anime

  // add anime to UI
  var cover_id = 0;
  watching.forEach(function(anime) {
    console.log(anime);
    add_anime(anime, cover_id);
    cover_id += 1;
  });
}

async function test() {

  console.log("test fn started");
  var response = await invoke("test");
  console.log(response);
}

// add an anime to the ui
async function add_anime(anime, cover_id) {

  var title = "No Title";
  if(anime.title.english != null){
    title = anime.title.english;
  } else if (anime.title.romaji != null) {
    title = anime.title.romaji;
  } else if (anime.title.native != null) {
    title = anime.title.native;
  }

  document.getElementById("cover_panal_grid").insertAdjacentHTML("beforeend", 
  "<div class=\"cover_container\" anime_id=" + anime.id + " title=\"" + title + "\" score=" + anime.average_score + " date=" + (anime.start_date.year * 10000 + anime.start_date.month * 100 + anime.start_date.day) + " popularity=" + anime.popularity + ">" +
    "<img class=\"image\" src=" + anime.cover_image.large + " id=\"" + cover_id + "\" alt=\"Cover Image\" width=\"200\" height=\"300\"/>" +
    "<button class=\"cover_play_button\" type=\"button\" onclick=\"getanime(" + anime.id + ", " + cover_id + ")\">Play</button>" +
    "<button class=\"cover_info_button\" type=\"button\" onclick=\"show_anime_info_window(" + anime.id + ")\">Info</button>" +
    "<div class=\"myProgress\">" +
      "<div class=\"myBar\" id=\"Bar" + cover_id + "\"></div>" +
    "</div>" +
    "<div class=\"cover_title\">" +
      "<p id=\"title" + anime.id + "\">" + title + "</p>" +
    "</div>" +
  "</div>");

  sort_anime();
}

// hide information window and return to cover grid
async function hide_anime_info_window() {
  document.getElementById("info_panal").style.visibility = "hidden";
  document.getElementById("cover_panal_grid").style.opacity = 1;
}

// show information window populated with the shows info
async function show_anime_info_window(anime_id) {
  var info = anime_info.get(anime_id);

  document.getElementById("info_cover").src = info.cover_image.large;
  document.getElementById("info_description").insertAdjacentHTML("afterbegin", info.description)
  if(info.title.english.length > 55) {
    document.getElementById("info_title").textContent = info.title.english.substring(0, 55) + "...";
  } else {
    document.getElementById("info_title").textContent = info.title.english;
  }
  if (info.format != "TV") {
    document.getElementById("info_format").textContent = info.format.charAt(0) + info.format.toLowerCase().slice(1);
  } else {
    document.getElementById("info_format").textContent = info.format;
  }
  document.getElementById("info_rating").textContent = info.average_score + "%";
  if (info.episodes == 1) {
    document.getElementById("info_duration").textContent = info.duration + " Minutes";
  } else {
    document.getElementById("info_duration").textContent = info.episodes + " x " + info.duration + " Minutes";
  }
  document.getElementById("info_season_year").textContent = info.season.charAt(0) + info.season.toLowerCase().slice(1) + " " + info.season_year;


  document.getElementById("info_panal").style.visibility = "visible";
  document.getElementById("cover_panal_grid").style.opacity = 0.3;
  console.log(anime_info.get(anime_id));
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
          sortMe.push([ elements[i].getAttribute("title") , elements[i] ]);
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

window.show_completed_anime = show_completed_anime;
window.show_watching_anime = show_watching_anime;
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