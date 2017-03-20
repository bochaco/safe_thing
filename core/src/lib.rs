#[macro_use]
extern crate serde_derive;

extern crate serde_json;

mod comm;
mod errors;

use std::collections::BTreeMap;
use std::thread;
use std::time::Duration;
use comm::{SAFEoTComm, ActionArgs};
use errors::{ResultReturn, Error, ErrorCode};

use std::fmt;

/// Which set of Things are allow to register to a topic
/// Thing: access only to the thing's application.
/// Owner: access also is allowed to an individual, application or system that is the actual owner of the Thing, plus the Thing itself.
/// Group: access to a group of individuals or Things, plus the Owner and the Thing itself.
/// All: access is allowed to anyone or anything, including Owner and the Thing itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub access: AccessType,
    pub args: ActionArgs
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

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
enum FilterOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
struct EventFilter {
    arg_name: String,
    arg_op: FilterOperator,
    arg_value: String
}

/// A subscription is a map from Topic to a list of filters
type Subscription = BTreeMap<String, Vec<EventFilter>>;

pub struct SAFEoT {
    pub thing_id: String,
    safeot_comm: SAFEoTComm,
    subscriptions: BTreeMap<String, Subscription>,
    notifs_cb: fn(&str, &str, &str)
}

impl SAFEoT {
    pub fn new(thing_id: &str, notifs_cb: fn(&str, &str, &str)) -> ResultReturn<SAFEoT> {
        println!("SAFEoT instance created with Thing ID: {}", thing_id);

        let thread = thread::spawn(move || {
            loop {
                println!("Checking events...");
//                for (thing_id, subs) in self.subscriptions.iter() {
//                    for (topic, filter) in subs.iter() {
                        //self.safeot_comm.get_topic_events(topic).map(|events| {
                            //println!("Event occurred for topic: {}, data: {}", topic, events);
                            //notifs_cb(thing.as_str(), topic.as_str(), events.as_str());
                            notifs_cb("thing", "printRequested", "events");
                        //});
//                    }
//                }
                thread::sleep(Duration::from_secs(2));
            }
        });

        let safeot = SAFEoT {
            thing_id: String::from(thing_id),
            safeot_comm: SAFEoTComm::new(thing_id)?,
            subscriptions: BTreeMap::new(),
            notifs_cb: notifs_cb
        };

        Ok(safeot)
    }

    /// Register and re-register a SAFE Thing specifying its attributes,
    /// events/topics and available actions
    pub fn register_thing(&mut self, attrs: Vec<ThingAttr>,
                            topics: Vec<Topic>, actions: Vec<ActionDef>) -> ResultReturn<()> {
        // Register it in the network
        self.safeot_comm.store_thing_entity(15000);

        // Populate entity with attributes
        let attrs: String = serde_json::to_string(&attrs).unwrap();
        self.safeot_comm.set_attributes(attrs.as_str());

        // Populate entity with topics
        let topics: String = serde_json::to_string(&topics).unwrap();
        self.safeot_comm.set_topics(topics.as_str());

        // Populate entity with actions
        let actions: String = serde_json::to_string(&actions).unwrap();
        self.safeot_comm.set_actions(actions.as_str());

        self.safeot_comm.set_status("Registered");

        println!("Thing registered wih id: {}", self.thing_id);
        Ok(())
    }

    /// Get status of a Thing
    pub fn get_thing_status(&self, thing_id: &str) -> ResultReturn<String> {
        // Search on the network by thing_id
        let status = self.safeot_comm.get_thing_status(thing_id)?;
        Ok(status)
    }

    /// Get address name of a Thing
    pub fn get_thing_addr_name(&self, thing_id: &str) -> ResultReturn<String> {
        // Search on the network by thing_id
        let addr_name = self.safeot_comm.get_thing_addr_name(thing_id)?;
        Ok(addr_name)
    }

    /// Get list of attrbiutes of a Thing
    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<Vec<ThingAttr>> {
        // Search on the network by thing_id
        let attrs_str = self.safeot_comm.get_thing_attrs(thing_id)?;
        let attrs: Vec<ThingAttr> = serde_json::from_str(&attrs_str).unwrap();
        Ok(attrs)
    }

    /// Get list of topics supported by a Thing
    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<Vec<Topic>> {
        // Search on the network by thing_id
        let topics_str = self.safeot_comm.get_thing_topics(thing_id)?;
        let topics: Vec<Topic> = serde_json::from_str(&topics_str).unwrap();
        Ok(topics)
    }

    /// Get list of actions supported by a Thing
    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<Vec<ActionDef>> {
        // Search on the network by thing_id
        let actions_str = self.safeot_comm.get_thing_actions(thing_id)?;
        let actions: Vec<ActionDef> = serde_json::from_str(&actions_str).unwrap();
        Ok(actions)
    }

    /// Publish the thing making it available and operative in the network, allowing other Things
    /// to request actions, subscribe to topics, and receive notifications upon events.
    pub fn publish_thing(&mut self, thing_id: &str) -> ResultReturn<()> {
        // Publish it in the network
        println!("Thing published wih id {:?}", thing_id);
        self.safeot_comm.set_status("Published");
        Ok(())
    }

    /// Subscribe to topics published by a Thing (all data is stored in the network to support device resets/reboots)
    /// Eventually this can support filters
    pub fn subscribe(&mut self, thing_id: &str, topic: &str/*, filter*/) -> ResultReturn<()>
    {
        self.subscriptions.entry(String::from(thing_id)).or_insert(BTreeMap::new());

        let thing = String::from(thing_id);
        self.subscriptions.get_mut(&thing).map(|subs| {
            let filters: Vec<EventFilter> = vec![];
            subs.insert(String::from(topic), filters);
        });

        // Store subscription on the network
        let subscriptions_str: String = serde_json::to_string(&self.subscriptions).unwrap();
        self.safeot_comm.set_subscriptions(subscriptions_str.as_str())?;

        Ok(())
    }

    /// Notify of an event associated to an speficic topic.
    /// Eventually this can support multiple topics.
    pub fn notify(&mut self, topic: &str, data: &str) -> ResultReturn<()>
    {
        println!("Event occurred for topic: {}, data: {}", topic, data);
        self.safeot_comm.set_topic_events(topic, data)?;
        Ok(())
    }

    /// Send an action request to a Thing and wait for response
    pub fn action_request(&self, thing_id: &str, action: &str, args: ActionArgs) -> ResultReturn<&str> {
        // Search on the network by thing_id
        //self.safeot_comm.send_action_request(thing_id, action, args).ok_or("Action request failure".to_owned())
        Ok("")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
