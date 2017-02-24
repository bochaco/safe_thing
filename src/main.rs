extern crate safe_o_t;

use safe_o_t::{get_thing_info, publish_thing, ThingInfo, ThingAttr, Topic, ActionDef, AccessType};

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

    let info: ThingInfo = publish_thing(id, attributes, topics, actions);

    let info2 = get_thing_info(id);
    println!("We got info: {} {}", info.id, info.addr_name);
}
