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

#[macro_use]
extern crate serde_derive;

mod comm;
mod errors;
mod safe_net;
mod safe_net_helpers;

extern crate serde_json;

use std::collections::BTreeMap;
use comm::{SAFEthingComm, ThingStatus, ActionArgs};
use errors::{ResultReturn, Error, ErrorCode};
use std::fmt;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

const THING_ID_MIN_LENGTH: usize = 5;
const SUBSCRIPTIONS_CHECK_FREQ: u64 = 5;

/// Group of SAFEthings that are allow to register to a topic
/// Thing: access only to the thing's application. This is the default and lowest level of access type.
/// Owner: access also is allowed to an individual, application or system that is the actual owner of the SAFEthing, plus the SAFEthing itself.
/// Group: access to a group of individuals or SAFEthings, plus the Owner and the SAFEthing itself.
/// All: access is allowed to anyone or anything, including the SAFEthing itself.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AccessType {
    Thing,
    Owner,
    Group,
    All
}

/// Status of the SAFEthing in the network
/// NonConnected: the SAFEthing is not even Connected in the network, only its ID is known in the framework
/// Connected: the SAFEthing was Connected but it's not published yet, which means it's not operative yet for subscribers
/// Published: the SAFEthing was plublished and it's operative, allowing SAFEthings to subscribe an interact with it
/// Disabled: the SAFEthing was disabled and it's not operative, even that it's information may still be visible
pub enum Status {
    Unknown,
    NonConnected,
    Connected,
    Published,
    Disabled
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            Status::Unknown => "Unknown",
            Status::NonConnected => "NonConnected",
            Status::Connected => "Connected",
            Status::Published => "Published",
            Status::Disabled => "Disabled"
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

/// This is the structure which defines the attributes of a SAFEthing
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

/// Actions that can be requested to a SAFEthing
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

/// A subscription is a map from Topic to a list of filters.
/// This is just a cache since it's all stored in the network.
type Subscription = BTreeMap<String, Vec<EventFilter>>;

pub struct SAFEthing<F: 'static>
    where F: Fn(&str, &str, &str)
{
    pub thing_id: String,
    safe_thing_comm: SAFEthingComm,
    subscriptions: BTreeMap<String, Subscription>,
    notifs_cb: F
}

impl<F: 'static> SAFEthing<F>
    where F: Fn(&str, &str, &str) + std::marker::Send + std::marker::Sync
{
    /// this should go probably as a helper function
    #[allow(dead_code)]
    pub fn check_subscriptions(&self) -> ResultReturn<()>
    {
        for (thing_id, subs) in self.subscriptions.iter() {
            for (topic, _filter) in subs.iter() {
                let events = self.safe_thing_comm.get_thing_topic_events(thing_id, topic)?;
                let mut events_vec: Vec<String> = match serde_json::from_str(&events) {
                    Ok(vec) => vec,
                    Err(_) => vec![]
                };
                for event in events_vec.iter() {
                    println!("Event occurred for topic: {}, event: {}", topic, event);
                    (self.notifs_cb)(thing_id.as_str(), topic.as_str(), event.as_str());
                }
            }
        }
        Ok(())
    }

    /// The thing id shall be an opaque string, and the auth URI shall not contain any
    /// scheme/protocol (i.e. without 'safe-...:' prefix) but just the encoded authorisation
    pub fn new(thing_id: &str, auth_uri: &str,
                notifs_cb: F) -> ResultReturn<SAFEthing<F>> {
        if thing_id.len() < THING_ID_MIN_LENGTH {
            return Err(Error::new(ErrorCode::InvalidParameters,
                format!("SAFEthing ID must be at least {} bytes long", THING_ID_MIN_LENGTH).as_str()));
        }

        let safe_thing = SAFEthing {
            thing_id: thing_id.to_string(),
            safe_thing_comm: SAFEthingComm::new(thing_id, auth_uri)?,
            subscriptions: BTreeMap::new(),
            notifs_cb: notifs_cb
        };
        println!("SAFEthing instance created with ID: {}", thing_id);

        // TODO: we should read the subscriptions from the network as this could have been
        // a device which was restarted. notifs_cb will be used for notifications
        let (tx, rx) = mpsc::channel();
        let _thread = thread::spawn(move || {

            /*for (thing_id, subs) in safe_thing.subscriptions.iter() {
                for (topic, _filter) in subs.iter() {
                    println!("TOPIC and THING_ID: {} {}", thing_id, topic);
                }
            }*/
            let safe_thing_comm = SAFEthingComm::new("thing_id", "auth_uri").unwrap();
            let _events = safe_thing_comm.get_thing_topic_events("thing_id", "topic");
            let vals = vec![
                String::from("hi 1"),
                String::from("from 2"),
                String::from("the 3"),
                String::from("thread 4"),
            ];

            for val in vals {
                tx.send(val).unwrap();
                thread::sleep(Duration::from_secs(SUBSCRIPTIONS_CHECK_FREQ));
            }

            /*loop {
                println!("Checking events...");
                thread::sleep(Duration::from_secs(SUBSCRIPTIONS_CHECK_FREQ));
                let val = String::from("hi from thread");
                tx.send(val).unwrap();
            }*/
        });

        thread::spawn(move || {
            for received in rx {
                println!("Got: {}", received);
            }
        });

        Ok(safe_thing)
    }

    /// Register and re-register a SAFEthing specifying its attributes,
    /// events/topics and available actions
    #[allow(unused_variables)]
    pub fn register(&mut self, attrs: &Vec<ThingAttr>,
                    topics: &Vec<Topic>, actions: &Vec<ActionDef>) -> ResultReturn<()> {
        // Register it in the network
        let thing_xorname: String = self.safe_thing_comm.store_thing_entity()?;
        println!("SAFEthing entity XoRname: {}", thing_xorname);

        // Populate entity with attributes
        let attrs: String = serde_json::to_string(&attrs).unwrap();
        self.safe_thing_comm.set_attributes(attrs.as_str())?;

        // Populate entity with topics
        let topics: String = serde_json::to_string(&topics).unwrap();
        self.safe_thing_comm.set_topics(topics.as_str())?;

        // Populate entity with actions
        let actions: String = serde_json::to_string(&actions).unwrap();
        self.safe_thing_comm.set_actions(actions.as_str())?;

        // Set SAFEthing status as Connected
        self.safe_thing_comm.set_status(ThingStatus::Connected)?;

        println!("SAFEthing Connected with ID: {}", self.thing_id);
        Ok(())
    }

    /// Get status of this SAFEthing
    pub fn status(&mut self) -> ResultReturn<Status> {
        match self.safe_thing_comm.get_status() {
            Ok(ThingStatus::Unknown) => Ok(Status::Unknown),
            Ok(ThingStatus::Connected) => Ok(Status::Connected),
            Ok(ThingStatus::Published) => Ok(Status::Published),
            Ok(ThingStatus::Disabled) => Ok(Status::Disabled),
            Err(err) => return Err(err)
        }
    }

    /// Get list of attrbiutes of a SAFEthing
    /// Search on the network by thing_id
    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<Vec<ThingAttr>> {
        let attrs_str = self.safe_thing_comm.get_thing_attrs(thing_id)?;
        let attrs: Vec<ThingAttr> = serde_json::from_str(&attrs_str).unwrap();
        Ok(attrs)
    }

    /// Get list of topics supported by a SAFEthing
    /// Search on the network by thing_id
    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<Vec<Topic>> {
        let topics_str = self.safe_thing_comm.get_thing_topics(thing_id)?;
        let topics: Vec<Topic> = serde_json::from_str(&topics_str).unwrap();
        Ok(topics)
    }

    /// Get list of actions supported by a SAFEthing
    /// Search on the network by thing_id
    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<Vec<ActionDef>> {
        let actions_str = self.safe_thing_comm.get_thing_actions(thing_id)?;
        let actions: Vec<ActionDef> = serde_json::from_str(&actions_str).unwrap();
        Ok(actions)
    }

    /// Publish the thing making it available and operative in the network, allowing other SAFEthings
    /// to request actions, subscribe to topics, and receive notifications upon events.
    pub fn publish(&mut self) -> ResultReturn<()> {
        let _ = self.safe_thing_comm.set_status(ThingStatus::Published);
        println!("SAFEthing published with ID: {}", self.thing_id);
        Ok(())
    }

    /// Subscribe to topics published by a SAFEthing (all data is stored in the network to support device resets/reboots)
    /// Eventually this can support filters
    pub fn subscribe(&mut self, thing_id: &str, topic: &str/*, filter*/) -> ResultReturn<()>
    {
        // TODO: check if thing is 'Published' before subscribing, and
        // also check if it supports the topic

        // We keep track of the suscriptions list in memory first
        self.subscriptions.entry(String::from(thing_id)).or_insert(BTreeMap::new());
        let thing = String::from(thing_id);
        self.subscriptions.get_mut(&thing).map(|subs| {
            let filters: Vec<EventFilter> = vec![];
            subs.insert(String::from(topic), filters);
        });

        // But also store subscriptions list on the network
        let subscriptions_str: String = serde_json::to_string(&self.subscriptions).unwrap();
        self.safe_thing_comm.set_subscriptions(subscriptions_str.as_str())?;

        Ok(())
    }

    /// Notify of an event associated to an speficic topic.
    /// Eventually this can support multiple topics.
    pub fn notify(&mut self, topic: &str, data: &str) -> ResultReturn<()>
    {
        println!("Event occurred for topic: {}, data: {}", topic, data);
        let events: String = self.safe_thing_comm.get_topic_events(topic)?;
        let mut events_vec: Vec<String> = match serde_json::from_str(&events) {
            Ok(vec) => vec,
            Err(_) => vec![]
        };
        events_vec.push(data.to_string());
        let events_str: String = serde_json::to_string(&events_vec).unwrap();
        self.safe_thing_comm.set_topic_events(topic, events_str.as_str())?;
        Ok(())
    }

    /// Send an action request to a SAFEthing and wait for the response
    /// Search on the network by thing_id
    pub fn action_request(&self, _thing_id: &str, _action: &str, _args: ActionArgs) -> ResultReturn<&str> {
        //self.safe_thing_comm.send_action_request(thing_id, action, args).ok_or("Action request failure".to_owned())
        Ok("")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
