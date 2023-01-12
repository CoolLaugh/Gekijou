const { invoke } = window.__TAURI__.tauri;



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