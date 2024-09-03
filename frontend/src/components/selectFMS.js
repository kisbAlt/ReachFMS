import {_arrayBufferToBase64} from "./settings";
import {setFMS} from "../api_handler";

export function SelectFMS(props) {
    return (
        <div>
            <h1 style={{textAlign: "center"}}>ReachFMS</h1>
            <p style={{textAlign: "center"}}>There is multiple pop out display found, please select the instrument/FMS display
                manually to continue.</p>
            <div style={{width: "fit-content", marginLeft: "auto", marginRight: "auto"}}>
                {props.instrumentObjects.map((item, i) => {
                    return (
                        <div key={i} style={{
                            display: "inline-block",
                            width: "fit-content",
                            height: "fit-content",
                            margin: "5px",
                            borderRadius: "7px",
                            border: "2px solid royalblue",
                            backgroundColor: "black"
                        }}>
                            <img alt={item.instrument} style={{cursor: "pointer"}} width={"200px"} height={"200px"}
                                 onClick={() => {
                                     if (!item.excluded) {
                                         props.setInstrument(item)
                                     }
                                 }}
                                 src={"data:image/png;base64," + _arrayBufferToBase64(item.jpeg_bytes)}/>
                            <p style={{textAlign: "center", fontSize: "large"}}>{item.instrument}</p>
                            <p onClick={() => {
                                setFMS(item.hwnd).then(() => {window.location.reload();})
                            }} style={{
                                backgroundColor: "green", padding: "10px",
                                marginLeft: "auto",
                                marginRight: "auto",
                                textAlign: "center",
                                fontWeight: "bold",
                                fontSize: "large",
                                cursor: "pointer"
                            }}>Set as FMS</p>
                        </div>
                    )
                })
                }

            </div>
        </div>
    )
}
