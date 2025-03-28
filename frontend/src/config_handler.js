import {getAddonConfig, getAircraftFile} from "./api_handler";

export async function getAircraftConfig(instruments) {

    let addons = await getAddonConfig();

    let aircraft = await getAircraftFile();
    aircraft = aircraft.toLowerCase()
    console.log(addons.aircraft_addons)
    for (let i = 0; i < addons.aircraft_addons.length; i++) {

        if (aircraft.includes(addons.aircraft_addons[i].title.toLowerCase())) {
            addons.aircraft_addons[i].version = addons.version;
            addons.aircraft_addons[i].updated = addons.updated;
            addons.aircraft_addons[i].app_version = addons.app_version;
            return addons.aircraft_addons[i];
        }
        // }
        // return null;
    }
    for (let i = 0; i < addons.aircraft_addons.length; i++) {
        for (let k = 0; k < addons.aircraft_addons[i].custom_popout.length; k++) {
            for (let j = 0; j < instruments.length; j++) {
                if (instruments[j].instrument.includes(addons.aircraft_addons[i].custom_popout[k])) {
                    addons.aircraft_addons[i].version = addons.version;
                    addons.aircraft_addons[i].updated = addons.updated;
                    addons.aircraft_addons[i].app_version = addons.app_version;
                    return addons.aircraft_addons[i];
                }
            }

        }
        // }
        // return null;
    }
    return {"version": addons.version, "updated": addons.updated, "app_version": addons.app_version}
}