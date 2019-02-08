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

use safe_thing::{SAFEthing, ThingAttr, Topic, ActionDef, AccessType};
use std::thread;
use std::time::Duration;

fn subscriptions_notif(thing_id: &str, topic: &str, data: &str) {
    println!("Printer #1: Notification received for thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "printer-serial-number-01010101";

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
        ThingAttr::new("wallet", "1KbCJfktc1JaKAwRtb42G8iNyhhh9zXRi4")
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

    let mut safe_thing = match SAFEthing::new(id, auth_uri, subscriptions_notif) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    match safe_thing.register(&attributes, &topics, &actions) {
       Ok(()) => println!("\nPrinter registered on the network"),
       Err(e) => println!("We got a problem!: {}", e)
   };

/*
    match safe_thing.status() {
        Ok(status) => println!("\nCurrent printer status: {}", status),
        Err(e) => println!("We got a problem!: {}", e)
    };

    match safe_thing.get_thing_attrs(id) {
        Ok(attrs) => println!("\nAttributes: {:?}", attrs),
        Err(e) => println!("We got a problem!: {}", e)
    }

    match safe_thing.get_thing_topics(id) {
        Ok(topics) => println!("\nTopics: {:?}", topics),
        Err(e) => println!("We got a problem!: {}", e)
    }

    match safe_thing.get_thing_actions(id) {
        Ok(actions) => println!("\nActions: {:?}", actions),
        Err(e) => println!("We got a problem!: {}", e)
    }
*/
    let _ = safe_thing.publish();

    match safe_thing.status() {
        Ok(status) => println!("\nCurrent status: {}", status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    println!("WAIT");
    thread::sleep(Duration::from_secs(10));
    println!("SENDING NOTIFICATION");
    let _ = safe_thing.notify("printRequested", "print job started");
    println!("NOTIFICATION SENT");
    thread::sleep(Duration::from_secs(15));
}
