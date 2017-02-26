extern crate safe_o_t;

use safe_o_t::{SAFEoT, ThingInfo, ThingAttr, Topic, ActionDef, AccessType};

fn printRequestedNotif(thing_id: &str, topic: &str, data: &str) {
    println!("Notification received from thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "sdienfionch439th34t4mtu894u8t34n9834ctn92pt8";
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

    let mut safeot: SAFEoT = SAFEoT::new(id).unwrap();
    match safeot.get_thing_info(id) {
        Ok(i) => println!("\nWe got info: {} - Status: {}", i, safeot.status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    safeot.register_thing(attributes, topics, actions);
    match safeot.get_thing_info(id) {
        Ok(i) => println!("\nWe got info: {} - Status: {}", i, safeot.status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    safeot.publish_thing(id);
    match safeot.get_thing_info(id) {
        Ok(i) => println!("\nWe got info: {} - Status: {}", i, safeot.status),
        Err(e) => println!("We got a problem!: {}", e)
    }

    safeot.subscribe(id, "printRequested", printRequestedNotif);

    safeot.notify("printRequested", "print job started");

}
