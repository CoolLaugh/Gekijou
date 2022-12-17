const { invoke } = window.__TAURI__.tauri;
var anime_info = new Map();

window.addEventListener("DOMContentLoaded", () => {
  test_add_anime();
  invoke("read_token_data");
});

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
  invoke("anilist_oauth_token", { code: document.getElementById("oauth_code").value});
}

async function test() {

  console.log("test fn started");
  var response = await invoke("test");
  console.log(response);
}

// add an anime to the ui
async function add_anime(anime_id, cover_id) {

  if(anime_info.has(anime_id) == false) {
    anime_info.set(anime_id, await invoke("get_anime_info_query", { id: anime_id }));
  }
  console.log(typeof anime_id);
  document.getElementById("cover_panal_grid").insertAdjacentHTML("beforeend", 
  "<div class=\"cover_container\" anime_id=" + anime_id + ">" +
    "<img class=\"image\" src=" + anime_info.get(anime_id).cover_image.large + " id=\"" + cover_id + "\" alt=\"Cover Image\" width=\"200\" height=\"300\"/>" +
    "<button class=\"cover_play_button\" type=\"button\" onclick=\"getanime(" + anime_id + ", " + cover_id + ")\">Play</button>" +
    "<button class=\"cover_info_button\" type=\"button\" onclick=\"show_anime_info_window(" + anime_id + ")\">Info</button>" +
    "<div class=\"myProgress\">" +
      "<div class=\"myBar\" id=\"Bar" + cover_id + "\"></div>" +
    "</div>" +
    "<div class=\"cover_title\">" +
      "<p id=\"title" + anime_id + "\">" + anime_info.get(anime_id).title.english + "</p>" +
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
          sortMe.push([ anime_info.get(id).title.english , elements[i] ]);
          break;
        case 1:
          sortMe.push([ anime_info.get(id).average_score , elements[i] ]);
          break;
        case 2:
          sortMe.push([ anime_info.get(id).start_date.year * 10000 + anime_info.get(id).start_date.month * 100 + anime_info.get(id).start_date.day , elements[i] ]);
          break;
        case 3:
          sortMe.push([ anime_info.get(id).popularity , elements[i] ]);
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