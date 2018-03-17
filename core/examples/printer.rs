extern crate safe_thing;

use safe_thing::{SAFEthing, ThingAttr, Topic, ActionDef, AccessType};

fn print_requested_notif(thing_id: &str, topic: &str, data: &str) {
    println!("Notification received from thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "printer-serial-number-01010101";

    // for mock
    let auth_uri = "AQAAAJL_DCEAAAAAAAAAACAAAAAAAAAA82fAWBOIGLAchU6n-guPZ6imzRVN2hGdFUr-Sjo-BBogAAAAAAAAABu6nck8eWKc9igs1b9j-tFZ-6uE6Z2OdQslZ0XEGMYuIAAAAAAAAAAJDBOQ4hHBeQddQ4WZ2zI8cW0LR8EhgBkZvkukUq2oy0AAAAAAAAAAfMn5B3D-7PSe_qE5me9NqutTfuWuFHkD5r3f9qsow30JDBOQ4hHBeQddQ4WZ2zI8cW0LR8EhgBkZvkukUq2oyyAAAAAAAAAAwpV4a5u-m53rxxebHXFD27QfcKogUJ_cQAQtUqjunFUgAAAAAAAAAFYkmckKJ1ADzvib3pkxWap6Ktl8gur25u16kmw6oKGLAAAAAAAAAAAAAAAAAAAAACSANxt1wwhwNzgeyKd4h79VQuO_JLIBH7kkeXl0PjnGmDoAAAAAAAAYAAAAAAAAAPpAT0823S4DmPEEX3b2gqoz46yRycVLwQMAAAAAAAAADAAAAAAAAABfcHVibGljTmFtZXM_ueEfCwto_U_Z2lAaMx9U9IaUFSZgu6n0OcNfIrIgJpg6AAAAAAAAASAAAAAAAAAAfByCajuzlHNClpkW_AJXCwQKKPQ99uJnh-M0qtiDvsMYAAAAAAAAAE0tWSQUWZ2wGVrxtbfsqZc3rZCDEQ5S-wACAAAAAAAAAAAAAAABAAAAIAAAAAAAAABhcHBzL25ldC5tYWlkc2FmZS50ZXN0LndlYmFwcC5pZP0Rr5H9ANW73ILl8k9RjfPghmyLkMmfFCHsr9_zuuuumDoAAAAAAAABIAAAAAAAAACWn-k9ud_kgeEuEfsVlZlux1elPQA92Yxu13GsXrQFxxgAAAAAAAAA4t6AHWblHfJsr7uIBY0KT5uj9BbGj52OAAUAAAAAAAAAAAAAAAEAAAACAAAAAwAAAAQAAAAHAAAAAAAAAF9wdWJsaWN7jgqShYtZJpCstBf-xbAgcFY0TshTeHNOTI8RNgcRiZg6AAAAAAAAAAADAAAAAAAAAAAAAAABAAAAAwAAAA";

    // for live net
    //let auth_uri = "AQAAAIwHbz8AAAAAAAAAACAAAAAAAAAAmWr9bLsYL6RAMwGjHHmsD0G6_U2jqm8kkb51INtXlOsgAAAAAAAAAGTAAyF9j9vI55cEDysXqxHT9mOdQnwp5idSVWraxQt0IAAAAAAAAABHfsW8ko-RsM8QL8xDoEgciyadhYhL_yxWlQ-az045wEAAAAAAAAAAsSbw3NCpAvIt7eZeMDOTC-4GqaHZKybLe-m71g5CFq1HfsW8ko-RsM8QL8xDoEgciyadhYhL_yxWlQ-az045wCAAAAAAAAAALhp_iiIHXVjDKzT6qDQlo7G4HpSjli1hxIkj5oJEa2EgAAAAAAAAAMhtQ5D1sSpTNMN1HM0h_P_dDphg5oNJb2XJr63zWNW2GQAAAAAAAAAQAAAAAAAAADE3OC42Mi43Ni44OjU0ODMTAAAAAAAAADEzOC42OC4xODUuMjE4OjU0ODMSAAAAAAAAADEzOC42OC4xODEuNTc6NTQ4MxIAAAAAAAAAMTM4LjY4LjE4MS42MDo1NDgzEgAAAAAAAAAxMzguNjguMTgxLjg2OjU0ODMSAAAAAAAAADEzOC42OC4xODEuODc6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4xNjg6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4xNzY6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4xNzk6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4xODA6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4xODI6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4yNDI6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4yNDM6NTQ4MxMAAAAAAAAAMTM4LjY4LjE4MS4yNDk6NTQ4MxIAAAAAAAAAMTM4LjY4LjE4OS4xNDo1NDgzEgAAAAAAAAAxMzguNjguMTg5LjE1OjU0ODMSAAAAAAAAADEzOC42OC4xODkuMTc6NTQ4MxIAAAAAAAAAMTM4LjY4LjE4OS4xODo1NDgzEgAAAAAAAAAxMzguNjguMTg5LjE5OjU0ODMSAAAAAAAAADEzOC42OC4xODkuMzE6NTQ4MxIAAAAAAAAAMTM4LjY4LjE4OS4zNDo1NDgzEgAAAAAAAAAxMzguNjguMTg5LjM2OjU0ODMSAAAAAAAAADEzOC42OC4xODkuMzg6NTQ4MxIAAAAAAAAAMTM4LjY4LjE4OS4zOTo1NDgzEQAAAAAAAAA0Ni4xMDEuNS4xNzk6NTQ4MwAAAAAAAAEHAAAAAAAAAGFscGhhXzIADnHT11l22Ruof4NAui_3sg7s3_FoGol_7u614PPHgPGYOgAAAAAAABgAAAAAAAAAYU41z2uM27rHebM9zh0T3ZoixFivPqLWAwAAAAAAAAAHAAAAAAAAAF9wdWJsaWOsAmnwijoPNsxWomwo9Udpe9MeQjDmcMqlgDv0Pq-kAZg6AAAAAAAAAAADAAAAAAAAAAAAAAABAAAAAwAAACAAAAAAAAAAYXBwcy9uZXQubWFpZHNhZmUudGVzdC53ZWJhcHAuaWQ6QVAfG9CotoElzOIPC4ixl3GQ4Ra6PLR6BXjRpSZC_Jg6AAAAAAAAASAAAAAAAAAAibkkWRvTJsOdmOiqRwa98aGk6C-JHTKYUzKlUkrRvfwYAAAAAAAAAPczxU4TLlGaCeyjVlmAkWJwPUaUZhVV-gAFAAAAAAAAAAAAAAABAAAAAgAAAAMAAAAEAAAADAAAAAAAAABfcHVibGljTmFtZXPxOOQ3CK9TBKLyWp_MlRWUHfjleiln8-IoDzaZdVsGm5g6AAAAAAAAASAAAAAAAAAA2IAqW9KH9i5Z4PClnWXpaovZU7SfErT_G5iXIKeeVKcYAAAAAAAAAH3RtQXIptnJfLhWb0A5NNA02vaNAJnCJwACAAAAAAAAAAAAAAABAAAA";

    let attributes = vec![
        ThingAttr::new("name", "SAFE Printer"),
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

    let mut safe_thing: SAFEthing = match SAFEthing::new(id, auth_uri) {
        Ok(s) => s,
        Err(e) => panic!("{}", e)
    };

    match safe_thing.register(attributes, topics, actions, print_requested_notif) {
       Ok(()) => println!("\nPrinter registered on the network"),
       Err(e) => println!("We got a problem!: {}", e)
   };

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

    let _ = safe_thing.publish();
    match safe_thing.status() {
        Ok(status) => println!("\nCurrent status: {}", status),
        Err(e) => println!("We got a problem!: {}", e)
    }
/*
    let _ = safe_thing.subscribe(id, "printRequested");
    //thread::sleep(Duration::from_secs(5));

    let _ = safe_thing.notify("printRequested", "print job started");
*/

}
