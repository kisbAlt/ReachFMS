const key_id_pair = [
    {"key": "a", "id": 65},{"key": "b", "id": 66},{"key": "c", "id": 67},{"key": "d", "id": 68},{"key": "e", "id": 69},
    {"key": "f", "id": 70},{"key": "g", "id": 71},{"key": "h", "id": 72},{"key": "i", "id": 73},{"key": "j", "id": 74},
    {"key": "k", "id": 75},{"key": "l", "id": 76},{"key": "m", "id": 77},{"key": "n", "id": 78},{"key": "o", "id": 79},
    {"key": "p", "id": 80},{"key": "q", "id": 81},{"key": "r", "id": 82},{"key": "s", "id": 83},{"key": "a", "id": 84},
    {"key": "u", "id": 85},{"key": "v", "id": 86},{"key": "w", "id":87},{"key": "x", "id": 88},{"key": "y", "id": 89},
    {"key": "z", "id": 90},{"key": "1", "id": 49},{"key": "2", "id": 50},{"key": "3", "id": 51},{"key": "4", "id": 52},
    {"key": "5", "id":53},{"key": "6", "id": 54},{"key": "7", "id": 55},{"key": "8", "id": 56},{"key": "9", "id": 57},
    {"key": "0", "id": 48},{"key": "ArrowUp", "id": 94},{"key": "ArrowLeft", "id": 95},{"key": "ArrowDown", "id": 30},
    {"key": "ArrowRight", "id": 31},{"key": "Backspace", "id": 8},
]

export function decodeKeyBoardAction(e) {
    for (let i = 0; i < key_id_pair.length; i++) {
        if (key_id_pair[i].key === e.key){
            var call_url;
            if(document.getElementById("foTogBtn")){
                call_url = "/mcdu_btn_press?btn="+key_id_pair[i].id+`&usefo=${document.getElementById("foTogBtn").checked}`
            }else{
                call_url = "/mcdu_btn_press?btn="+key_id_pair[i].id

            }
            fetch(call_url)
            return;
        }
    }
}