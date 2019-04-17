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

use safe_thing::{FilterOperator, SAFEthing, ThingAttr};
use std::thread;
use std::time::Duration;

pub fn main() {
    // Let's create a SAFEthing id for the controller device of our gardening system
    let id = "gardening-controller-device-serial-number-01010101";

    // Let's have the SAFEthing framework to authorise the device with any
    // available SAFE Authenticator rather than providing the authorisation credentials
    let auth_uri = "";

    let attributes = [
        ThingAttr::new("name", "SAFEthing Gardening Controller", false),
        ThingAttr::new("firmware", "v0.1.0", false),
        ThingAttr::new("status", "on", true),
    ];

    let topics = [];

    let actions = [];

    // Let's create an instance of SAFEthing for this device.
    // We already provide the two callback functions to be called
    // for subscriptions notifications and action requests respectively.
    let mut safe_thing =
        SAFEthing::new(&id, auth_uri, &subscriptions_notif, &|_, _, _, _, _| {}).unwrap();

    // Register the SAFEthing on the network, this won't make it active yet
    safe_thing
        .register(&attributes, &topics, &actions)
        .map(|()| println!("Gardening controller device registered on the network"))
        .unwrap();

    // Let's now make it active and ready for receiving action requests (although this
    // device doesn't expose any action or topic yet)
    safe_thing.publish().expect("Failed to publish SAFEthing");

    // The following is the SAFEthing id of the device we are controlling and monitoring.
    // This id is all we need to find it on the SAFE Network when using the SAFEthing API.
    let gardening_device_id = "gardening-device-serial-number-01010101";

    // The first thing we do is subscribe to a dynamic attribute ("moisture-level") of the gardening device.
    // We want to automatically receive a notification when the moisture level measured by
    // the gardening device falls below 5.0 (this is just the theshold we define to act upon).
    safe_thing
        .subscribe_to_attr(
            gardening_device_id,
            "moisture-level",
            FilterOperator::LessThan,
            "5.0",
        )
        .expect("Failed to subscribe to a dynamic attribute");

    // Let's also subscribe to one of the topics published by the gardening device ("VeryWetAlarm")
    // to receive notifications when the soil moisture level goes beyond a threshold that is
    // considered to be too wet and unhealthy for our plants.
    safe_thing
        .subscribe_to_topic(gardening_device_id, "VeryWetAlarm", FilterOperator::Any, "")
        .expect("Failed to subscribe to a topic");

    // Let's just wait for any events, this would usually be an infinite loop in a SAFEthing
    thread::sleep(Duration::from_millis(2000000));
}

fn subscriptions_notif(
    safe_thing: &SAFEthing,
    thing_id: &str,
    topic: &str,
    data: &str,
    timestamp: u128,
) {
    println!(
        "New event: Notification received from thing_id: '{}', topic: '{}', data: '{}', timestamp: {}",
        thing_id, topic, data, timestamp
    );

    // Let's act according to the notifications received...
    match topic {
        "moisture-level" => {
            // The soil moisture level felt below the threshold we defined (5.0) in the subscription
            // we made when invoking the `subscribe_to_attr` function.
            // Let's then send an action request to the gardening device to open the water valve,
            // so we can get the soil back to an state that is healthy for our plants.
            let req_id = safe_thing
                .action_request(
                    thing_id,    // the id of the gardening device where we are sending the action request to
                    "OpenValve", // request to open the water valve
                    &["60"],     // the desired water pressure
                    &handle_req_state_change, // callback to get notified about the state of this action request
                )
                .expect("Failed to send 'OpenValve' action request");

            println!(
                "Action request to OPEN the water valve sent, id: '{}'",
                req_id
            );
        }
        "VeryWetAlarm" => {
            // The gardening device is detecting that the soil is too wet now, so let's send an action
            // request to the device to close the water valve, our plant has enough water for now ;)
            let req_id = safe_thing
                .action_request(
                    thing_id,
                    "CloseValve", // request to close the water valve
                    &[],
                    &handle_req_state_change,
                )
                .expect("Failed to send 'CloseValve' action request");

            println!(
                "Action request to CLOSE the water valve sent, id: '{}'",
                req_id
            );
        }
        _ => eprintln!("Unexpected notification received, topic: {}", topic),
    };
}

fn handle_req_state_change(state: &str) -> bool {
    println!(
        "The action request sent to open/close the water valve was reported to be in state: '{}'",
        state
    );

    // We return 'true' to keep receiving state changes notifications for this action request
    // until the action is finally in "Done" state. Although in this particular case we won't be
    // receiving any "new state" report as the gardening device doesn't support any intermediate
    // states for the actions it supports.
    true
}
