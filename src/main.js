const { invoke } = window.__TAURI__.tauri;

window.addEventListener("DOMContentLoaded", () => {
  test_add_anime();
});

async function getanime(anime_id, button_id) {
  document.getElementById(button_id).src = await invoke("get_cover_image", { id: anime_id });
}

async function test_add_anime(){
  add_anime(5114, 1);
  add_anime(9253, 2);
  add_anime(21202, 3);
  add_anime(17074, 4);
  add_anime(2904, 5);
  add_anime(114745, 6);
  add_anime(7311, 7);
  add_anime(437, 8);
  add_anime(109190, 9);
  add_anime(21366, 10);
}

async function add_anime(anime_id, cover_id) {
  document.getElementById("cover_panal_grid").insertAdjacentHTML("beforeend", 
  "<div class=\"cover_container\">" +
    "<img src=" + await invoke("get_cover_image", { id: anime_id }) + " id=\"" + cover_id + "\" alt=\"Cover Image\" width=\"200\" height=\"300\"/>" +
    "<button class=\"cover_play_button\" type=\"button\" onclick=\"getanime(" + anime_id + ", " + cover_id + ")\">Play</button>" +
    "<button class=\"cover_info_button\" type=\"button\" onclick=\"show_anime_info_window(" + anime_id + ")\">Info</button>" +
    "<div class=\"myProgress\">" +
      "<div class=\"myBar\" id=\"Bar" + cover_id + "\"></div>" +
    "</div>" +
  "</div>");
}

async function hide_anime_info_window() {
  document.getElementById("info_panal").style.visibility = "hidden";
  document.getElementById("cover_panal_grid").style.opacity = 1;
}

async function show_anime_info_window(anime_id) {
  document.getElementById("info_cover").src = await invoke("get_cover_image", { id: anime_id });
  document.getElementById("info_panal").style.visibility = "visible";
  document.getElementById("cover_panal_grid").style.opacity = 0.3;
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

window.show_anime_info_window = show_anime_info_window;
window.hide_anime_info_window = hide_anime_info_window;
window.getanime = getanime;
window.add_anime = add_anime;
window.exitWindow = exitWindow;
window.minimizeWindow = minimizeWindow;
window.toggleMaximizeWindow = toggleMaximizeWindow;