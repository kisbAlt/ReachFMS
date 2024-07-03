import { useEffect } from 'react';
export function LoverEcam(props) {

    useEffect(() => {
        fetch(process.env.REACT_APP_LOCALHOST_PREFIX + "/ecam.svg")
            .then((response) => response.text())
            .then((data) => {
                document.getElementById("ecamSvg").innerHTML += data
                scaleMcdu()
            });
        window.addEventListener("resize", scaleMcdu);
        return () => window.removeEventListener("resize", scaleMcdu);
        //eslint-disable-next-line
    }, []);

    useEffect(() => {
        //scaleMcdu()
        //eslint-disable-next-line
    }, [props.fullScreenMode]);

    function scaleMcdu() {
        var ecamsv = document.getElementById("ecamSvgImage")
        var streamimg = document.getElementById("streamImage");
        streamimg.style.height = ecamsv.getBoundingClientRect().width+"px";
        streamimg.style.width = ecamsv.getBoundingClientRect().width+"px";

        if (ecamsv.getBoundingClientRect().height+streamimg.getBoundingClientRect().height > window.screen.height-80){
            var newH = window.screen.height-80-ecamsv.getBoundingClientRect().height
            streamimg.style.height = newH+"px";
            streamimg.style.width = newH+"px";

            document.getElementById("ecamSvgMain").style.height = `calc(100% - 140px)`
            document.getElementById("ecamSvgMain").style.top = "auto";
            document.getElementById("ecamSvgMain").style.bottom = "0px";

        } else {
        document.getElementById("ecamSvgMain").style.top = streamimg.getBoundingClientRect().width+75+"px";

        }

        // var maxW =  document.getElementById("svgParent").clientWidth;
        // var maxH =  document.getElementById("svgParent").clientHeight;
        //
        // var scalewidth = 1;
        // if(758 > maxW){
        //     scalewidth = (maxW-3)/758;
        //     document.getElementById("ecamSvg").style.scale = `${scalewidth}`;
        //     document.getElementById("ecamSvg").style.left = `-${(1-scalewidth)*(758/2)}px`;
        //     document.getElementById("ecamSvg").style.top = `-${(1-scalewidth)*(1100/2)}px`;
        //     // document.getElementById("mcduSvg").style.transform =
        //     //     `translate(${(1-scale)*centerW}, ${(1-scale)*centerH}) scale(${scale})`;
        //
        //
        // }
        // if(1100 > maxH){
        //     var scaleheight;
        //     if (props.fullScreenMode) {
        //         scaleheight = (maxH) / 1175
        //     } else {
        //         scaleheight = (maxH - 100) / 1175
        //     }
        //     if(scaleheight < scalewidth)
        //     {
        //         document.getElementById("ecamSvg").style.scale = `${scaleheight}`;
        //         document.getElementById("ecamSvg").style.top = `-${(1-scaleheight)*(1100/2)}px`;
        //         //document.getElementById("mcduSvg").style.left = `-${(1-scaleheight)*(758/2)}px`;
        //     }
        // }
    }

    return (
        <div id={"svgParent"} style={{height: "100%", width: "100%", position: "relative"}}>
            <div style={{width: "fit-content", height: "fit-content", margin: "auto", top: 0, left: 0, right: 0, zIndex: 1,
                position: "absolute", overflow: "hidden", backgroundColor: "black"}} id={"ecamSvg"}>
                {props.useTif ? (
                    <div style={{ width: "730px", left: "0", right: "0", marginLeft: "auto", position: "fixed",
                        marginRight: "auto", top: "75px", zIndex: -1,}}
                         id={"streamImage"}/>
                ) : (
                    <img style={{ width: "730px", left: "0", right: "0", marginLeft: "auto", position: "fixed",
                        marginRight: "auto", top: "75px", zIndex: -1,}}
                         id={"streamImage"} src={process.env.REACT_APP_LOCALHOST_PREFIX + `/get_image`} alt={"streamImage"}/>

                )}
                {/*<TimedImage src={process.env.REACT_APP_LOCALHOST_PREFIX + "/get_image"} interval={300} />*/}
            </div>
        </div>
    );
}