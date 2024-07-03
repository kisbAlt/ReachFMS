import {useEffect} from 'react';
import {McduSideToggleSwitch} from "./mcduSideToggleSwitch";
import {decodeKeyBoardAction} from "../decodeKeyBoardAction";
export function McduComponent(props) {
    useEffect(() => {
        fetch(process.env.REACT_APP_LOCALHOST_PREFIX + "/mcdu.svg")
            .then((response) => response.text())
            .then((data) => {
                document.getElementById("mcduSvg").innerHTML += data;
                setTimeout(() => {
                    scaleMcdu()
                }, "50");
            });
        window.addEventListener("resize", scaleMcdu);
        window.addEventListener('keydown', decodeKeyBoardAction)

        return () => {
            window.removeEventListener("resize", scaleMcdu)
            window.removeEventListener("keydown", decodeKeyBoardAction)
        };
        //eslint-disable-next-line
    }, []);

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
        var mcscg = document.getElementById("svg5476")
        if(mcscg == null) {return}
        var defAspect = 884 / 569;
        var diff = mcscg.getBoundingClientRect().height / mcscg.getBoundingClientRect().width;
        if (diff > defAspect) {
            var szorz = mcscg.getBoundingClientRect().height / diff;

            mcscg.style.height =
                `calc(100% - ${(884 / szorz) * 125}px)`;
        } else {
            //mcscg.style.height = `calc(100% - 80px)`;
        }

        if(props.fullScreenMode){
            mcscg.style.top = "0px";
        }else {
            mcscg.style.top = "75px";
        }

        document.getElementById("blackBack").style.height = document.getElementById("layer1").getBoundingClientRect().height + "px"
        document.getElementById("blackBack").style.width = document.getElementById("layer1").getBoundingClientRect().width + "px"



        // var maxW = document.getElementById("mcduParent").clientWidth;
        // var maxH = document.getElementById("mcduParent").clientHeight;
        // document.getElementById("mcduSvg").style.maxWidth = maxW+"px";
        // document.getElementById("mcduSvg").style.maxHeight = maxH+"px";
        //
        // var scalewidth = 1;
        // if (758 > maxW) {
        //     scalewidth = (maxW - 3) / 758;
        //     document.getElementById("mcduSvg").style.scale = `${scalewidth}`;
        //     document.getElementById("mcduSvg").style.left = `-${(1 - scalewidth) * (758 / 2)}px`;
        //     document.getElementById("mcduSvg").style.top = `-${(1 - scalewidth) * (1175 / 2)}px`;
        //     // document.getElementById("mcduSvg").style.transform =
        //     //     `translate(${(1-scale)*centerW}, ${(1-scale)*centerH}) scale(${scale})`;
        //
        //
        // }
        // if (1175 > maxH) {
        //     var scaleheight;
        //     if (props.fullScreenMode) {
        //         scaleheight = (maxH) / 1175
        //     } else {
        //         scaleheight = (maxH - 100) / 1175
        //     }
        //
        //     if (scaleheight < scalewidth) {
        //         document.getElementById("mcduSvg").style.scale = `${scaleheight}`;
        //         document.getElementById("mcduSvg").style.top = `-${(1 - scaleheight) * (1175 / 2)}px`;
        //         //document.getElementById("mcduSvg").style.left = `-${(1-scaleheight)*(758/2)}px`;
        //     }
        // }


        //width: "68.4%",
    }

    return (
        <div id={"mcduParent"} style={{height: "100%", width: "100%", position: "relative", overflow: "hidden"}}>
            <div style={{
                scale: "0.6", position: "absolute", zIndex: "10000", top: "-8px", left: "0", right: "0", marginLeft: "auto",
                marginRight: "auto",
            }}>
                <McduSideToggleSwitch/>
            </div>
            <div style={{}} id={"mcduSvg"}>
                <div id={"blackBack"} style={{backgroundColor: "black", position: "fixed", left: "0", right: "0",
                    marginLeft: "auto", marginRight: "auto"}}>
                    {props.useTif ? (
                        <div style={{
                            position: "absolute", width: "515px", left: "0", right: "0", marginLeft: "auto",
                            marginRight: "auto", top: "85px", zIndex: -1,
                        }}
                             id={"streamImage"}></div>) : (
                        <img style={{
                            position: "relative", width: "68.4%", left: "0", right: "0", marginLeft: "auto",
                            marginRight: "auto", top: "6.7%"
                        }}
                             id={"streamImage"} src={process.env.REACT_APP_LOCALHOST_PREFIX + `/get_image`}
                             alt={"streamImage"}/>
                    )}

                </div>
            </div>
        </div>
    );
}