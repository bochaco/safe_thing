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
extern crate serde_json;

extern crate env_logger;
extern crate log;

use log::{debug, error, info, trace, warn};

mod comm;
mod errors;
mod safe_net;
mod safe_net_helpers;

use comm::{SAFEthingComm, ThingStatus};
use errors::{Error, ErrorCode, ResultReturn};
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};
use std::{fmt, thread};

const THING_ID_MIN_LENGTH: usize = 5;
const SUBSCRIPTIONS_CHECK_FREQ: u64 = 5_000;
const ACTION_REQUEST_CHECK_FREQ: u64 = 4_000;
const ACTION_REQUEST_INIT_STATE: &str = "Requested";
const ACTION_REQUEST_DONE_STATE: &str = "Done";
const ACTION_REQUEST_MONITORING_FREQ: u64 = 2_000;
const ACTION_REQUEST_MONITORING_TIMEOUT: u64 = 60_000;

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
    All,
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
    Disabled,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Status::Unknown => "Unknown",
                Status::NonConnected => "NonConnected",
                Status::Connected => "Connected",
                Status::Published => "Published",
                Status::Disabled => "Disabled",
            }
        )
    }
}

/// Topic name and access type
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub access: AccessType,
}

impl Topic {
    pub fn new(name: &str, access: AccessType) -> Topic {
        Topic {
            name: String::from(name),
            access: access,
        }
    }
}

/// This is the structure which defines the attributes of a SAFEthing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThingAttr {
    pub attr: String,
    pub value: String,
}

impl ThingAttr {
    pub fn new(attr: &str, value: &str) -> ThingAttr {
        ThingAttr {
            attr: String::from(attr),
            value: String::from(value),
        }
    }
}

/// Actions that can be requested to a SAFEthing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub access: AccessType,
    pub args: ActionArgs,
}

// TODO: change to Vec<&'static str>
pub type ActionArgs = Vec<String>; // the values are opaque for the framework

