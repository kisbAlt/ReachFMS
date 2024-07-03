import {useEffect, useState} from 'react';
import {forceInstruments, getInstruments, getStatus, sendCalibration} from "../api_handler";
import {_arrayBufferToBase64} from "./settings";
import {LoadingComponent} from "./loadingComponent";

export function CalibrationComponent(props) {
    const [progress, setProgress] = useState("firstStep")
    const [popouts, setPopouts] = useState([])
    const [calibratedInstr, setCalibratedInstr] = useState([])
    const [instrDone, setInstrDone] = useState(0)

    function startCalibration() {
        setProgress("promptOpen")
    }

    async function getWindows() {
        var instrList = await getInstruments();
        var templ = [];
        for (let i = 0; i < instrList.length; i++) {
            templ.push({"hwnd": instrList[i].hwnd, "calibrated": "None"})
        }
        setCalibratedInstr(templ)

        setPopouts(instrList)
    }

    function selectHandler(e) {
        var localCal = [...calibratedInstr]
        var sum = 0;
        for (let i = 0; i < localCal.length; i++) {
            if (localCal[i].hwnd === parseInt(e.target.id)) {
                localCal[i].calibrated = e.target.value
            }
            if (localCal[i].calibrated !== "None") {
                sum++
            }
        }
        if (sum === localCal.length && popouts.length === 5) {
            document.getElementById("finishButton").style.backgroundColor = "green";
        } else {
            document.getElementById("finishButton").style.backgroundColor = "gray";
        }

        setInstrDone(sum);

        setCalibratedInstr(localCal)
    }

    async function finish() {
        // if (instrDone === 0 || instrDone !== popouts.length || popouts.length !== 5) {
        //     return
        // }
        setProgress("forceUpdate")
        var calibstring = "";
        for (let i = 0; i < calibratedInstr.length; i++) {
            calibstring += `${calibratedInstr[i].hwnd}:${calibratedInstr[i].calibrated};`
        }
        calibstring = calibstring.slice(0, calibstring.length - 1)
        await sendCalibration(calibstring)
        await forceInstruments()
        window.location = "/";
    }

    return (
        <div style={{
            backgroundColor: "rgb(4, 16, 19)",
            height: "100vh",
            width: "100vw",
            overflow: "scroll",
            marginTop: "0"
        }}>
            <div style={{textAlign: "center",}}>
                <img alt={"mcdu"} src={process.env.REACT_APP_LOCALHOST_PREFIX + "/icon.png"}
                     style={{height: "85px", marginTop: "20px"}}/>
                <p style={{fontSize: "large"}}>Remote MCDU for the fenix a320</p>
                <p style={{fontSize: "small"}}>By <a rel="noreferrer"
                                                     style={{
                                                         color: "cornflowerblue",
                                                         fontWeight: "600"
                                                     }} target="_blank"
                                                     href={"http://airportfinder.us.to"}>kisbAlt</a>
                </p>
            </div>
            <h1 style={{textAlign: "center", marginTop: "30px"}}>Calibration</h1>
            {progress == "firstStep" && (<div>
                <p style={{
                    textAlign: "center",
                    marginTop: "20px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px"
                }}>
                    It looks like you have not calibrated your instruments yet. You should definitely do this process if
                    you
                    started the app for the first time.<br/><br/> You only need to do this process once, so the
                    instruments
                    will be
                    recognised automatically after the calibration.
                    If you are experiencing problems with the instrument detection (identifying all displays as MCDU, or
                    as a
                    wrong instrument) you should also redo this calibration.</p>

                <p style={{
                    textAlign: "center",
                    marginTop: "15px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px",
                    fontStyle: "italic"
                }}>You can skip calibration if you only want to use the MCDU. If you skip the calibration, instruments
                    won't
                    be recognised correctly. You can start the calibration anytime from the settings page.</p>

                <p onClick={startCalibration} style={{
                    cursor: "pointer",
                    marginTop: "35px",
                    padding: "15px",
                    backgroundColor: "darkgreen",
                    width: "fit-content",
                    marginLeft: "auto",
                    marginRight: "auto",
                    borderRadius: "5px",
                    fontSize: "large",
                    fontWeight: "bolder"
                }}>Start calibration</p>

                <p onClick={() => {
                    window.location = "/?skip_calibration=true"
                }}
                   style={{
                       color: "cornflowerblue",
                       fontWeight: "600",
                       textAlign: "center",
                       cursor: "pointer",
                       textDecoration: "underline",
                       marginTop: "15px"
                   }} target="_blank"
                >Skip calibration</p>
            </div>)}

            {progress == "promptOpen" && (<div>
                <p style={{
                    textAlign: "center",
                    marginTop: "0px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px"
                }}> You should now load up the simulator, and start a flight with the fenix A320. Before continuing make
                    sure that you have the aircraft started with all the displays and systems aligned. You should then
                    pop out the
                    following displays with clicking on the displays while holding left ALT:</p>
                <p style={{
                    textAlign: "center",
                    marginTop: "-10px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px",
                    fontWeight: "bolder",
                    color: "cornflowerblue",
                    fontSize: "large"
                }}>CAPTAIN PFD, CAPTAIN ND, UPPER ECAM, LOWER ECAM, CPT MCDU</p>

                <p onClick={() => {
                    setProgress("dispSelect");
                    getWindows()
                }} style={{
                    cursor: "pointer",
                    marginTop: "35px",
                    padding: "15px",
                    backgroundColor: "darkgreen",
                    width: "fit-content",
                    marginLeft: "auto",
                    marginRight: "auto",
                    borderRadius: "5px",
                    fontSize: "large",
                    fontWeight: "bolder"
                }}>Continue</p>
            </div>)}

            {progress == "dispSelect" && (<div>
                <p style={{
                    textAlign: "center",
                    marginTop: "0px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px"
                }}>You should have exactly 5 pop-outs. Select manually which window is which instrument</p>
                <p style={{
                    textAlign: "center",
                    marginTop: "0px",
                    maxWidth: "800px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    padding: "20px",
                    fontWeight: "bolder",
                    color: "cornflowerblue",
                    fontSize: "large"
                }}>CAPTAIN PFD, CAPTAIN ND, UPPER ECAM, LOWER ECAM, CPT MCDU</p>
                <div style={{padding: "10px", marginLeft: "auto", marginRight: "auto", width: "fit-content"}}>
                    {popouts.map((popout, i) => {
                            return (
                                <div key={i} style={{
                                    display: "inline-block",
                                    marginLeft: "10px",
                                    marginTop: "10px",
                                    padding: "10px",
                                    backgroundColor: "black",
                                    border: "1px solid gray",
                                    borderRadius: "8px",
                                    textAlign: "center"
                                }}>

                                    <div>
                                        <img alt={popout.instrument} style={{cursor: "pointer"}} width={"120px"}
                                             height={"120px"}
                                             onClick={() => {
                                                 if (!popout.excluded) {
                                                     props.setInstrument(popout)
                                                 }
                                             }}
                                             src={"data:image/png;base64," + _arrayBufferToBase64(popout.jpeg_bytes)}/>
                                    </div>
                                    <select style={selectStyle} name="instrSelect" id={popout.hwnd}
                                            onChange={selectHandler}>
                                        <option value={"None"}>Select</option>
                                        <option value="PFD">PFD</option>
                                        <option value="ND">ND</option>
                                        <option value="U_ECAM">U_ECAM</option>
                                        <option value="L_ECAM">L_ECAM</option>
                                        <option value="MCDU">MCDU</option>
                                    </select>
                                </div>
                            )
                        }
                    )}
                </div>
                {(instrDone === 0 || instrDone !== popouts.length) && (<div>
                    <p style={{textAlign: "center", color: "red", fontWeight: "bolder"}}>
                        It looks like you have not selected an instrument for all of the pop-outs.</p>
                </div>)}

                {(5 !== popouts.length) && (<div>
                    <p style={{textAlign: "center", color: "red", fontWeight: "bolder"}}>
                        You should only pop out the listed displays! It seems like that you have less/more displays
                        popped out.</p>
                </div>)}

                <p onClick={() => {
                    getWindows()
                }} style={{
                    cursor: "pointer",
                    marginTop: "15px",
                    padding: "15px",
                    backgroundColor: "blueviolet",
                    width: "fit-content",
                    marginLeft: "auto",
                    marginRight: "auto",
                    borderRadius: "5px",
                    fontSize: "large",
                    fontWeight: "bolder"
                }}>Refresh</p>

                <p style={{textAlign: "center"}}>

                    <button id={"finishButton"} onClick={() => {
                        finish()
                    }} style={{
                        marginTop: "35px",
                        padding: "15px",
                        backgroundColor: "gray",
                        width: "fit-content",
                        marginLeft: "auto",
                        marginRight: "auto",
                        borderRadius: "5px",
                        fontSize: "large",
                        fontWeight: "bolder",
                        textAlign: "center"
                    }}>Finish calibration
                    </button>
                </p>
            </div>)}

            {progress === "forceUpdate" && (<div>
                <p style={{
                    textAlign: "center",
                    marginTop: "10vh",
                    fontSize: "xxx-large",
                    fontWeight: "lighter",
                    color: "cornflowerblue"
                }}>Matching templates, please wait...</p>
            </div>)}
        </div>
    )
}

const selectStyle = {
    backgroundColor: "#222422",
    border: "none",
    padding: "10px",
    borderRadius: "5px",
    color: "white",
    marginRight: "auto",
    marginLeft: "auto"
}