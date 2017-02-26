//use rust_sodium::crypto::hash::sha256;
use std::fmt;
use std::{time, thread};

mod comm;
use comm::SAFEoTComm;

/// Which set of Things are allow to register to a topic
/// Thing: access only to the thing's application.
/// Owner: access also is allowed to an individual, application or system that is the actual owner of the Thing, plus the Thing itself.
/// Group: access to a group of individuals or Things, plus the Owner and the Thing itself.
/// All: access is allowed to anyone or anything, including Owner and the Thing itself.
#[derive(Clone, Debug)]
pub enum AccessType {
    Thing,
    Owner,
    Group,
    All
}

/// Status of the Thing in the network
/// Unregistered: the Thing is not even registered in the network, only its ID is known in the framework
/// Registered: the Thing was registered but it's not published yet, which means it's not operative yet for subscribers
/// Published: the Thing was plublished and it's operative, allowing Things to subscribe an interact with it
pub enum Status {
    Unregistered,
    Registered,
    Published
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Status::Unregistered => "Unregistered",
            Status::Registered => "Registered",
            Status::Published => "Published"
        })
    }
}

/// Topic name and access type
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ThingAttr {
    pub attr: String,
    pub value: String
}

impl ThingAttr {
    pub fn new(attr: &str, value: &str) -> ThingAttr {
        ThingAttr {attr: String::from(attr), value: String::from(value)}
    }
}

/// Actions that can be requested to a Thing
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ThingInfo {
    pub id: String,
    pub addr_name: String,
    pub attrs: Vec<ThingAttr>,
    pub topics: Vec<Topic>,
    pub actions: Vec<ActionDef>
}

impl fmt::Display for ThingInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(id: {}, addr_name: {})", self.id, self.addr_name)
    }
}

// This will be removed when we have the mock or the integration with the safe_app library
struct MutableData {
    name: String,
    type_tag: u64
}

impl MutableData {
    pub fn new(id: &String, type_tag: u64) -> MutableData {
        //let Digest(digest) = sha256::hash(id.as_bytes());
        let mut id = String::from(id.as_str());
        id.push_str("0101010");
        MutableData {name: id, type_tag: type_tag}
    }
}


pub struct SAFEoT {
    pub thing_id: String,
    pub status: Status,
    storage: Option<ThingInfo>,
    safeot_comm: SAFEoTComm,
    notifs_cb: fn(&str, &str, &str)
}

fn cb_default(_: &str, _: &str, _: &str) {
    println!("No callback defined");
}

impl SAFEoT {
    pub fn new(thing_id: &str) -> Result<SAFEoT, String> {
        println!("SAFEoT instance created with Thing ID: {}", thing_id);
        let safeot = SAFEoT {
            thing_id: String::from(thing_id),
            status: Status::Unregistered,
            storage: None,
            safeot_comm: SAFEoTComm::new(thing_id),
            notifs_cb: cb_default
        };

        thread::spawn(|| {
            thread::sleep(time::Duration::from_millis(10000));
            println!("this is thread number {}", 1);
            //(safeot.notifs_cb)(safeot.thing_id.as_str(), "topic", "data");
        });

        Ok(safeot)
    }

    /// Register and re-register a SAFE Thing specifying its attributes,
    /// events/topics and available actions
    pub fn register_thing(&mut self, attrs: Vec<ThingAttr>, topics: Vec<Topic>, actions: Vec<ActionDef>) -> Option<String> {
        // Register it in the network
        let md = MutableData::new(&self.thing_id, 15000);

        let info: ThingInfo = ThingInfo {
            id: String::from(self.thing_id.as_str()),
            addr_name: md.name,
            attrs: attrs,
            topics: topics,
            actions: actions
        };

        println!("Thing registered wih id {:?}", info.id);
        self.storage = Some(info);
        self.status = Status::Registered;
        None
    }

    /// Publish the thing making it available and operative in the network, allowing other Things
    /// to request actions, subscribe to topics, and receive notifications for events.
    pub fn publish_thing(&mut self, thing_id: &str) -> Option<String> {
        // Publish it in the network
        println!("Thing published wih id {:?}", thing_id);
        self.status = Status::Published;
        None
    }

    /// Get information about a Thing in order to then subscribe to topics supported
    pub fn get_thing_info(&self, thing_id: &str) -> Result<ThingInfo, String> {
        // Search on the network by thing_id
        self.storage.clone().ok_or("The thing doesn't contain information".to_owned())
    }

    /// Subscribe to topics published by a Thing (all data is stored in the network to support device resets/reboots)
    /// Eventually this can support filters
    pub fn subscribe(&mut self, thing_id: &str, topic: &str, cb: fn(&str, &str, &str)) -> Option<String>
    {
        self.safeot_comm.addSubscription(thing_id, topic);
        self.notifs_cb = cb;
        None
    }

    /// Notify of an event associated to an spefici topic.
    /// Eventually this can support multiple topics.
    pub fn notify(&mut self, topic: &str, data: &str) -> Option<String>
    {
        self.safeot_comm.publishEvent(topic, data);
        println!("Event occurred, send notification for topic: {}, thing_id: {}", topic, self.thing_id);

        (self.notifs_cb)(self.thing_id.as_str(), "topic", "data");

        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
