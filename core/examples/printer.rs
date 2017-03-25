extern crate safe_o_t;

use std::thread;
use std::time::Duration;
use safe_o_t::{SAFEoT, ThingAttr, Topic, ActionDef, AccessType};

fn print_requested_notif(thing_id: &str, topic: &str, data: &str) {
    println!("Notification received from thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "printer-serial-number-1";
    let auth_token = "safe-bmV0Lm1haWRzYWZlLnRlc3Qud2ViYXBwLmlk:AAAAAX54wJsAAAAAAAAAAAAAAAAAAAAgyK2b4gfAQVuMmjIHW6g0wLN6JsiM-rFxVFkHIvBrThgAAAAAAAAAILSMNzIsMsb3RbWkpS34xA5Gro74XCeoNf-ScnxU4PyHAAAAAAAAACCkMBe7eSn224RJsH9kOeWLRExeP2J4daX_cpiGdyx5nQAAAAAAAABA1No25wKSvXFiexPQGYer1zZNgnVcTl7iHFwtVa7MZl-kMBe7eSn224RJsH9kOeWLRExeP2J4daX_cpiGdyx5nQAAAAAAAAAgqyJzijuoO_aYFL9rRmP2oUXFIVJqLq2Z44sb8CYQQyUAAAAAAAAAIM6Qb6BgqGiQMs51WujArJ0ISll2QQNh-m_bBnrBIgoEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIM41EhSoR2AniPZYpv8QTpTAJzB8Eiau5TZF5DqWD2TeAAAAAAAAOpgAAAAAAAAAGETLKLNYCX5hkDQRCgTjoAEX8OeZdEUwyQ";

    let attributes = vec![
        ThingAttr::new("name", "Printer at home"),
        ThingAttr::new("model", "HP LaserJet 400 M401"),
        ThingAttr::new("firmware", "v1.3.0"),
        ThingAttr::new("status", "on"),
        ThingAttr::new("ink-level", "70%"),
        ThingAttr::new("service-price", "1"),
        ThingAttr::new("payment-timeout", "60000"),
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
        ActionDef::new("deiverCopy", AccessType::Thing, vec![])
    ];

    let mut safeot: SAFEoT;
    match SAFEoT::new(id, auth_token, print_requested_notif) {
        Ok(s) => safeot = s,
        Err(e) => panic!("{}", e)
    }

    let _ = safeot.register_thing(attributes, topics, actions);
    thread::sleep(Duration::from_secs(2));

    match safeot.get_thing_status(id) {
        Ok(status) => println!("\nWe got status: {:?}", status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    match safeot.get_thing_addr_name(id) {
        Ok(addr_name) => println!("\nWe got address name: {:?}", addr_name),
        Err(e) => println!("We got a problem!: {}", e)
    }
/*
    match safeot.get_thing_attrs(id) {
        Ok(attrs) => println!("\nWe got attrs: {:?}", attrs),
        Err(e) => println!("We got a problem!: {}", e)
    }

    match safeot.get_thing_topics(id) {
        Ok(topics) => println!("\nWe got topics: {:?}", topics),
        Err(e) => println!("We got a problem!: {}", e)
    }

    match safeot.get_thing_actions(id) {
        Ok(actions) => println!("\nWe got actions: {:?}", actions),
        Err(e) => println!("We got a problem!: {}", e)
    }

    let _ = safeot.publish_thing(id);
    match safeot.get_thing_status(id) {
        Ok(status) => println!("\nWe got status: {:?}", status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    let _ = safeot.subscribe(id, "printRequested");
    //thread::sleep(Duration::from_secs(5));

    let _ = safeot.notify("printRequested", "print job started");
*/

    thread::sleep(Duration::from_secs(2));
}
