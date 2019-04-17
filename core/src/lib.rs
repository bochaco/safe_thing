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

mod comm;
mod errors;
mod safe_net;
mod safe_net_helpers;

use comm::{SAFEthingComm, ThingStatus};
use errors::{Error, ErrorCode, ResultReturn};
use log::{debug, error, info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
            name: name.to_string(),
            access: access,
        }
    }
}

/// This is the structure which defines the attributes of a SAFEthing
/// SAFEthings can subscribe for notifications upon changes detected on dynamic attributes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThingAttr {
    pub attr: String,
    pub value: String,
    pub is_dynamic: bool,
}

impl ThingAttr {
    pub fn new(attr: &str, value: &str, is_dynamic: bool) -> ThingAttr {
        ThingAttr {
            attr: attr.to_string(),
            value: value.to_string(),
            is_dynamic,
        }
    }
}

/// Actions that can be requested to a SAFEthing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionDef {
    pub name: String,
    pub access: AccessType,
    pub params: Vec<String>,
}

impl ActionDef {
    pub fn new(name: &str, access: AccessType, params: &[&str]) -> ActionDef {
        let mut parameters = vec![];
        parameters.extend(params.iter().map(|&p| p.to_string()));
        ActionDef {
            name: name.to_string(),
            access: access,
            params: parameters,
        }
    }
}

