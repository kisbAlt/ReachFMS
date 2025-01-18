import {useEffect, useRef, useState} from 'react';
import {McduSideToggleSwitch} from "./mcduSideToggleSwitch";
import {decodeKeyBoardAction} from "../decodeKeyBoardAction";
import {btnEvent, getAddonConfig, sendTouchEvent} from "../api_handler";
import {getAircraftConfig} from "../config_handler";
import {type} from "@testing-library/user-event/dist/type";

const TOUCH_ROTATE_SENSITIVITY = 2;

export function McduComponent(props) {
    const [webSocket, setWebSocket] = useState({});
    const [commandValue, setCommandValue] = useState("");
    const [pingStatus, setPingStatus] = useState("");
    const [imgWidth, setImgWidth] = useState(200);
    const [imgTop, setImgTop] = useState(20);
    const [imgLeft, setImgLeft] = useState(0);
    const aircraftConf = useRef({});
    const rtryTouched = useRef({"id": "", element: null});
    const lastTouchMoveLocation = useRef({
        "x": 0,
        "y": 0,
        "changedX": 0,
        "changedY": 0
    });

    useEffect(() => {
        start();
        window.addEventListener("resize", scaleMcdu);
        window.addEventListener('keydown', decodeKeyBoardAction)
        window.addEventListener('wheel', scrollEvent)
        window.addEventListener('touchstart', clickDownHandler)
        window.addEventListener('touchend', clickReleaseHandler)
        window.addEventListener('touchmove', touchMoveHandler)


        return () => {
            window.removeEventListener("resize", scaleMcdu)
            window.removeEventListener("keydown", decodeKeyBoardAction)
            window.removeEventListener("wheel", scrollEvent)
            window.removeEventListener("touchstart", clickDownHandler)
            window.removeEventListener("touchend", clickReleaseHandler)
            window.removeEventListener("touchmove", touchMoveHandler)

        };
        //eslint-disable-next-line
    }, []);

    async function start() {
        await loadSvg()
        connect_socket();
    }

    function clickDownHandler(e) {

        let parent = getRotaryElement(e.target)
        if (parent == null) {
            return
        }
        let rtry_id = parent.getAttribute("inkscape:label")

        rtryTouched.current = {"id": rtry_id.replace("RTRY", "BTN"), element: parent};
    }

    function clickReleaseHandler(e) {

        rtryTouched.current = {"id": "", element: null};
        lastTouchMoveLocation.current = {
            "x": 0,
            "y": 0,
            "changedX": 0,
            "changedY": 0
        }
    }

    function touchMoveHandler(e) {

        if (rtryTouched.current.id !== "") {
            e.preventDefault()

            if (lastTouchMoveLocation.current.x != 0) {
                let changedX = lastTouchMoveLocation.current.x - e.changedTouches[0].screenX;
                let changedY = lastTouchMoveLocation.current.y - e.changedTouches[0].screenY;
                lastTouchMoveLocation.current.changedX -= changedX;
                lastTouchMoveLocation.current.changedY -= changedY;
            }
            lastTouchMoveLocation.current.x = e.changedTouches[0].screenX;
            lastTouchMoveLocation.current.y = e.changedTouches[0].screenY;

            if (lastTouchMoveLocation.current.changedX > TOUCH_ROTATE_SENSITIVITY
            ) {
                lastTouchMoveLocation.current.changedX = 0;
                rotate_rotary(rtryTouched.current.element, 10);
                btnEvent(rtryTouched.current.id + "_INC")
            } else if (lastTouchMoveLocation.current.changedX < -1 * TOUCH_ROTATE_SENSITIVITY) {
                lastTouchMoveLocation.current.changedX = 0;
                rotate_rotary(rtryTouched.current.element, -10);
                btnEvent(rtryTouched.current.id + "_DEC")
            }
        }
    }

    function connect_socket() {
        // "ws://localhost:5273/ws
        let curr_location = window.location.origin;

        let ws_url;
        if (process.env.REACT_APP_LOCALHOST_PREFIX != "") {
            ws_url = process.env.REACT_APP_LOCALHOST_PREFIX.split("http").join("ws") +
                `/ws`;
        } else {
            // ws_url = curr_location.split("http").join("ws") +
            //     `/ws`;
            ws_url = `${curr_location.replace("http", "ws")}/ws`
        }

        let lwebSocket = new WebSocket(ws_url);
        let mcdu_screen = document.getElementById("streamImage");
        var urlCreator = window.URL || window.webkitURL;
        let lastImage = null;
        lwebSocket.onmessage = function (e) {

            if (lastImage != null) {
                URL.revokeObjectURL(lastImage);
            }
            if (typeof e.data === 'string') {

            } else {
                let imgBlob = new Blob([e.data], {type: 'image/png'});
                var imageUrl = urlCreator.createObjectURL(imgBlob);
                mcdu_screen.src = imageUrl;
                lastImage = imageUrl
            }
        }
        var t;
        lwebSocket.onopen = (event) => {
            setWebSocket(lwebSocket)
            lwebSocket.send("IMAGESUBSCRIBE")
        };
        return () => {
            //setConsoleContent(blank_console);
            lwebSocket.close()
            clearInterval(t)
        };
    }

    async function loadSvg() {
        console.log("loadsvg")
        let aircraft_config = await getAircraftConfig(props.instrumentObjects);
        console.log(aircraft_config)
        aircraftConf.current = aircraft_config
        if (aircraft_config != null) {
            setImgWidth(aircraft_config.display_width);
            setImgTop(aircraft_config.display_top);
            setImgLeft(aircraft_config.display_left);
            // DEBUG
            //aircraft_config.svg_image = "GNS530.svg"
            // /DEBUG

            fetch(process.env.REACT_APP_LOCALHOST_PREFIX + `/static/${aircraft_config.svg_image}`)
                .then((response) => response.text())
                .then((data) => {
                    let mcduPlacement = document.getElementById('mcduPlacement')
                    mcduPlacement.innerHTML = data;
                    setTimeout(() => {
                        let mwidth = mcduPlacement.getBoundingClientRect().width;
                        let mheight = mcduPlacement.getBoundingClientRect().height;
                        document.getElementById("blackBack").style.width = `${mwidth}px`;
                        document.getElementById("blackBack").style.height = `${mheight}px`;

                        scaleMcdu();
                        //loadSvg()
                    }, "50");
                });
        }
    }

    useEffect(() => {


    }, [props.fullScreenMode]);


    function scaleMcdu() {
        var current_scale = Number(document.getElementById("mcduSvg").style.scale);
        if (current_scale == 0) {
            current_scale = 1;
        }
        var mcscg = document.getElementById("mcduPlacement")
        var parent = document.getElementById("mcduParent")
        if (mcscg == null) {
            return
        }
        let mcdu_h = mcscg.getBoundingClientRect().height / current_scale;
        let mcdu_w = mcscg.getBoundingClientRect().width / current_scale;
        let mcdu_aspect = mcdu_w / mcdu_h;

        let window_w = parent.getBoundingClientRect().width;
        let window_h = parent.getBoundingClientRect().height - 100;
        let window_aspect = window_w / window_h;
        let scale;
        let elementToScale = document.getElementById("mcduSvg")
        if (mcdu_aspect > window_aspect) {
            console.log(1)
            scale = Math.round((window_w / mcdu_w) * 1000) / 1000;
            if (scale < 1) {
                elementToScale.style.width = "fit-content";
            }
        } else {
            console.log(2)
            scale = Math.round((window_h / mcdu_h) * 1000) / 1000;
            elementToScale.style.width = "100vw";
        }
        elementToScale.style.scale = `${scale}`

    }

    function imgClicked(e) {

        if (!aircraftConf.current.touch_enabled) {
            return
        }

        var rect = document.getElementById("streamImage").getBoundingClientRect();
        var x = Math.round(e.clientX - rect.left); //x position within the element.
        var y = Math.round(e.clientY - rect.top);  //y position within the element.
        if (x >= 0 && y >= 0 && x <= rect.width && y <= rect.height) {

            var current_scale = Number(document.getElementById("mcduSvg").style.scale);
            sendTouchEvent(Math.round(x / current_scale),
                Math.round(y / current_scale))
        }
    }

    function getRotaryElement(elem) {
        while (
            elem != null && (
                elem.getAttribute("inkscape:label") == null ||
                !elem.getAttribute("inkscape:label").includes("RTRY:")
            )) {
            elem = elem.parentElement;
        }
        return elem
    }


    function scrollEvent(e) {
        let parent = getRotaryElement(e.target)
        if (parent == null) {
            return
        }
        let rtry_id = parent.getAttribute("inkscape:label")

        if (rtry_id.includes("RTRY:")) {
            rtry_id = rtry_id.replace("RTRY", "BTN")
            if (e.deltaY > 0) {
                rotate_rotary(parent, -10);
                rtry_id += "_DEC"
            } else {
                rotate_rotary(parent, 10);
                rtry_id += "_INC"
            }
            //parent.style.transform = `rotate(${rotation}deg, ${cx}, ${cy})`
            //parent.setAttribute('transform', `rotate(${rotation}, ${cx}, ${cy})`);
            btnEvent(rtry_id)
        }
    }

    function rotate_rotary(parent, deg) {
        var style = window.getComputedStyle(parent);
        var matrix = new DOMMatrix(style.transform);

        const currentTransform = parent.transform.baseVal.consolidate();
        //let matrix = currentTransform ? currentTransform.matrix : new DOMMatrix();
        // Initialize current angle
        let currentAngle = 0;

        // If a transform exists, extract the current rotation angle
        if (currentTransform) {
            const matrix = currentTransform.matrix;

            // Extract the rotation angle from the matrix
            currentAngle = Math.atan2(matrix.b, matrix.a) * (180 / Math.PI);
        }


        let rotation = currentAngle + deg
        // Get the bounding box of the group element
        const bbox = parent.getBBox();

        // Calculate the center of the bounding box
        const cx = bbox.x + bbox.width / 2;
        const cy = bbox.y + bbox.height / 2;

        // Apply the transform


        matrix = matrix.translate(cx, cy);

        // Apply the new rotation
        matrix = matrix.rotate(rotation);

        // Translate back to the original position
        matrix = matrix.translate(-cx, -cy);


        // Apply the matrix to the transform attribute of the <g> element
        parent.setAttribute('transform', matrix.toString());
    }

    function getCurrentRotation(el) {
        var st = window.getComputedStyle(el, null);
        var tm = st.getPropertyValue("-webkit-transform") ||
            st.getPropertyValue("-moz-transform") ||
            st.getPropertyValue("-ms-transform") ||
            st.getPropertyValue("-o-transform") ||
            st.getPropertyValue("transform") ||
            "none";
        if (tm != "none") {
            var values = tm.split('(')[1].split(')')[0].split(',');
            /*
            a = values[0];
            b = values[1];
            angle = Math.round(Math.atan2(b,a) * (180/Math.PI));
            */
            //return Math.round(Math.atan2(values[1],values[0]) * (180/Math.PI)); //this would return negative values the OP doesn't wants so it got commented and the next lines of code added
            var angle = Math.round(Math.atan2(values[1], values[0]) * (180 / Math.PI));
            return (angle < 0 ? angle + 360 : angle); //adding 360 degrees here when angle < 0 is equivalent to adding (2 * Math.PI) radians before
        }
        return 0;
    }


    return (
        <div id={"mcduParent"} style={{height: "100%", width: "100%", position: "relative", overflow: "hidden"}}>

            <div style={{width: "fit-content", height: "fit-content"}} id={"mcduSvg"}>

                    <div style={{
                        position: "absolute",
                        zIndex: "5",
                        left: "0px",
                        right: "0px",
                        marginLeft: "auto",
                        marginRight: "auto",
                        width: "fit-content",
                    }} onClick={imgClicked}
                         id={"mcduPlacement"}>

                    </div>
                    <div id={"blackBack"} style={{
                        position: "fixed", left: "0", right: "0",
                        marginLeft: "auto", marginRight: "auto", overflow: "hidden"
                    }}>

                        <img onClick={imgClicked} style={{
                            position: "absolute", width: `${imgWidth}px`, left: "0", right: "0", marginLeft: "auto",
                            marginRight: "auto", marginTop: `${imgTop}px`, paddingLeft: `${imgLeft}px`,
                            border: "10px solid black"
                        }}
                             id={"streamImage"}
                             alt={"Image Stream loading..."}/>
                    </div>

            </div>
        </div>
    );
}