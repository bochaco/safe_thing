// Copyright 2019 Gabriel Viganotti <@bochaco>.
//
// This file is part of the SAFEthing Framework.
//
// The SAFEthing Framework is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The SAFEthing Framework is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with the SAFEthing Framework. If not, see <https://www.gnu.org/licenses/>.

use safe_thing::{AccessType, ActionDef, SAFEthing, ThingAttr, Topic};
use std::thread;
use std::time::Duration;

const RANDOM_PSI_FACTOR: f32 = 8.0;
const MAX_PSI_ALLOWED: u32 = 100;

static mut CURRENT_MOISTURE_LEVEL_FACTOR: f32 = -1.0;

pub fn main() {
    // Let's create a SAFEthing id for the gardening device, this could be the device serial number
    let id = "gardening-device-serial-number-01010101";

    // Let's have the SAFEthing framework to authorise the device with any
    // available SAFE Authentiactor rather than providing the authorisation credentials
    let auth_uri = "";

    let attributes = [
        ThingAttr::new("name", "SAFEthing Gardening Device", false),
        ThingAttr::new("firmware", "v0.1.0", false),
        ThingAttr::new("moisture-level", "", true),
        ThingAttr::new("pressure-psi", "50", true),
        ThingAttr::new("valve_state", "closed", true),
    ];

    let topics = [
        Topic::new("VeryDryAlarm", AccessType::All),
        Topic::new("VeryWetAlarm", AccessType::All),
    ];

    let actions = [
        ActionDef::new("OpenValve", AccessType::All, &["psi"]),
        ActionDef::new("CloseValve", AccessType::All, &[]),
    ];

    // Let's create an instance of SAFEthing for this device.
    // We already provide the two callback functions to be called
    // for subcriptions notifications and action requests respectively.
    let mut safe_thing =
        SAFEthing::new(&id, auth_uri, &subscriptions_notif, &action_request_notif).unwrap();

    // Register the SAFEthing on the network, this won't make it active yet
    // but it will just store the device's data onto the network as a SAFEthing entity
    safe_thing
        .register(&attributes, &topics, &actions)
        .map(|()| println!("Gardening device registered on the network"))
        .unwrap();

    // Let's now make it active and ready for receiving action requests from other SAFEthings
    safe_thing.publish().expect("Failed to publish SAFEthing");

    // Part of the main logic of this device is to keep reading the moisture level of the soil,
    // and update its published dynamic `moisture_level` attribute.
    // Any other SAFEthing device subscribed to the dynamic `moisture_level` attribute will
    // automatically receive notifications when its value changes but taking into account the
    // filters it provided in the subscription. Note that such notifications are sent by
    // the other device's SAFEthing framework instance rather than this one.
    // Let's set some initial value simulating to be the current moisture level read from a sensor.
    let mut current_moisture_level: f32 = 6.5;
    let mut notif_sent = false;

    // We now go into an infinite loop which contains the main logic of this device
    loop {
        // In a real situation, the following statement would be replaced by the logic
        // to actually read the value from a soil moisture sensor.
        // In order to simulate such an environment, we will consider the moisture level
        // decrease 0.1 if the water valve is closed, trying to mimic a situation where the soil
        // dries out as the time passes by, and the moisture level would be increased while the
        // water valve is open (note we are using the requested water PSI in when setting the factor)
        unsafe {
            current_moisture_level += 0.1 * CURRENT_MOISTURE_LEVEL_FACTOR;
        }

        // And let's keep the published dynamic attribute up to date for any
        // other SAFEthing to be notified if it's interested in knowing about the new value
        println!(
            "Updating value of 'moisture_level' dynamic attribute to '{}'",
            current_moisture_level
        );
        safe_thing
            .set_attr_value("moisture-level", &current_moisture_level.to_string())
            .unwrap();

        // This device also supports two topics (VeryWetAlarm and VeryDryAlarm) that
        // other SAFEthings can register to in order to receive notifications when it detects
        // that the soil moisture level goes beyond thresholds. Thus we should check the current
        // moisture level and send the notification for the corresponding topic.
        // We also keep a local flag so we don't send duplicate notifications.
        if current_moisture_level > 8.0 {
            if !notif_sent {
                let _ = safe_thing.notify("VeryWetAlarm", "");
            }
            notif_sent = true;
        } else if current_moisture_level < 3.0 {
            if !notif_sent {
                let _ = safe_thing.notify("VeryDryAlarm", "");
            }
            notif_sent = true;
        } else {
            notif_sent = false;
        }

        // Let's wait for some time before going into a new cycle of reading the
        // current soil moisture level and publishing the new value
        thread::sleep(Duration::from_millis(3000));
    }
}

fn action_request_notif(
    safe_thing: &SAFEthing,
    request_id: u128,
    thing_id: &str,
    action: &str,
    args: &[&str],
) {
    println!(
        "New action request received, id: '{}', from thing_id: '{}', action: '{}', args: {:?}",
        request_id, thing_id, action, args
    );

    // Let's act according to the notification we received...
    match action {
        "OpenValve" => {
            let psi = args[0].parse::<u32>().unwrap_or(10); // if we fail to parse it assume 10 psi by default
            let requested_water_psi: u32 = if psi > MAX_PSI_ALLOWED {
                MAX_PSI_ALLOWED
            } else {
                psi
            }; // 100 psi is the max we allow

            // We just print a message here , but in a real situation this is where we should
            // open the watering device valve.
            println!(
                "Opening the water valve as requested by SAFEthing {}, with a presure of {} psi",
                thing_id, requested_water_psi
            );
            safe_thing.set_attr_value("valve_state", "open").unwrap();

            // To simulate that the soil moisture level increases when the water valve is open
            // we set the factor to a positive number and apply some factoring to also consider the
            // requested psi. Remember, in a real situation, the following statement wouldn't be needed.
            unsafe {
                CURRENT_MOISTURE_LEVEL_FACTOR = requested_water_psi as f32 / RANDOM_PSI_FACTOR;
            }
        }
        "CloseValve" => {
            println!(
                "Closing the water valve as requested by SAFEthing {}",
                thing_id
            );
            safe_thing.set_attr_value("valve_state", "closed").unwrap();

            // To simulate that the soil moisture level decreases when the water valve is closed
            // we set the factor to a negative number. Remember, in a real situation,
            // the following statement wouldn't be needed.
            unsafe {
                CURRENT_MOISTURE_LEVEL_FACTOR = -1.0; // negative to decrease the level
            }
        }
        &_ => eprintln!("Unknown action request received: {}", action),
    }
}

// We haven't subsribed to any SAFEthing's topic, thus this function shoulnd't be invoked
fn subscriptions_notif(
    _safe_thing: &SAFEthing,
    _thing_id: &str,
    _topic: &str,
    _data: &str,
    _timestamp: u128,
) {
}