pub type ActionArgs = Vec<String>; // the values are opaque for the framework

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ActionReq {
    pub thing_id: String,
    pub action: String,
    pub args: ActionArgs,
    pub state: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FilterOperator {
    Any,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
}

impl FilterOperator {
    pub fn eval(&self, lvalue: &str, rvalue: &str) -> bool {
        let result = match self {
            FilterOperator::Any => true, // this filter lets all values through
            FilterOperator::Equal => lvalue == rvalue,
            FilterOperator::NotEqual => lvalue != rvalue,
            FilterOperator::LessThan => {
                lvalue.parse::<f64>().unwrap() < rvalue.parse::<f64>().unwrap()
            }
            FilterOperator::GreaterThan => {
                lvalue.parse::<f64>().unwrap() > rvalue.parse::<f64>().unwrap()
            }
        };

        trace!(
            "Evaluation of '{}' '{:?}' '{}' => {}",
            lvalue,
            self,
            rvalue,
            result
        );

        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AttrSubscription {
    attr_name: String,
    filter_op: FilterOperator,
    filter_value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TopicSubscription {
    topic: String,
    filter_op: FilterOperator,
    filter_value: String,
}

/// Each subscription can either be for a specific topic or for a dynamic attribute.
/// We also keep track of the last time we checked the remote thing for any new topic event, or
/// in the case of a attribute subscription we keep a copy of last value we reported in a
/// notification to prevent from sending duplicate notifications.
#[derive(Serialize, Deserialize, Clone, Debug)]
enum Subscription {
    Topic((TopicSubscription, Timestamp)),
    Attr((AttrSubscription, String)),
}

/// Timestamps for events are all kept in nanos elapsed since epoch
type Timestamp = u128;

/// Several subscriptions can be registered for a remote SAFEthing.
type ThingSubscriptions = Vec<Subscription>;

/// We need an structure to keep track of the subscriptions for different SAFEthings.
/// This is a map from a SAFEthing ID to a list of subscriptions.
/// This is just an in memory cache since it's all stored on the network.
type RegisteredSubscriptions = BTreeMap<String, ThingSubscriptions>;

/// Everytime a new event is emitted for any topic the SAFEthing has subscribed to,
/// the framework will invoke the registered callback function.
/// The following arguments are passed to the callback function:
/// thing_id: the SAFEthing id which emitted the event
/// topic: the corresponding topic the event belongs to
/// data: any data provided by the event emitter with the event
/// timestamp: event timestamp as registered by the event emitter
type SubsNotifCallback =
    Fn(&SAFEthing, &str, &str, &str, Timestamp) + std::marker::Send + std::marker::Sync;

/// Every action reqeust is assigned its unique identifier
type ActionReqId = u128;

/// When an action request is received for any of the published supported actions, the framework
/// will invoke a callback function for the SAFEthing can act upon it.
/// The following arguments are passed to the callback function:
/// request_id: an unique identifier for the action request
/// thing_id: identifier of the SAFEthing sending the action request
/// action: the name of the action
/// args: the list of arguments provided for the action
type ActionReqCallback =
    Fn(&SAFEthing, ActionReqId, &str, &str, &[&str]) + std::marker::Send + std::marker::Sync;

#[derive(Clone)]
pub struct SAFEthing {
    pub thing_id: String,
    safe_thing_comm: SAFEthingComm,
    subscriptions: RegisteredSubscriptions,
    subsc_thread_channel_tx: Option<Sender<(String, ThingSubscriptions)>>,
    notifs_cb: &'static SubsNotifCallback,
    action_req_cb: &'static ActionReqCallback,
}

impl SAFEthing {
    /// The thing id shall be an opaque string, and the auth URI shall not contain any
    /// scheme/protocol (i.e. without 'safe-...:' prefix) but just the encoded authorisation
    pub fn new(
        thing_id: &str,
        auth_uri: &str,
        notifs_cb: &'static SubsNotifCallback,
        action_req_cb: &'static ActionReqCallback,
    ) -> ResultReturn<SAFEthing> {
        env_logger::init();
        if thing_id.len() < THING_ID_MIN_LENGTH {
            return Err(Error::new(
                ErrorCode::InvalidArgument,
                format!(
                    "SAFEthing ID must be at least {} bytes long",
                    THING_ID_MIN_LENGTH
                )
                .as_str(),
            ));
        }

        let safe_thing = SAFEthing {
            thing_id: thing_id.to_string(),
            safe_thing_comm: SAFEthingComm::new(thing_id, auth_uri)?,
            subscriptions: RegisteredSubscriptions::default(),
            subsc_thread_channel_tx: None,
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
        let subscriptions_str = self.safe_thing_comm.get_subscriptions()?;
        self.subscriptions = serde_json::from_str(&subscriptions_str).unwrap();

        // Create a channel to notify the subscriptions monitoring thread
        // upon any new subscriptions created by the SAFEthing
        let (tx, rx): (
            Sender<(String, ThingSubscriptions)>,
            Receiver<(String, ThingSubscriptions)>,
        ) = mpsc::channel();
        self.subsc_thread_channel_tx = Some(tx);

        // Spawn thread in charge of checking subscriptions
        // and notifying the SAFEthing by invoking the callback
        // TODO: share self (SAFEthing) among threads instead of cloning
        let notifs_cb: &'static SubsNotifCallback = self.notifs_cb;
        spawn_check_subsc_thread(self.clone(), notifs_cb, self.subscriptions.clone(), rx);

        // Spawn thread in charge of checking for action requests
        // and invoking the corresponding callback function
        // TODO: share self (SAFEthing) among threads instead of cloning
        let action_req_cb: &'static ActionReqCallback = self.action_req_cb;
        spawn_check_new_action_reqs(self.clone(), action_req_cb);

        info!("SAFEthing Connected with ID: {}", self.thing_id);
        Ok(())
    }

    /// Get status of this SAFEthing
    pub fn status(&self) -> ResultReturn<Status> {
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

    /// Set a new value for an attribute, or add it if it didn't exist
    pub fn set_attr_value(&self, attr: &str, value: &str) -> ResultReturn<()> {
        let attrs_str = self.safe_thing_comm.get_thing_attrs(&self.thing_id)?;
        let mut attrs: Vec<ThingAttr> = serde_json::from_str(&attrs_str).unwrap();
        match attrs.iter().position(|ref x| x.attr == attr) {
            Some(i) => {
                let item = &mut attrs[i];
                item.value = value.to_string();
            }
            None => attrs.push(ThingAttr {
                attr: attr.to_string(),
                value: value.to_string(),
                is_dynamic: true,
            }),
        };

        // Update attributes on the network
        let attrs_str: String = serde_json::to_string(&attrs).unwrap();
        self.safe_thing_comm.set_attributes(attrs_str.as_str())?;
        Ok(())
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
    pub fn publish(&self) -> ResultReturn<()> {
        let _ = self.safe_thing_comm.set_status(ThingStatus::Published);
        info!("SAFEthing published with ID: {}", self.thing_id);
        Ok(())
    }

    /// Subscribe to topics published by a SAFEthing (all data is stored in the network to support device resets/reboots)
    pub fn subscribe_to_topic(
        &mut self,
        thing_id: &str,
        topic: &str,
        filter_op: FilterOperator,
        filter_value: &str,
    ) -> ResultReturn<()> {
        // TODO: check if thing is 'Published' before subscribing,
        // and also check if it supports the topic

        let topic_subs = TopicSubscription {
            topic: topic.to_string(),
            filter_op: filter_op.clone(),
            filter_value: filter_value.to_string(),
        };

        self.register_new_subscription(thing_id, Subscription::Topic((topic_subs, gen_timestamp())))
    }

    /// Subscribe to a dynamic attribute published by a SAFEthing in order to receive notifications
    /// upon changes detected on them and based on the filters provided
    pub fn subscribe_to_attr(
        &mut self,
        thing_id: &str,
        attr_name: &str,
        filter_op: FilterOperator,
        filter_value: &str,
    ) -> ResultReturn<()> {
        // TODO: check if thing is 'Published' before subscribing,
        // and also check if the attribute is_dynamic

        let attr_subs = AttrSubscription {
            attr_name: attr_name.to_string(),
            filter_op: filter_op.clone(),
            filter_value: filter_value.to_string(),
        };

        self.register_new_subscription(thing_id, Subscription::Attr((attr_subs, String::from(""))))
    }

    // private helper
    fn register_new_subscription(
        &mut self,
        thing_id: &str,
        subscription: Subscription,
    ) -> ResultReturn<()> {
        // TODO: check if thing is 'Published' before subscribing,
        // and also check if it supports the topic

        // We keep track of the suscriptions list in memory first
        let mut thing_subs = match self.subscriptions.get(thing_id) {
            Some(subs) => subs.clone(),
            None => vec![],
        };
        thing_subs.push(subscription);
        let thing_subs_clone = thing_subs.clone();
        self.subscriptions.insert(thing_id.to_string(), thing_subs);

        // Also notify the thread which is monitoring topics so it starts checking this new one
        if let Some(tx) = &self.subsc_thread_channel_tx {
            match tx.send((thing_id.to_string(), thing_subs_clone)) {
                Ok(()) => trace!("New subscription notified to monitoring thread"),
                Err(err) => error!(
                    "Failed to notify new subscription to monitoring thread: {}",
                    err
                ),
            };
        }

        // But also update subscriptions list on the network
        // TODO: we are not making it persistent just yet
        /*
        let subscriptions_str: String = serde_json::to_string(&self.subscriptions).unwrap();
        self.safe_thing_comm
            .set_subscriptions(subscriptions_str.as_str())?;
        */

        Ok(())
    }

    /// Notify of an event associated to an speficic topic.
    /// Eventually this can support multiple topics.
    pub fn notify(&self, topic: &str, data: &str) -> ResultReturn<()> {
        info!("Notifying event for topic: {}, data: {}", topic, data);
        let events: String = self.safe_thing_comm.get_topic_events(topic)?;
        let mut events_vec: Vec<(Timestamp, String)> = match serde_json::from_str(&events) {
            Ok(vec) => vec,
            Err(_) => vec![],
        };
        let timestamp = gen_timestamp();
        events_vec.push((timestamp, data.to_string()));
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
    ) -> ResultReturn<ActionReqId> {
        let mut args_vec = Vec::new();
        args_vec.extend(args.iter().map(|&arg| arg.to_string()));
        let action_req = ActionReq {
            thing_id: self.thing_id.clone(),
            action: action.to_string(),
            args: args_vec,
            state: ACTION_REQUEST_INIT_STATE.to_string(),
        };
        let action_req_str: String = serde_json::to_string(&action_req).unwrap();

        let req_id = self
            .safe_thing_comm
            .send_action_request(thing_id, action_req_str.as_str())?;

        // TODO: share self (SAFEthing) among threads instead of cloning
        let safething_comm = self.safe_thing_comm.clone();
        spawn_action_req_monitoring_thread(thing_id.to_string(), req_id, safething_comm, cb);

        Ok(req_id)
    }

    /// Update the state of an action reqeust
    pub fn update_action_request_state(
        &self,
        request_id: ActionReqId,
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

// Helper to generate timestamp (in nanos)
fn gen_timestamp() -> Timestamp {
    let now_timestamp = SystemTime::now();
    let since_the_epoch = now_timestamp
        .duration_since(UNIX_EPOCH)
        .expect("Failed to get time since epoch");

    since_the_epoch.as_nanos()
}

// spawn a thread which takes care of monitoring topics which the SAFEthing subcribed to
fn spawn_check_subsc_thread(
    safe_thing: SAFEthing,
    notifs_cb: &'static SubsNotifCallback,
    subs: RegisteredSubscriptions,
    subsc_thread_channel_rx: Receiver<(String, ThingSubscriptions)>,
) {
    thread::spawn(move || {
        let mut subscriptions = subs;
        loop {
            trace!("Checking subscriptions...");
            for (thing_id, thing_subscription) in subsc_thread_channel_rx.try_iter() {
                debug!(
                    "New subscription to monitor: {} - {:?}",
                    thing_id, thing_subscription
                );
                subscriptions.insert(thing_id, thing_subscription);
            }

            for (thing_id, thing_subs) in subscriptions.iter_mut() {
                for subscription in thing_subs.iter_mut() {
                    match subscription {
                        Subscription::Topic((topic_subs, last_report_timestamp)) => {
                            check_topic_subs_and_notify(
                                thing_id,
                                safe_thing.clone(),
                                notifs_cb,
                                topic_subs,
                                last_report_timestamp,
                            );
                        }
                        Subscription::Attr((attr_subs, last_val_reported)) => {
                            check_attrs_subs_and_notify(
                                thing_id,
                                safe_thing.clone(),
                                notifs_cb,
                                attr_subs,
                                last_val_reported,
                            );
                        }
                    }
                }
            }

            trace!("CHECKED SUBSCRIPTIONS....WAIT FOR NEXT LOOP");
            thread::sleep(Duration::from_millis(SUBSCRIPTIONS_CHECK_FREQ));
        }
    });
}

fn check_topic_subs_and_notify(
    thing_id: &String,
    safe_thing: SAFEthing,
    notifs_cb: &'static SubsNotifCallback,
    topic_subs: &mut TopicSubscription,
    last_report_timestamp: &mut Timestamp,
) {
    let TopicSubscription {
        topic,
        filter_op,
        filter_value,
    } = topic_subs;
    trace!(
        "CHECKING TOPIC EVENTS FROM (thingId -> topic): {} -> {}",
        thing_id,
        topic
    );

    let events = safe_thing
        .safe_thing_comm
        .get_thing_topic_events(thing_id, topic)
        .unwrap();
    let events_vec: Vec<(Timestamp, String)> = match serde_json::from_str(&events) {
        Ok(vec) => vec,
        Err(_) => vec![],
    };
    for (event_timestamp, event) in events_vec.iter() {
        let do_eval = event_timestamp > last_report_timestamp;
        if do_eval && filter_op.eval(&event, &filter_value) {
            debug!(
                "Event occurred for topic: {}, event: ({}, {})",
                topic, event_timestamp, event
            );
            (notifs_cb)(
                &safe_thing,
                thing_id.as_str(),
                topic.as_str(),
                event.as_str(),
                *event_timestamp,
            );

            // update last_report_timestamp in the subscriptions list to not
            // keep sending the notification for same (and already notified) event
            *last_report_timestamp = *event_timestamp;
            // TODO: we may want to persist this updates on the network as well
        }
    }
}

fn check_attrs_subs_and_notify(
    thing_id: &String,
    safe_thing: SAFEthing,
    notifs_cb: &'static SubsNotifCallback,
    attr_subs: &mut AttrSubscription,
    last_val_reported: &mut String,
) {
    let AttrSubscription {
        attr_name,
        filter_op,
        filter_value,
    } = attr_subs;

    trace!(
        "CHECKING DYNAMIC ATTRIBUTES CHANGES FROM thingId: {} - {:?}",
        thing_id,
        attr_name
    );

    let attrs = safe_thing
        .safe_thing_comm
        .get_thing_attrs(thing_id)
        .unwrap();
    let attrs_vec: Vec<ThingAttr> = match serde_json::from_str(&attrs) {
        Ok(vec) => vec,
        Err(_) => vec![],
    };
    for ThingAttr {
        attr,
        value,
        is_dynamic,
    } in attrs_vec
    {
        let do_eval = is_dynamic
            && *attr_name == attr
            && (last_val_reported.len() == 0 || !filter_op.eval(&value, last_val_reported));
        if do_eval && filter_op.eval(&value, &filter_value) {
            debug!(
                "Dynamic attribute change occurred: {}, value: {}",
                attr, value
            );
            (notifs_cb)(
                &safe_thing,
                thing_id.as_str(),
                attr.as_str(),
                value.as_str(),
                0,
            );

            // update last_val_reported in the subscriptions list to not
            // keep sending the notification for same (and already notified) event
            *last_val_reported = value;
            // TODO: we may want to persist this updates on the network as well
        }
    }
}

// spawn a thread which takes care of monitoring for new action requests received
fn spawn_check_new_action_reqs(safe_thing: SAFEthing, action_req_cb: &'static ActionReqCallback) {
    thread::spawn(move || {
        loop {
            trace!("Checking for new action requests...");
            let actions_reqs_vec = safe_thing.safe_thing_comm.get_actions_requests().unwrap();
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
                            let action_args: Vec<&str> = args.iter().map(|i| i.as_str()).collect();
                            (action_req_cb)(
                                &safe_thing,
                                *request_id,
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
                            safe_thing
                                .safe_thing_comm
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
    });
}

// spawn a thread to check for a change in the state of an action request sent
fn spawn_action_req_monitoring_thread(
    thing_id: String,
    request_id: ActionReqId,
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
                        debug!("Keep checking sent action request state? {}", keep_checking);
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

        debug!(
            "Ending monitoring thread for sent action request: {}",
            request_id
        );
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
