import {useEffect, useState} from 'react';
import {McduSideToggleSwitch} from "./mcduSideToggleSwitch";
import {decodeKeyBoardAction} from "../decodeKeyBoardAction";
import {getAddonConfig} from "../api_handler";
import {getAircraftConfig} from "../config_handler";
import {type} from "@testing-library/user-event/dist/type";

export function McduComponent(props) {
    const [webSocket, setWebSocket] = useState({});
    const [commandValue, setCommandValue] = useState("");
    const [pingStatus, setPingStatus] = useState("");
    const [imgWidth, setImgWidth] = useState(200);
    const [imgTop, setImgTop] = useState(20);

    useEffect(() => {
        start();
        window.addEventListener("resize", scaleMcdu);
        window.addEventListener('keydown', decodeKeyBoardAction)

        return () => {
            window.removeEventListener("resize", scaleMcdu)
            window.removeEventListener("keydown", decodeKeyBoardAction)
        };
        //eslint-disable-next-line
    }, []);

    async function start() {
        await loadSvg()
        connect_socket();
    }

    function connect_socket() {
        // "ws://localhost:5273/ws/
        let ws_url = process.env.REACT_APP_LOCALHOST_PREFIX.replaceAll("http", "ws") +
            `/ws/?frontend`;
        console.log(ws_url)
        let lwebSocket = new WebSocket(ws_url);
        let mcdu_screen = document.getElementById("streamImage");
        var urlCreator = window.URL || window.webkitURL;
        lwebSocket.onmessage = function (e) {

            if (typeof e.data === 'string') {

            } else {
                var imageUrl = urlCreator.createObjectURL(e.data);
                mcdu_screen.src = imageUrl;
            }
        }
        var t;
        lwebSocket.onopen = (event) => {
            setWebSocket(lwebSocket)
            console.log("sending sub")
            lwebSocket.send("IMAGESUBSCRIBE")
        };
        return () => {
            //setConsoleContent(blank_console);
            lwebSocket.close()
            clearInterval(t)
        };
    }

    async function loadSvg() {
        let aircraft_config = await getAircraftConfig();
        console.log(aircraft_config)
        if (aircraft_config != null) {
            console.log(aircraft_config.display_width)
            console.log(aircraft_config.display_top)
            setImgWidth(aircraft_config.display_width);
            setImgTop(aircraft_config.display_top);
            fetch(process.env.REACT_APP_LOCALHOST_PREFIX + `/static/${aircraft_config.svg_image}`)
                .then((response) => response.text())
                .then((data) => {
                    document.getElementById('mcduPlacement').innerHTML = data;
                    setTimeout(() => {
                        scaleMcdu()
                    }, "50");
                });
        }
    }

    useEffect(() => {
        scaleMcdu()
        if (localStorage.getItem("usefo") !== null) {
            if (localStorage.getItem("usefo") === "true") {
                //document.getElementById("foTogBtn").checked = true;
                setTimeout(() => {
                    document.getElementById("foTogBtn").click();
                }, "100");
            }
        }

        //eslint-disable-next-line
    }, [props.fullScreenMode]);


    function scaleMcdu() {
        var mcscg = document.getElementById("mcduPlacement")
        var parent = document.getElementById("mcduParent")
        if (mcscg == null) {
            return
        }
        let mcdu_h = mcscg.getBoundingClientRect().height;
        let mcdu_w = mcscg.getBoundingClientRect().width;
        let mcdu_aspect = mcdu_w / mcdu_h;

        let window_w = parent.getBoundingClientRect().width;
        let window_h = parent.getBoundingClientRect().height - 100;
        let window_aspect = window_w / window_h;
        let scale;
        if (mcdu_aspect > window_aspect) {
            scale = window_w / mcdu_w;
        } else {
            scale = window_h / mcdu_h;
        }
        document.getElementById("mcduSvg").style.scale = scale;

        // var defAspect = 884 / 569;
        // var diff = mcscg.getBoundingClientRect().height / mcscg.getBoundingClientRect().width;
        // if (diff > defAspect) {
        //     var szorz = mcscg.getBoundingClientRect().height / diff;
        //
        //     mcscg.style.height =
        //         `calc(100% - ${(884 / szorz) * 125}px)`;
        // } else {
        //     //mcscg.style.height = `calc(100% - 80px)`;
        // }
        //
        // if(props.fullScreenMode){
        //     mcscg.style.top = "0px";
        // }else {
        //     mcscg.style.top = "75px";
        // }
        //
        // document.getElementById("blackBack").style.height = document.getElementById("layer1").getBoundingClientRect().height + "px"
        // document.getElementById("blackBack").style.width = document.getElementById("layer1").getBoundingClientRect().width + "px"

    }

    return (
        <div id={"mcduParent"} style={{height: "100%", width: "100%", position: "relative", overflow: "hidden"}}>
            {/*<div style={{*/}
            {/*    scale: "0.6",*/}
            {/*    position: "absolute",*/}
            {/*    zIndex: "10000",*/}
            {/*    top: "-8px",*/}
            {/*    left: "0",*/}
            {/*    right: "0",*/}
            {/*    marginLeft: "auto",*/}
            {/*    marginRight: "auto",*/}
            {/*}}>*/}
            {/*    <McduSideToggleSwitch/>*/}
            {/*</div>*/}
            <div style={{}} id={"mcduSvg"}>
                <div style={{
                    position: "absolute",
                    zIndex: "5",
                    left: "0px",
                    right: "0px",
                    marginLeft: "auto",
                    marginRight: "auto",
                    width: "fit-content",
                }}
                     id={"mcduPlacement"}>

                </div>
                <div id={"blackBack"} style={{
                    position: "fixed", left: "0", right: "0",
                    marginLeft: "auto", marginRight: "auto"
                }}>

                    <img style={{
                        position: "absolute", width: `${imgWidth}px`, left: "0", right: "0", marginLeft: "auto",
                        marginRight: "auto", marginTop: `${imgTop}px`
                    }}
                         id={"streamImage"} src={process.env.REACT_APP_LOCALHOST_PREFIX + `/get_image`}
                         alt={"streamImage"}/>
                </div>
            </div>
        </div>
    );
}