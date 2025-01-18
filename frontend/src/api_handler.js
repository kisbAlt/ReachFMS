
export async function getInstruments() {
    var resp = await fetch(getServerAddr() + "/get_windows")
    resp = await resp.json();
    return resp
}

export async function forceInstruments() {
    var resp = await fetch(getServerAddr() + "/force_rescan")
    resp = await resp.json();
    return resp
}

export async function sendSettings(refresh, autohide, maxfps, multiple, alternate, autoStart) {
    var resp = await fetch(getServerAddr() +
        `/set_settings?refresh=${refresh}&autohide=${autohide}&maxfps=${maxfps}&multiple=${multiple}&alternate=${alternate}&autostart=${autoStart}`)
    return await resp.text()
}

export async function getSettings() {
    var resp = await fetch(getServerAddr() +
        `/settings`)
    resp = await resp.json();
    return resp
}

export async function getStatus() {
    return (await fetch(getServerAddr() +
        `/status`)).json()
}

export async function restoreWindows() {
    var resp = await fetch(getServerAddr() +
        `/restore_windows`)
    resp = await resp.text();
    return resp
}

export async function hideWindows() {
    return (await fetch(getServerAddr() +
        `/hide_windows`)).text()
}

export async function reconnectBridge() {
    return (await fetch(getServerAddr() +
        `/reconnect`)).text()
}

export async function setHwnd(hwnd) {
    return (await fetch(getServerAddr() +
        `/set_hwnd?hwnd=${hwnd}`)).text()
}

export async function saveDebug() {
    return (await fetch(getServerAddr() +
        `/save_debug`)).text()
}

export async function instrAutoHide(hwnd, auto_hide) {
    return (await fetch(getServerAddr() +
        `/set_hwnd_settings?hwnd=${hwnd}&autohide=${auto_hide}`)).text()
}

export async function instrExclude(hwnd, excluded) {
    return (await fetch(getServerAddr() +
        `/set_hwnd_settings?hwnd=${hwnd}&excluded=${excluded}`)).text()
}

export async function getAddonConfig() {
    return (await fetch(getServerAddr() +
        `/static/addon_config.json`)).json()
}

export async function getAircraftFile() {
    return (await fetch(getServerAddr() +
        `/get_aircraft`)).text()
}
export async function setFMS(hwnd) {
    return (await fetch(getServerAddr() +
        `/set_hwnd?hwnd=${hwnd}`)).text()
}

export async function sendTouchEvent(xPos, yPos) {
    return (await fetch(getServerAddr() +
        `/touch_event?x_pos=${xPos}&y_pos=${yPos}`)).text()
}

export async function btnEvent(btn_name) {
    return (await fetch(getServerAddr() +
        `/mcdu_btn_press?btn=${btn_name}`)).text()
}

function getServerAddr() {
    let curr_location = window.location.origin;
    if (curr_location.includes("3000")) {
        return curr_location.replace("3000", "5273")
    }else {
        return curr_location
    }
}