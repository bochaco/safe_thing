//use rust_sodium::crypto::hash::sha256;

/// Which set of Things are allow to register to a topic
/// Thing: access only to the thing's application.
/// Owner: access also is allowed to an individual, application or system that is the actual owner of the Thing, plus the Thing itself.
/// Group: access to a group of individuals or Things, plus the Owner and the Thing itself.
/// All: access is allowed to anyone or anything, including Owner and the Thing itself.
pub enum AccessType {
    Thing,
    Owner,
    Group,
    All
}

/// Topic name and access type
pub struct Topic {
    pub name: String,
    pub access: AccessType
}

impl Topic {
    pub fn new(name: &str, access: AccessType) -> Topic {
        Topic {name: String::from(name), access: access}
    }
}

/// This is the structure which defines the attributes of a SAFE Thing
pub struct ThingAttr {
    pub attr: String,
    pub value: String
}

impl ThingAttr {
    pub fn new(attr: &str, value: &str) -> ThingAttr {
        ThingAttr {attr: String::from(attr), value: String::from(value)}
    }
}

/// Actions that can be request to a Thing
pub struct ActionDef {
    pub name: String,
    pub access: AccessType,
    pub args: Vec<String> // arg name, the values are opaque for the framework
}

impl ActionDef {
    pub fn new(name: &str, access: AccessType, args: Vec<&str>) -> ActionDef {
        let mut arguments = vec![];
        for arg in &args {
            arguments.push(String::from(*arg));
        }
        ActionDef {name: String::from(name), access: access, args: arguments}
    }
}

/// Information of a Thing describing all published attributes, topics and actions
pub struct ThingInfo {
    pub id: String,
    pub addr_name: String,
    pub attrs: Vec<ThingAttr>,
    pub topics: Vec<Topic>,
    pub actions: Vec<ActionDef>
}

// This will be removed when we have the mock or the integration with the safe_app library
struct MutableData {
    name: String,
    type_tag: u64
}

impl MutableData {
    pub fn new(id: &str, type_tag: u64) -> MutableData {
        //let Digest(digest) = sha256::hash(id.as_bytes());
        MutableData {name: String::from(id) + "0101010", type_tag: type_tag}
    }
}

/// Publish and re-publish a SAFE Thing specifying its attributes,
/// events/topics and available actions
pub fn publish_thing(thing_id: &str, attrs: Vec<ThingAttr>, topics: Vec<Topic>, actions: Vec<ActionDef>) -> ThingInfo {

    let md = MutableData::new(thing_id, 15000);

    let info: ThingInfo = ThingInfo {
        id: String::from(thing_id),
        addr_name: md.name,
        attrs: attrs,
        topics: topics,
        actions: actions
    };

    println!("Thing published wih id {:?}", info.id);
    info
}

/// Get information about a Thing in order to then subscribe to topics supported
pub fn get_thing_info(thing_id: &str) -> ThingInfo {

    let md = MutableData::new(thing_id, 15000);

    let info = ThingInfo {
        id: String::from(thing_id),
        addr_name: md.name,
        attrs: vec![ThingAttr {attr: String::from("name"), value: String::from("Printer at home")}],
        topics: vec![Topic {name: String::from("printRequested"), access: AccessType::Group}],
        actions: vec![]
    };

    return info;
}

/// Subscribe to supported topics accepted by a Thing (all data is stored in the network to support device resets/reboots)
/// Eventually this can support filters
pub fn subscribe(thing_id: String, topic: String/*, cb: Fn*/) -> () {
    println!("A {} {}", thing_id, topic);
}

//store_subscription(...)
//fetch_subscription(...)


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
