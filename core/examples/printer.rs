// Copyright 2017-2019 Gabriel Viganotti <@bochaco>.
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

extern crate safe_thing;

use safe_thing::{AccessType, ActionDef, FilterOperator, SAFEthing, ThingAttr, Topic};
use std::thread;
use std::time::Duration;

fn subscriptions_notif(thing_id: &str, topic: &str, data: &str, timestamp: u128) {
    println!(
        "New event: Notification received from thing_id: '{}', topic: '{}', data: '{}', timestamp: {}",
        thing_id, topic, data, timestamp
    )
}

fn action_request_notif(request_id: u128, thing_id: &str, action: &str, args: &[&str]) {
    println!(
        "New action: Action request received, id: '{}', from thing_id: '{}', action: '{}', args: {:?}",
        request_id, thing_id, action, args
    );
}

fn handle_print_req_state(state: &str) -> bool {
    println!("Print action request new state: '{}'", state);

    true // keep monitoring the state until it's "Done"
}

pub fn main() {
    // let's create a SAFEthing id, this could be the device serial number
    let id = "printer-serial-number-010101012";

    // for mock network
    let auth_uri = "bAEAAAAAGINJJUAAAAAAAAAAAAAQAAAAAAAAAAAF7WNLN4TB2BFIR4EDHZ7MNFC2QVGTKCQS37DBHXYFXBCQ6TDGZZEQAAAAAAAAAAAAGIBOIFK6G3YHVQ5ZWCDW3YTCEYE63JDCTNFUVBLOHSIJYTF3OQAQAAAAAAAAAAAHHDJHNVSAO3OHTCXZD3ZJLDTBTHRPMCBKULK6GR5LQ2DAEJBLG5JAAAAAAAAAAAAE6CDXUIA2XE5QPCGWPEDOQ7LGJGW73PK6Y47ZREBCBNRCRMEJ637TRUTW2ZAHNXDZRL4R54UVRZQZTYXWBAVKFVPDI6VYNBQCEQVTOUIAAAAAAAAAAAATOEKLDWVD42P23KAWYVKQUMTWFN7BXQ2N2245J5PDAXJ44ABJEWIAAAAAAAAAAABAALXMFJ5L5Q2MDQQPH7EOAH4UZX5YMKWTJVH23SYD6UY2VAHTIIAAAAAAAAAAAAAAAAAAAAAAAAAGV5SLI344FOAMMI27CCICD66DHHDR3VEYI2EOUPU4VXTS4SJYEC2MDUAAAAAAAAAAYAAAAAAAAAAAF2WYHMI677OZLG252KI2NQXW2CIR5HIPVCIYBAQAAAAAAAAAAAAAB";

    let attributes = [
        ThingAttr::new("name", "SAFEthing Printer", false),
        ThingAttr::new("model", "ArduinoDigital PRT1", false),
        ThingAttr::new("firmware", "v0.1.0", false),
        ThingAttr::new("status", "on", true),
        ThingAttr::new("ink-level", "%", true),
        ThingAttr::new("service-price", "1", false),
        ThingAttr::new("payment-window", "60000", false),
        ThingAttr::new("wallet", "1KbCJfktc1JaKAwRtb42G8iNyhhh9zXRi4", false),
    ];

    let topics = [
        Topic::new("printRequested", AccessType::All),
        Topic::new("printPaid", AccessType::All),
        Topic::new("printSuccess", AccessType::All),
        Topic::new("printFail", AccessType::All),
        Topic::new("copyDelivered", AccessType::All),
        Topic::new("copyNotDelivered", AccessType::All),
        Topic::new("outOfInk", AccessType::All),
    ];

    let actions = [
        ActionDef::new("turnOn", AccessType::Owner, &[]),
        ActionDef::new("turnOff", AccessType::Owner, &["timer"]),
        ActionDef::new("print", AccessType::All, &["data", "deliverTo"]),
        ActionDef::new("orderInk", AccessType::Owner, &[]),
    ];

    // Let's create an instance of SAFEthing for the printer.
    // We already provide the two callback functions to be called
    // for subcriptions notifications and action requests respectively.
    let mut safe_thing =
        SAFEthing::new(&id, auth_uri, &subscriptions_notif, &action_request_notif).unwrap();

    // Register the SAFEthing on the network, this won't make it active yet
    safe_thing
        .register(&attributes, &topics, &actions)
        .map(|()| println!("Printer registered on the network"))
        .unwrap();

    /*
        safe_thing
            .status()
            .map(|status| println!("Current printer status: {}", status))
            .unwrap();

        safe_thing
            .get_thing_attrs(&id)
            .map(|attrs| println!("Attributes: {:?}", attrs))
            .unwrap();

        safe_thing
            .get_thing_topics(&id)
            .map(|topics| println!("Topics: {:?}", topics))
            .unwrap();

        safe_thing
            .get_thing_actions(&id)
            .map(|actions| println!("Actions: {:?}", actions))
            .unwrap();
    */

    // Let's now make it active and ready for receiving action requests
    safe_thing.publish().expect("Failed to publish SAFEthing");

    // ***** this shall be part of a printer_consumer app *****
    // for testing as it probably doesn't make sense
    // to subscribe to its own events
    safe_thing
        .subscribe_to_topic(&id, "printRequested", &[])
        .expect("Failed to subscribe to a topic");

    safe_thing
        .subscribe_to_attr(&id, &[("ink-level", FilterOperator::LessThan, "10")])
        .expect("Failed to subscribe to a dynamic attribute");

    thread::sleep(Duration::from_millis(6000));
    println!("SENDING NOTIFICATION");
    let _ = safe_thing.notify("printRequested", "print job started");
    println!("NOTIFICATION SENT");
    thread::sleep(Duration::from_millis(20000));

    /*let req_id = safe_thing
        .action_request(
            &id,
            "print",
            &["some text", "recipient"],
            &handle_print_req_state,
        )
        .expect("Failed to send 'print' action request");
    println!("Action request sent, id: '{}'", req_id);*/
    /***** END printer_consumer *************/

    // Let's just wait for any request sent,
    // this would usually be an infinite loop in a SAFEthing
    thread::sleep(Duration::from_millis(9000));
}
