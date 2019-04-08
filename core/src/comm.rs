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

extern crate safe_core;

use log::debug;

use errors::{Error, ErrorCode, ResultReturn};

// Functions to access the SAFE Network
use self::safe_core::ffi::arrays::XorNameArray;
use safe_net::{MutableData, SAFENet};
use std::time::{SystemTime, UNIX_EPOCH};

const SAFE_THING_TYPE_TAG: u64 = 27417;

static SAFE_THING_ENTRY_K_STATUS: &'static str = "_safe_thing_status";
static SAFE_THING_ENTRY_V_STATUS_CONNECTED: &'static str = "Connected";
static SAFE_THING_ENTRY_V_STATUS_PUBLISHED: &'static str = "Published";
static SAFE_THING_ENTRY_V_STATUS_DISABLED: &'static str = "Disabled";

static SAFE_THING_ENTRY_K_ATTRS: &'static str = "_safe_thing_attributes";
static SAFE_THING_ENTRY_K_TOPICS: &'static str = "_safe_thing_topics";
static SAFE_THING_ENTRY_K_ACTIONS: &'static str = "_safe_thing_actions";
static SAFE_THING_ENTRY_K_SUBSCRIPTIONS: &'static str = "_safe_thing_subscriptions";
static SAFE_THING_ENTRY_K_EVENTS: &'static str = "_safe_thing_events_";
static SAFE_THING_ENTRY_K_ACTION_REQ: &'static str = "_safe_thing_action_req_";

#[derive(Debug)]
pub enum ThingStatus {
    Unknown,
    Connected,
    Published,
    Disabled,
}

pub struct SAFEthingComm {
    thing_id: String,
    safe_net: SAFENet,
    auth_str: String,
    thing_mdata: MutableData,
    xor_name: XorNameArray,
}

impl SAFEthingComm {
    pub fn clone(&self) -> ResultReturn<SAFEthingComm> {
        let safething_comm = SAFEthingComm {
            thing_id: self.thing_id.clone(),
            /// TODO: pass a callback function for disconnection notif to reconnect
            safe_net: SAFENet::connect(&self.thing_id, &self.auth_str)?, // Connect to the SAFE Network using the auth URI
            auth_str: self.auth_str.clone(),
            thing_mdata: self.thing_mdata.clone(),
            xor_name: self.xor_name.clone(),
        };

        Ok(safething_comm)
    }

    pub fn new(thing_id: &str, auth_uri: &str) -> ResultReturn<SAFEthingComm> {
        let auth_str: String = if auth_uri.is_empty() {
            debug!("Authorising SAFEthing app with safe_auth webservice...");
            SAFENet::gen_auth_request(thing_id)?
        } else {
            debug!("Using the provided authorisation credentials to connect...");
            auth_uri.to_string()
        };

        let safe_thing_comm = SAFEthingComm {
            thing_id: thing_id.to_string(),
            /// TODO: pass a callback function for disconnection notif to reconnect
            safe_net: SAFENet::connect(thing_id, &auth_str)?, // Connect to the SAFE Network using the auth URI
            auth_str,
            thing_mdata: Default::default(),
            xor_name: Default::default(),
        };

        Ok(safe_thing_comm)
    }

