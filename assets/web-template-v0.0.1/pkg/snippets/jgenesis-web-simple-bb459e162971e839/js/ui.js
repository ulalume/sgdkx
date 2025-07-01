export function showUi() {
    document.getElementById("jgenesis").classList.remove("hidden");
}

/**
 * @param fullscreen {boolean}
 */
export function setFullscreen(fullscreen) {
    document.querySelectorAll(".hide-fullscreen").forEach((element) => {
        element.hidden = fullscreen;
        if (fullscreen) {
            element.classList.add("hidden");
        } else {
            element.classList.remove("hidden");
        }
    });
}

export function focusCanvas() {
    document.querySelector("canvas").focus();
}

export function showSmsGgConfig() {
}

export function showGenesisConfig() {
}

export function showSnesConfig() {
}

/**
 * @param visible {boolean}
 */
export function setCursorVisible(visible) {
    let canvas = document.querySelector("canvas");
    if (visible) {
        canvas.classList.remove("cursor-hidden");
    } else {
        canvas.classList.add("cursor-hidden");
    }
}

/**
 * @param romTitle {string}
 */
export function setRomTitle(romTitle) {
    console.log("Setting ROM title to:", romTitle);
}

/**
 * @param saveUiEnabled {boolean}
 */
export function setSaveUiEnabled(saveUiEnabled) {
    let saveButtons = document.getElementsByClassName("save-button");
    if (saveUiEnabled) {
        for (let i = 0; i < saveButtons.length; i++) {
            saveButtons[i].removeAttribute("disabled");
        }
    } else {
        for (let i = 0; i < saveButtons.length; i++) {
            saveButtons[i].setAttribute("disabled", "");
        }
    }
}

/**
 * @param key {string}
 * @return {string | null}
 */
export function localStorageGet(key) {
    return localStorage.getItem(key);
}

/**
 * @param key {string}
 * @param value {string}
 */
export function localStorageSet(key, value) {
    localStorage.setItem(key, value);
}