
export async function getInstruments() {
    var resp = await fetch(process.env.REACT_APP_LOCALHOST_PREFIX + "/get_windows")
    resp = await resp.json();
    return resp
}

export async function forceInstruments() {
    var resp = await fetch(process.env.REACT_APP_LOCALHOST_PREFIX + "/force_rescan")
    resp = await resp.json();
    return resp
}

export async function sendSettings(refresh, autohide, maxfps, multiple, alternate, autoStart) {
    var resp = await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/set_settings?refresh=${refresh}&autohide=${autohide}&maxfps=${maxfps}&multiple=${multiple}&alternate=${alternate}&autostart=${autoStart}`)
    return await resp.text()
}

export async function getSettings() {
    var resp = await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/settings`)
    resp = await resp.json();
    return resp
}

export async function getStatus() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/status`)).json()
}

export async function restoreWindows() {
    var resp = await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/restore_windows`)
    resp = await resp.text();
    return resp
}

export async function hideWindows() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/hide_windows`)).text()
}

export async function reconnectBridge() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/reconnect`)).text()
}

export async function setHwnd(hwnd) {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/set_hwnd?hwnd=${hwnd}`)).text()
}

export async function saveDebug() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/save_debug`)).text()
}

export async function instrAutoHide(hwnd, auto_hide) {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/set_hwnd_settings?hwnd=${hwnd}&autohide=${auto_hide}`)).text()
}

export async function instrExclude(hwnd, excluded) {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/set_hwnd_settings?hwnd=${hwnd}&excluded=${excluded}`)).text()
}

export async function getAddonConfig() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/static/addon_config.json`)).json()
}

export async function getAircraftFile() {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/get_aircraft`)).text()
}
export async function setFMS(hwnd) {
    return (await fetch(process.env.REACT_APP_LOCALHOST_PREFIX +
        `/set_hwnd?hwnd=${hwnd}`)).text()
}