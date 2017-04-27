extern crate safe_o_t;

use safe_o_t::{SAFEoT, ThingAttr, Topic, ActionDef, AccessType};

fn print_requested_notif(thing_id: &str, topic: &str, data: &str) {
    println!("Notification received from thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "printer-serial-number-01010101";
    let auth_token = "safe-bmv0lm1hawrzywzllmv4yw1wbgvzlm1krxhhbxbszq:AQAAAOIDI_gAAAAAAAAAACAAAAAAAAAAGWzDHH2GG-TUtS_qLvytHNXrAPWGtI6QLDuoP28EE_0gAAAAAAAAALPyoRvbtvPKs9bWYgsQvT3strDfQsw4HXRzNW_cfmxTIAAAAAAAAAD_a6ysxSGIUWz9pOLlq9hRMM-EJQctDpVkhRTXPar-W0AAAAAAAAAA-O8HsVV5ZZbiAwWTTFXQeNX7pSYtLmZXRHnrdVyXZvv_a6ysxSGIUWz9pOLlq9hRMM-EJQctDpVkhRTXPar-WyAAAAAAAAAAUnTeCf39C-KDfioarbgDedqYhu_ZEpCHK_CatkiYNFUgAAAAAAAAAOTkFE7GibxaH0egTV1NtczggZkyAsCVRY6AcbceiSNfAAAAAAAAAAAAAAAAAAAAAAAAAAAAMCralz2EJh0ML2wMZLBhh0hELI1dIQUlVtaWHqIClqmYOgAAAAAAABgAAAAAAAAA2lo16ByCIq4SnojMIRPV_RSvQIOelGUD";

    let attributes = vec![
        ThingAttr::new("name", "Printer at home"),
        ThingAttr::new("model", "HP LaserJet 400 M401"),
        ThingAttr::new("firmware", "v1.3.0"),
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
        ActionDef::new("deiverCopy", AccessType::Thing, vec![])
    ];

    let mut safeot: SAFEoT;
    match SAFEoT::new(id, auth_token, print_requested_notif) {
        Ok(s) => safeot = s,
        Err(e) => panic!("{}", e)
    }

    let _ = safeot.register_thing(attributes, topics, actions);

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

}
