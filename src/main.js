const { invoke } = window.__TAURI__.tauri;

let greetInputEl;
let greetMsgEl;

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
});

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

async function getanime(animeID, button_id) {
  greetMsgEl.textContent = await invoke("get_cover_image", { id: animeID });
  document.getElementById(button_id).src = greetMsgEl.textContent;
}

async function exitWindow() {
  window.close();
}

async function minimizeWindow() {
  greetMsgEl.textContent = "minimize";
  window.minimize();
}

async function toggleMaximizeWindow() {
  greetMsgEl.textContent = "maximize";
  window.toggleMaximizeWindow();
}

window.greet = greet;
window.getanime = getanime;
window.exitWindow = exitWindow;
window.minimizeWindow = minimizeWindow;
window.toggleMaximizeWindow = toggleMaximizeWindow;