    pub fn store_thing_entity(&mut self) -> ResultReturn<(String, u64)> {
        let xor_name = self.safe_net.gen_xor_name(self.thing_id.as_str());
        // FIXME: set permissions to allow others to 'Insert'
        self.thing_mdata = self
            .safe_net
            .new_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)?;
        self.xor_name = xor_name;
        Ok((self.addr_name()?, SAFE_THING_TYPE_TAG))
    }

    pub fn addr_name(&self) -> ResultReturn<String> {
        let mut xor_name = String::new();
        for i in self.xor_name.iter() {
            let x = format!("{:02x}", i);
            xor_name.push_str(x.as_str());
        }
        Ok(xor_name)
    }

    pub fn set_status(&self, status: ThingStatus) -> ResultReturn<()> {
        let status_str;
        // We don't allow status to be set to Unknown
        match status {
            ThingStatus::Connected => status_str = SAFE_THING_ENTRY_V_STATUS_CONNECTED,
            ThingStatus::Published => status_str = SAFE_THING_ENTRY_V_STATUS_PUBLISHED,
            ThingStatus::Disabled => status_str = SAFE_THING_ENTRY_V_STATUS_DISABLED,
            _ => {
                return Err(Error::new(
                    ErrorCode::InvalidArgument,
                    format!("Status param is invalid: {:?}", status).as_str(),
                ));
            }
        }
        self.safe_net.mutable_data_set_value(
            &self.thing_mdata,
            SAFE_THING_ENTRY_K_STATUS,
            status_str,
        )?;
        Ok(())
    }

    pub fn get_status(&self) -> ResultReturn<ThingStatus> {
        let mut status = ThingStatus::Unknown;
        let status_str = self
            .safe_net
            .mutable_data_get_value(&self.thing_mdata, SAFE_THING_ENTRY_K_STATUS)?;
        if status_str == SAFE_THING_ENTRY_V_STATUS_CONNECTED {
            status = ThingStatus::Connected;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_PUBLISHED {
            status = ThingStatus::Published;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_DISABLED {
            status = ThingStatus::Disabled;
        }
        Ok(status)
    }

    pub fn set_attributes(&self, attrs: &str) -> ResultReturn<()> {
        self.safe_net
            .mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_ATTRS, attrs)?;
        Ok(())
    }

    // Private helper
    fn get_mdata(&self, thing_id: &str) -> ResultReturn<MutableData> {
        let xor_name = self.safe_net.gen_xor_name(thing_id);
        self.safe_net
            .get_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)
    }

    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net
            .mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_ATTRS)
    }

    pub fn set_topics(&self, topics: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(
            &self.thing_mdata,
            SAFE_THING_ENTRY_K_TOPICS,
            topics,
        )?;
        Ok(())
    }

    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net
            .mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_TOPICS)
    }

    pub fn set_actions(&self, actions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(
            &self.thing_mdata,
            SAFE_THING_ENTRY_K_ACTIONS,
            actions,
        )?;
        Ok(())
    }

    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net
            .mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_ACTIONS)
    }

    pub fn set_subscriptions(&self, subscriptions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(
            &self.thing_mdata,
            SAFE_THING_ENTRY_K_SUBSCRIPTIONS,
            subscriptions,
        )?;
        Ok(())
    }

    pub fn get_subscriptions(&self) -> ResultReturn<(String)> {
        // FIXME: we are not being able to retrieve the entry with self.thing_mdata
        let thing_mdata = self.get_mdata(&self.thing_id)?;

        match self
            .safe_net
            .mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_SUBSCRIPTIONS)
        {
            Ok(str) => Ok(str),
            Err(_) => Ok(String::from("{}")),
        }
    }

    pub fn set_topic_events(&self, topic: &str, events: &str) -> ResultReturn<()> {
        let topic_entry_key = SAFE_THING_ENTRY_K_EVENTS.to_owned() + topic;
        self.safe_net
            .mutable_data_set_value(&self.thing_mdata, &topic_entry_key, events)?;
        Ok(())
    }

    pub fn get_topic_events(&self, topic: &str) -> ResultReturn<(String)> {
        let topic_entry_key = SAFE_THING_ENTRY_K_EVENTS.to_owned() + topic;
        match self
            .safe_net
            .mutable_data_get_value(&self.thing_mdata, &topic_entry_key)
        {
            Ok(str) => Ok(str),
            Err(_) => Ok(String::from("[]")),
        }
    }

    pub fn get_thing_topic_events(&self, thing_id: &str, topic: &str) -> ResultReturn<(String)> {
        let topic_entry_key = SAFE_THING_ENTRY_K_EVENTS.to_owned() + topic;
        let thing_mdata = self.get_mdata(thing_id)?;
        match self
            .safe_net
            .mutable_data_get_value(&thing_mdata, &topic_entry_key)
        {
            Ok(str) => Ok(str),
            Err(_) => Ok(String::from("[]")),
        }
    }

    pub fn send_action_request(&self, thing_id: &str, action_req: &str) -> ResultReturn<u128> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get time since epoch");
        let request_id: u128 = since_the_epoch.as_nanos();
        let actions_req_key = format!("{}{:?}", SAFE_THING_ENTRY_K_ACTION_REQ, request_id);
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net
            .mutable_data_set_value(&thing_mdata, &actions_req_key, action_req)?;

        Ok(request_id)
    }

    pub fn get_thing_action_request_state(
        &self,
        thing_id: &str,
        request_id: u128,
    ) -> ResultReturn<(String)> {
        let actions_req_key = format!("{}{:?}", SAFE_THING_ENTRY_K_ACTION_REQ, request_id);
        let thing_mdata = self.get_mdata(thing_id)?;
        let action_req: String = match self
            .safe_net
            .mutable_data_get_value(&thing_mdata, &actions_req_key)
        {
            Ok(str) => str,
            Err(_) => String::from("{}"),
        };

        Ok(action_req)
    }

    pub fn get_actions_requests(&self) -> ResultReturn<(Vec<(u128, String)>)> {
        // FIXME: we are not being able to retrieve the entry with self.thing_mdata
        let thing_mdata = self.get_mdata(&self.thing_id)?;

        let actions_reqs = match self.safe_net.mutable_data_get_entries(&thing_mdata) {
            Ok(entries) => {
                entries
                    .iter()
                    .filter_map(|(key, value)| {
                        // let's filter the soft-deleted values and those whic are not action requests
                        if key.starts_with(SAFE_THING_ENTRY_K_ACTION_REQ) && !value.is_empty() {
                            let request_id = key
                                .replace(SAFE_THING_ENTRY_K_ACTION_REQ, "")
                                .parse::<u128>()
                                .unwrap();
                            Some((request_id, value.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            Err(_) => vec![],
        };

        Ok(actions_reqs)
    }

    pub fn set_action_request_state(&self, request_id: u128, new_state: &str) -> ResultReturn<()> {
        let actions_req_key = format!("{}{:?}", SAFE_THING_ENTRY_K_ACTION_REQ, request_id);

        // FIXME: we are not being able to retrieve the entry with self.thing_mdata
        let thing_mdata = self.get_mdata(&self.thing_id)?;

        self.safe_net
            .mutable_data_set_value(&thing_mdata, &actions_req_key, new_state)?;

        Ok(())
    }

    pub fn sim_net_disconnect(&mut self) {
        self.safe_net.sim_net_disconnect();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