impl ActionDef {
    pub fn new(name: &str, access: AccessType, args: &[&str]) -> ActionDef {
        let mut arguments = vec![];
        for arg in args {
            arguments.push(String::from(*arg));
        }
        ActionDef {
            name: String::from(name),
            access: access,
            args: arguments,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ActionReq {
    pub thing_id: String,
    pub action: String,
    pub args: ActionArgs,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum FilterOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventFilter {
    arg_name: String,
    arg_op: FilterOperator,
    arg_value: String,
}

/// A subscription is a map from Topic to a list of filters.
/// This is just a cache since it's all stored in the network.
type Subscription = BTreeMap<String, Vec<EventFilter>>;

/// We need an structure to keep track of the subscriptions for different SAFEthings
type ThingsSubscriptions = BTreeMap<String, Subscription>;

pub struct SAFEthing<F: 'static, G: 'static>
where
    F: Fn(&str, &str, &str),
    G: Fn(u128, &str, &str, &str, &[&str]),
{
    pub thing_id: String,
    auth_uri: String,
    safe_thing_comm: SAFEthingComm,
    subscriptions: ThingsSubscriptions,
    notifs_cb: &'static F,
    action_req_cb: &'static G,
}

impl<F, G> SAFEthing<F, G>
where
    F: Fn(&str, &str, &str) + std::marker::Send + std::marker::Sync,
    G: Fn(u128, &str, &str, &str, &[&str]) + std::marker::Send + std::marker::Sync,
{
    /// The thing id shall be an opaque string, and the auth URI shall not contain any
    /// scheme/protocol (i.e. without 'safe-...:' prefix) but just the encoded authorisation
    pub fn new(
        thing_id: &str,
        auth_uri: &str,
        notifs_cb: &'static F,
        action_req_cb: &'static G,
    ) -> ResultReturn<SAFEthing<F, G>> {
        env_logger::init();
        if thing_id.len() < THING_ID_MIN_LENGTH {
            return Err(Error::new(
                ErrorCode::InvalidParameters,
                format!(
                    "SAFEthing ID must be at least {} bytes long",
                    THING_ID_MIN_LENGTH
                )
                .as_str(),
            ));
        }

        let safe_thing = SAFEthing {
            thing_id: thing_id.to_string(),
            auth_uri: auth_uri.to_string(),
            safe_thing_comm: SAFEthingComm::new(thing_id, auth_uri)?,
            subscriptions: ThingsSubscriptions::default(),
            notifs_cb: notifs_cb,
            action_req_cb: action_req_cb,
        };

        info!("SAFEthing instance created with ID: {}", thing_id);
        Ok(safe_thing)
    }

    /// Register and re-register a SAFEthing specifying its attributes,
    /// events/topics and available actions
    pub fn register(
        &mut self,
        attrs: &[ThingAttr],
        topics: &[Topic],
        actions: &[ActionDef],
    ) -> ResultReturn<()> {
        // Register it on the network
        let (thing_xorname, thing_typetag) = self.safe_thing_comm.store_thing_entity()?;
        debug!(
            "SAFEthing entity XoRname: {}:{}",
            thing_xorname, thing_typetag
        );

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

        // We read the subscriptions from the network as this could have been a device
        // which was restarted and we need to catch up with any pending notifs.
        // `notifs_cb` will be used for notifications
        let subscriptions_str = self.safe_thing_comm.get_subscriptions()?;
        self.subscriptions = serde_json::from_str(&subscriptions_str).unwrap();

        // TODO: create channel to notify to the subscriptions manager thread
        // upon any new subscriptions created by the thing:
        // let (tx, rx): (Sender<&str>, Receiver<&str>) = mpsc::channel();

        let mut threads = vec![];
        // Spawn thread in charge of checking subscriptions
        // and sending the notifications
        let subscriptions = self.subscriptions.clone();
        let notifs_cb: &'static F = self.notifs_cb;
        // TODO: share self.safething_comm among threads?
        let safething_comm = SAFEthingComm::new(&self.thing_id, &self.auth_uri)?;
        threads.push(thread::spawn(move || {
            loop {
                trace!("Checking subscriptions...");
                for (thing_id, subs) in subscriptions.iter() {
                    for (topic, _filter) in subs.iter() {
                        trace!(
                            "CHECKING EVENTS FROM (thingId -> topic): {} -> {}",
                            thing_id,
                            topic
                        );

                        let events = safething_comm
                            .get_thing_topic_events(thing_id, topic)
                            .unwrap();
                        let events_vec: Vec<String> = match serde_json::from_str(&events) {
                            Ok(vec) => vec,
                            Err(_) => vec![],
                        };
                        for event in events_vec.iter() {
                            debug!("Event occurred for topic: {}, event: {}", topic, event);
                            (notifs_cb)(thing_id.as_str(), topic.as_str(), event.as_str());
                            // TODO: keep track of the events that were already notified
                        }
                    }
                }

                trace!("CHECKED SUBSCRIPTIONS....WAIT FOR NEXT LOOP");
                thread::sleep(Duration::from_millis(SUBSCRIPTIONS_CHECK_FREQ));
            }
        }));

        // Spawn thread in charge of checking for action requests
        // and invoking the corresponding function
        let action_req_cb: &'static G = self.action_req_cb;
        // TODO: share self.safething_comm among threads?
        let safething_comm = SAFEthingComm::new(&self.thing_id, &self.auth_uri)?;
        threads.push(thread::spawn(move || {
            loop {
                trace!("Checking for new action requests...");
                let actions_reqs_vec = safething_comm.get_actions_requests().unwrap();
                trace!("Actions requested to process: {:?}", actions_reqs_vec);
                for (request_id, action_req_str) in actions_reqs_vec.iter() {
                    match serde_json::from_str(&action_req_str) {
                        Ok(ActionReq {
                            thing_id,
                            action,
                            args,
                            state,
                        }) => {
                            if state == ACTION_REQUEST_INIT_STATE {
                                debug!("Action requested: {:?}", action);
                                let args2 = args.clone();
                                let action_args: Vec<&str> =
                                    args.iter().map(|i| i.as_str()).collect();
                                (action_req_cb)(
                                    *request_id,
                                    state.as_str(),
                                    thing_id.as_str(),
                                    action.as_str(),
                                    &action_args,
                                );
                                debug!(
                                    "Action request handled by SAFEthing. Updating new state to {}",
                                    ACTION_REQUEST_DONE_STATE
                                );
                                let action_req = ActionReq {
                                    thing_id,
                                    action,
                                    args: args2,
                                    state: ACTION_REQUEST_DONE_STATE.to_string(),
                                };
                                let action_req_str: String =
                                    serde_json::to_string(&action_req).unwrap();
                                safething_comm
                                    .set_action_request_state(*request_id, action_req_str.as_str())
                                    .expect("Failed to update action request state");
                            }
                        }
                        Err(err) => error!("Action request is invalid, thus ignoring it: {}", err),
                    };

                    // TODO: keep track of the actions requests that were already notified
                }
                trace!("CHECKED ACTIONS....WAIT FOR NEXT LOOP");
                thread::sleep(Duration::from_millis(ACTION_REQUEST_CHECK_FREQ));
            }
        }));

        info!("SAFEthing Connected with ID: {}", self.thing_id);
        Ok(())
    }

    /// Get status of this SAFEthing
    pub fn status(&mut self) -> ResultReturn<Status> {
        match self.safe_thing_comm.get_status() {
            Ok(ThingStatus::Unknown) => Ok(Status::Unknown),
            Ok(ThingStatus::Connected) => Ok(Status::Connected),
            Ok(ThingStatus::Published) => Ok(Status::Published),
            Ok(ThingStatus::Disabled) => Ok(Status::Disabled),
            Err(err) => return Err(err),
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
        info!("SAFEthing published with ID: {}", self.thing_id);
        Ok(())
    }

    /// Subscribe to topics published by a SAFEthing (all data is stored in the network to support device resets/reboots)
    /// Eventually this can support filters
    pub fn subscribe(&mut self, thing_id: &str, topic: &str /*, filter*/) -> ResultReturn<()> {
        // TODO: check if thing is 'Published' before subscribing,
        // and also check if it supports the topic

        // We keep track of the suscriptions list in memory first
        self.subscriptions
            .entry(String::from(thing_id))
            .or_insert(BTreeMap::new());
        let thing = String::from(thing_id);
        self.subscriptions.get_mut(&thing).map(|subs| {
            let filters: Vec<EventFilter> = vec![];
            subs.insert(String::from(topic), filters);
        });

        // But also store subscriptions list on the network
        let subscriptions_str: String = serde_json::to_string(&self.subscriptions).unwrap();
        self.safe_thing_comm
            .set_subscriptions(subscriptions_str.as_str())?;

        Ok(())
    }

    /// Notify of an event associated to an speficic topic.
    /// Eventually this can support multiple topics.
    pub fn notify(&mut self, topic: &str, data: &str) -> ResultReturn<()> {
        info!("Notifying event for topic: {}, data: {}", topic, data);
        let events: String = self.safe_thing_comm.get_topic_events(topic)?;
        let mut events_vec: Vec<String> = match serde_json::from_str(&events) {
            Ok(vec) => vec,
            Err(_) => vec![],
        };
        events_vec.push(data.to_string());
        let events_str: String = serde_json::to_string(&events_vec).unwrap();
        self.safe_thing_comm
            .set_topic_events(topic, events_str.as_str())?;
        Ok(())
    }

    /// Send an action request to a SAFEthing and monitor its state
    /// Search on the network by thing_id
    pub fn action_request(
        &self,
        thing_id: &str,
        action: &str,
        args: &[&str],
        cb: &'static (Fn(&str) -> bool + std::marker::Send + std::marker::Sync),
    ) -> ResultReturn<u128> {
        let mut args_vec = Vec::new();
        args_vec.extend(args.iter().map(|&i| i.to_string()));
        let action_req = ActionReq {
            thing_id: thing_id.to_string(),
            action: action.to_string(),
            args: args_vec,
            state: ACTION_REQUEST_INIT_STATE.to_string(),
        };
        let action_req_str: String = serde_json::to_string(&action_req).unwrap();

        let req_id = self
            .safe_thing_comm
            .send_action_request(thing_id, action_req_str.as_str())?;

        // TODO: share self.safething_comm among threads?
        let safething_comm = SAFEthingComm::new(&self.thing_id, &self.auth_uri)?;
        spawn_monitoring_thread(thing_id.to_string(), req_id, safething_comm, cb);

        Ok(req_id)
    }

    /// Update the state of an action reqeust
    pub fn update_action_request_state(
        &self,
        request_id: u128,
        new_state: &str,
    ) -> ResultReturn<()> {
        self.safe_thing_comm
            .set_action_request_state(request_id, new_state)?;
        Ok(())
    }

    /// Only for testing, to simulate a network disconnection event
    pub fn simulate_net_disconnect(&mut self) {
        self.safe_thing_comm.sim_net_disconnect();
    }
}

fn spawn_monitoring_thread(
    thing_id: String,
    request_id: u128,
    safething_comm: SAFEthingComm,
    cb: &'static (Fn(&str) -> bool + std::marker::Send + std::marker::Sync),
) {
    let mut current_state = ACTION_REQUEST_INIT_STATE.to_string();
    let mut keep_checking = true;
    let start_timestamp = SystemTime::now();
    let mut timeout = false;

    thread::spawn(move || {
        while keep_checking && current_state != ACTION_REQUEST_DONE_STATE && !timeout {
            trace!("Checking action request state...");
            let action_req_str = safething_comm
                .get_thing_action_request_state(&thing_id, request_id)
                .unwrap();
            match serde_json::from_str(&action_req_str) {
                Ok(ActionReq {
                    thing_id: _,
                    action: _,
                    args: _,
                    state,
                }) => {
                    trace!(
                        "Action request new state obtained, request id: {}, new state: {}",
                        request_id,
                        state
                    );
                    if state != current_state {
                        current_state = state.clone();
                        debug!(
                        "Callback to notify action request new state, request id: {}, new state: {}",
                        request_id, state);
                        keep_checking = (cb)(state.as_str());
                        debug!("Keep checking action request state? {}", keep_checking);
                    }
                }
                Err(_) => {
                    error!(
                        "Actin request ({}) current state couldn't be read",
                        request_id
                    );
                }
            };

            trace!("CHECKED ACTION REQUEST STATE....WAIT FOR NEXT LOOP");
            thread::sleep(Duration::from_millis(ACTION_REQUEST_MONITORING_FREQ));
            timeout = match start_timestamp.elapsed() {
                Ok(elapsed) => elapsed > Duration::from_millis(ACTION_REQUEST_MONITORING_TIMEOUT),
                Err(_) => false,
            };

            if timeout {
                warn!(
                    "Monitoring thread for action request {} timed out",
                    request_id
                );
            }
        }

        debug!("Ending monitoring thread for request: {}", request_id);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
