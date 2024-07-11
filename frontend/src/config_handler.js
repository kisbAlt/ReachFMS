import {getAddonConfig, getAircraftFile} from "./api_handler";

export async function getAircraftConfig() {
    let addons = await getAddonConfig();
    let aircraft = await getAircraftFile();

    for (let i = 0; i < addons.aircraft_addons.length; i++) {
        if (aircraft.includes(addons.aircraft_addons[i].title)) {
            return addons.aircraft_addons[i];
        }
    }
    return null;
}