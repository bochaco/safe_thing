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

use safe_thing::{AccessType, ActionDef, SAFEthing, ThingAttr, Topic};
use std::time::{SystemTime, UNIX_EPOCH};

fn subscriptions_notif(thing_id: &str, topic: &str, data: &str) {
    println!(
        "Printer #1: Notification received from thing_id: {}, topic: {}, data: {}",
        thing_id, topic, data
    )
}

pub fn main() {
    // let's create a random SAFEthing id for tetign purposes
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let id = format!("printer-serial-number-{:?}", since_the_epoch);

    // for mock
    let auth_uri = "bAEAAAABGXJVR6AAAAAAAAAAAAAQAAAAAAAAAAAC5EQI4XKHIVKRYNULVZALV26XFTCUMA53ABDZEVIUQPRF6OZWVEEQAAAAAAAAAAAAAVA7KSQEG6CORP7TXB3NTFO5YT23HQQ6TRENT3D5V27ZLA5GD2AQAAAAAAAAAAABRRCGSRFZ3OYPAEB2T4IY6FRIZAIP3A3L3TEPCYHYJVY5OKSJ6WFAAAAAAAAAAAAGAU3SZXJF5WYG4IRXMKQ2GALCEB5QLBL7YFGNUSEFK7JOREI6LBAYYRDJIS45XMHQCA5J6EMPCYUMQEH5QNV5ZSHRMD4E24OXFJE7LCIAAAAAAAAAAABFJE77ZSFAMLDDJT7WV25LIPK54L5HYSI2IFFSDAQUWLELNPA7VWIAAAAAAAAAAADYAAKQ3MRXOOF2JFBVGC3X3U2WOULFDD5HADEVZG2SAAA7R4CJZSAAAAAAAAAAAAAAAAAAAAAAAAACOFVBXHPHW24CPVQRIK22GNZTDWPXCCWSCLHJXUL42K4ILQTFFUGMDUAAAAAAAAAAYAAAAAAAAAAAKXVATQLDZOEVOGPQJQM7456E5FKHQKPBZRSOU2UBAAAAAAAAAAAAHAAAAAAAAAAAF64DVMJWGSY6Z5FM5UJYCEKBSMDFQ62WWYDUZGGCUGS3PWLU6J7FJLAVYG62XVKMDUAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAACQAAAAAAAAAABPWI33DOVWWK3TUOOC4QT7NYZQT4SQJRSSGOJGLGRSZLTEBF5HG5TBBW6RTET422Q7MNGB2AAAAAAAAAAASAAAAAAAAAAAAWOIVVABZOJETOB372WNSOWUJ5FHAYWH3SUYFHQENEZ4NQQIFEEZBQAAAAAAAAAAA3EEMVKL5EQUISMN7H3VHSVMF76IP7QN3UNIRHRYAAEAAAAAAAAAAAAIAAAAAC";

    // for live net
    //let auth_uri = "";

    let attributes = vec![
        ThingAttr::new("name", "SAFEthing Printer"),
        ThingAttr::new("model", "ArduinoDigital PRT1"),
        ThingAttr::new("firmware", "v0.1.0"),
        ThingAttr::new("status", "on"),
        ThingAttr::new("ink-level", "%"),
        ThingAttr::new("service-price", "1"),
        ThingAttr::new("payment-window", "60000"),
        ThingAttr::new("wallet", "1KbCJfktc1JaKAwRtb42G8iNyhhh9zXRi4"),
    ];
    let topics = vec![
        Topic::new("printRequested", AccessType::All),
        Topic::new("printPaid", AccessType::All),
        Topic::new("printSuccess", AccessType::All),
        Topic::new("printFail", AccessType::All),
        Topic::new("copyDelivered", AccessType::All),
        Topic::new("copyNotDelivered", AccessType::All),
        Topic::new("outOfInk", AccessType::All),
    ];
    let actions = vec![
        ActionDef::new("turnOn", AccessType::Owner, vec![]),
        ActionDef::new("turnOff", AccessType::Owner, vec!["timer"]),
        ActionDef::new("print", AccessType::All, vec!["data"]),
        ActionDef::new("orderInk", AccessType::Owner, vec![]),
    ];

    let mut safe_thing = SAFEthing::new(&id, auth_uri, subscriptions_notif).unwrap();

    safe_thing
        .register(&attributes, &topics, &actions)
        .map(|()| println!("\nPrinter registered on the network"))
        .unwrap();

    safe_thing
        .status()
        .map(|status| println!("\nCurrent printer status: {}", status))
        .unwrap();

    safe_thing
        .get_thing_attrs(&id)
        .map(|attrs| println!("\nAttributes: {:?}", attrs))
        .unwrap();

    safe_thing
        .get_thing_topics(&id)
        .map(|topics| println!("\nTopics: {:?}", topics))
        .unwrap();

    safe_thing
        .get_thing_actions(&id)
        .map(|actions| println!("\nActions: {:?}", actions))
        .unwrap();

    safe_thing.publish().expect("Failed to publish SAFEthing");

    safe_thing
        .status()
        .map(|status| println!("\nCurrent status: {}", status))
        .unwrap();

    // for testing as it probably doesn't make sense
    // to subscribe to its own events
    safe_thing
        .subscribe(&id, "printRequested")
        .expect("Failed to subscribe to a topic");

    let res = safe_thing
        .action_request(&id, "print", vec!["arg1".to_string(), "arg2".to_string()])
        .unwrap();
    println!("Response received for action request: {}", res);

    println!("SENDING NOTIFICATION");
    let _ = safe_thing.notify("printRequested", "print job started");
    println!("NOTIFICATION SENT");

    safe_thing
        .check_subscriptions()
        .expect("Failed to check subscriptions");

    // safe_thing.simulate_net_disconnect();

    safe_thing
        .status()
        .map(|status| println!("\nCurrent status: {}", status))
        .unwrap();
}
