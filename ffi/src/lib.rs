extern crate safe_o_t;

use safe_o_t::{SAFEoT/*, ThingInfo, ThingAttr, Topic, ActionDef, AccessType*/};

#[no_mangle]
pub extern "C" fn hello_rust(n: i32) -> *const u8 {
    let thing_id = n.to_string();
    let mut safeot: SAFEoT = SAFEoT::new(thing_id.as_str()).unwrap();

    let mut t = String::from("Hello, world! - id: ");
    t.push_str(safeot.thing_id.as_str());
    t.push_str(", status: ");
    t.push_str("estado");
    t.as_ptr()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
//        hello_rust(5);
    }
}
