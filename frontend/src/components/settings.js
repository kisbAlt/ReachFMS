import {useState, useEffect} from 'react';
import {
    getAddonConfig,
    getStatus,
    hideWindows,
    instrAutoHide, instrExclude,
    reconnectBridge,
    restoreWindows,
    saveDebug,
    sendSettings
} from "../api_handler";
import {Tick} from "./tick";
import {getAircraftConfig} from "../config_handler";


export function _arrayBufferToBase64(buffer) {
    var binary = '';
    var bytes = new Uint8Array(buffer);
    var len = bytes.byteLength;
    for (var i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary);
}


export function SettignsComponent(props) {
    const [autoChecked, setAutoChecked] = useState(true)
    const [maxChecked, setMaxChecked] = useState(true)
    const [alternateChecked, setAlternateChecked] = useState(false)
    const [multipleChecked, setMultipleChecked] = useState(true)
    const [bridgeConnected, setBridgeConnected] = useState(false)
    const [loadedAircraft, setLoadedAircraft] = useState("")
    const [instrumentSettings, setInstrumentSettings] = useState([])
    const [autoStartServer, setAutoStartServer] = useState(false)
    const [showConfigs, setShowConfigs] = useState(false)
    const [allConfigs, setAllConfigs] = useState([])
    const [versions, setVersions] = useState({"version": 0, "date": ""})


    function checkerHandler() {
        if (autoChecked) {
            setAutoChecked(false)
        } else {
            setAutoChecked(true)
        }
    }

    function checkerAutoStartup() {
        if (autoStartServer) {
            setAutoStartServer(false)
        } else {
            setAutoStartServer(true)
        }
    }

    function checkerHandlerMultiple() {
        if (multipleChecked) {
            setMultipleChecked(false)
        } else {
            setMultipleChecked(true)
        }
    }

    async function reconnect() {
        await reconnectBridge();
        await loadSave();
    }

    async function loadAllAddons() {
        setShowConfigs(true)
        let addons_ls = await getAddonConfig();
        setAllConfigs(addons_ls.aircraft_addons)
    }

    async function sendSave() {
        var selected = document.getElementById("refresh").value;
        var refresh_rate = 200;
        switch (selected) {
            case "slow":
                refresh_rate = 450;
                break;
            case "normal":
                refresh_rate = 200;
                break;
            case "fast":
                refresh_rate = 100;
                break;
            case "veryfast":
                refresh_rate = 50;
                break;
            default:
                refresh_rate = 200;
                break;
        }
        props.setRefresh(refresh_rate)
        props.setTiff(!maxChecked)
        await sendSettings(refresh_rate, autoChecked, maxChecked, multipleChecked, alternateChecked, autoStartServer)
        props.showNotification("Settings saved.")
        props.refreshFunction()
    }

    async function loadSave() {
        var status = await getStatus();
        var settings = status.settings;
        var refresh_setting = "normal"

        props.setRefresh(settings.refresh_rate)
        switch (settings.refresh_rate) {
            case 450:
                refresh_setting = "slow";
                break;
            case 200:
                refresh_setting = "normal";
                break;
            case 100:
                refresh_setting = "fast";
                break;
            case 50:
                refresh_setting = "veryfast";
                break;
            default:
                refresh_setting = "normal";
                break;
        }
        setAutoChecked(settings.auto_hide)
        setMaxChecked(settings.max_fps)
        setMultipleChecked(settings.multiple_displays)
        setAlternateChecked(settings.cpu_displays)
        setAutoStartServer(settings.auto_start)
        props.setTiff(!settings.max_fps)
        document.getElementById("refresh").value = refresh_setting;

        setBridgeConnected(status.bridge_status.connected)

        let aircraft_config = await getAircraftConfig(props.instrumentObjects);
        console.log(aircraft_config)
        if (aircraft_config != null) {
            setVersions({"version": aircraft_config.version, "date": aircraft_config.updated})
            if ("display" in aircraft_config) {
                setLoadedAircraft(aircraft_config.display)
                if (aircraft_config.touch_enabled) {
                    if (settings.auto_hide) {
                        props.showNotification("Pop-out auto hiding is NOT supported with touch instruments. You need to have the " +
                            "Pop-out window visible on a monitor at all times for this experimental feature to work!", true, 15)
                    } else {
                        props.showNotification("It seems like that you are using a touch instrument. This feature is highly " +
                            "experimental. Make sure that you have the pop-out window visible on a monitor at all times.", false, 10);
                    }
                }
            }
        }

    }

    useEffect(() => {
        var temp_array = [];
        for (let i = 0; i < props.instrumentObjects.length; i++) {
            temp_array.push({
                "auto_hide": props.instrumentObjects[i].auto_hide,
                "excluded": props.instrumentObjects[i].excluded,
                "jpeg_bytes": props.instrumentObjects[i].jpeg_bytes,
                "instrument": props.instrumentObjects[i].instrument,
                "hwnd": props.instrumentObjects[i].hwnd,
                "selected": props.instrumentObjects[i].selected,
            })
        }
        setInstrumentSettings(temp_array)
        loadSave();
        // eslint-disable-next-line
    }, [props.instrumentObjects]);

    useEffect(() => {

        // eslint-disable-next-line
    }, []);

    return (
        <div style={{overflow: "scroll", maxHeight: "100%"}}>
            {showConfigs && (<div style={{
                width: "100vw", position: "absolute", opacity: "0.9",
                height: "100vh", backgroundColor: "black"
            }}>
                <div style={{marginLeft: "auto", marginRight: "auto", width: "fit-content", marginTop: "10%"}}>
                    <table style={{borderColor: "gray"}}>
                        <thead>
                        <tr>
                            <th>Aircraft</th>
                            <th>Last updated</th>
                        </tr>
                        </thead>
                        <tbody>
                        {allConfigs.map((item, i) => (
                            <tr key={i}>
                                <td>{item.display}</td>
                                <td>{item.last_updated}</td>
                            </tr>


                        ))}
                        </tbody>
                    </table>
                    <p onClick={() => setShowConfigs(false)}
                       style={{
                           padding: "8px", backgroundColor: "darkorange", borderRadius: "5px", marginRight: "auto",
                           marginLeft: "auto", marginTop: "20px", width: "fit-content", cursor: "pointer"
                       }}>Close</p>
                </div>
            </div>)}
            <h2>MSFS: <span style={{color: `${bridgeConnected ? ("greenyellow") : ("red")}`}}>
                {bridgeConnected ? ("Connected") : ("Disconnected")}</span>
                {bridgeConnected && (<Tick color={"green"}/>)}</h2>
            {bridgeConnected && (
                <h2><span style={{color: `${loadedAircraft != "" ? ("greenyellow") : ("red")}`}}>
                {loadedAircraft != "" ? (loadedAircraft) : ("Please load a supported aircraft!")}</span>
                    {loadedAircraft != "" && (<Tick color={"green"}/>)}</h2>
            )}
            <div style={{
                width: "1300px",
                maxWidth: "95vw",
                marginLeft: "auto",
                marginRight: "auto"
            }}>
                {bridgeConnected ?
                    (
                        <div>

                            <p onClick={async () => {
                                await hideWindows()
                                props.showNotification("Pop-out windows hidden")
                            }} style={{
                                display: "inline-block",
                                cursor: "pointer",
                                marginTop: "5px",
                                padding: "5px",
                                backgroundColor: "darkorange",
                                width: "130px",
                                marginLeft: "auto",
                                marginRight: "10px",
                                borderRadius: "5px",
                            }}>Hide windows</p>

                            <p onClick={async () => {
                                await restoreWindows()
                                props.showNotification("Pop-out windows restored")
                            }} style={{
                                display: "inline-block",
                                cursor: "pointer",
                                marginTop: "5px",
                                padding: "5px",
                                backgroundColor: "royalblue",
                                width: "130px",
                                marginLeft: "auto",
                                marginRight: "10px",
                                borderRadius: "5px",
                            }}>Restore windows</p>
                            <p onClick={async () => {
                                await props.refreshFunction()
                                props.showNotification("Instruments scanned, list is updated")
                            }} style={{
                                display: "inline-block",
                                cursor: "pointer",
                                marginTop: "5px",
                                padding: "5px",
                                backgroundColor: "darkgreen",
                                width: "140px",
                                marginLeft: "auto",
                                marginRight: "10px",
                                borderRadius: "5px",
                            }}>Rescan instruments</p>
                        </div>
                    ) : (
                        <p onClick={reconnect} style={{
                            display: "inline-block",
                            cursor: "pointer",
                            marginTop: "5px",
                            padding: "5px",
                            backgroundColor: "green",
                            width: "80px",
                            marginLeft: "auto",
                            marginRight: "10px",
                            borderRadius: "5px",
                        }}>Refresh</p>
                    )
                }

            </div>
            <div style={{marginTop: "20px"}}>
                {instrumentSettings.map((item, i) => {
                    return (
                        <div key={i} style={{
                            display: "inline-block",
                            width: "120px",
                            height: "160px",
                            margin: "5px",
                            borderRadius: "7px",
                            border: "2px solid royalblue",
                            backgroundColor: "black"
                        }}>
                            <p style={{fontWeight: "bold", color: "lightskyblue"}}>{item.instrument}</p>
                            <p style={{height: "20px", color: "goldenrod"}}>{(item.selected && loadedAircraft != "")
                                ? ("SELECTED") : ("")}</p>
                            <img alt={item.instrument}
                                 style={{cursor: (item.selected && loadedAircraft != "") ? ("pointer") : ""}}
                                 width={"120px"} height={"120px"}
                                 onClick={() => {
                                     if (item.instrument != "UNKNOWN") {
                                         props.setInstrument(item)
                                     }
                                 }}
                                 src={"data:image/png;base64," + _arrayBufferToBase64(item.jpeg_bytes)}/>
                        </div>
                    )
                })
                }
            </div>

            <h1>Settings</h1>
            <div style={{
                width: "800px",
                maxWidth: "98vw",
                marginLeft: "auto",
                marginRight: "auto"
            }}>

                {/*</div>*/}
                <div style={{display: "flex", alignItems: "baseline"}}>
                    <p className={"settigns-left"}>Refresh rate:</p>
                    <div className={"settings-right"}>
                        <select style={{maxWidth: "80px"}} name="refresh" id="refresh">
                            <option value="slow">Slow</option>
                            <option value="normal">Normal</option>
                            <option value="fast">Fast</option>
                            <option value="veryfast">Very fast</option>
                        </select>
                    </div>
                </div>

                {/*</div>*/}


                <div style={{display: "flex", alignItems: "baseline"}}>
                    <p className={"settigns-left"}>Hide pop-outs automatically:
                        <span
                            style={{fontSize: "small", marginLeft: "5px", color: "green"}}>
                        (Recommended)</span></p>
                    <input onChange={checkerHandler} style={{width: "15px", height: "15px"}}
                           className={'settings-right'} type="checkbox" name="checkbox-checked" checked={autoChecked}/>
                </div>

                <div style={{display: "flex", alignItems: "baseline"}}>
                    <p className={"settigns-left"}>Automatically start server at startup:</p>

                    <input onChange={checkerAutoStartup} style={{width: "15px", height: "15px"}}
                           className={'settings-right'} type="checkbox" name="checkbox-checked"
                           checked={autoStartServer}/>
                </div>


                <p onClick={sendSave} style={{
                    cursor: "pointer",
                    marginTop: "35px",
                    padding: "7px",
                    backgroundColor: "darkgreen",
                    width: "50px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    borderRadius: "5px",
                }}>Save</p>


            </div>
            <p style={{marginTop: "50px", color: "lightgray", fontSize: "small"}}>Config
                version: {versions.version} (updated: {versions.date})</p>

            <p onClick={() => {
                loadAllAddons()
            }}
               style={{cursor: "pointer", textDecoration: "underline", fontSize: "small", color: "goldenrod"}}>
                List of all supported addons in config</p>

            <p onClick={() => {
                saveDebug()
                props.showNotification("Logging is now enabled, restart the ReachFMS app to create logs. Logging will only be enabled for the session starting with the next launch, it will be disabled after closing.")
            }} style={{
                marginTop: "30px",
                marginLeft: "auto",
                marginRight: "auto",
                cursor: "pointer",
                marginBottom: "10px",
                color: "royalblue"
            }}>Enable logging for the next session</p>
            <div style={{height: "200px"}}></div>
        </div>
    )
}