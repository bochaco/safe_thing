extern crate rust_sodium;
extern crate rustc_serialize;

use self::rust_sodium::crypto::hash::sha256;
use self::rust_sodium::crypto::hash::sha256::Digest;
use self::rustc_serialize::base64::{CharacterSet, Config, Newline, ToBase64};

use std::collections::BTreeMap;

use errors::{ResultReturn, Error, ErrorCode};

// Functions to access the SAFE network
use safe_net::{SAFENet, MutableData};


const SAFE_THING_TYPE_TAG: u64 = 270417;

static SAFE_THING_ENTRY_K_STATUS: &'static str = "_safe_thing_status";
static SAFE_THING_ENTRY_V_STATUS_REGISTERED: &'static str = "Registered";
static SAFE_THING_ENTRY_V_STATUS_PUBLISHED: &'static str = "Published";
static SAFE_THING_ENTRY_V_STATUS_DISABLED: &'static str = "Disabled";

static SAFE_THING_ENTRY_K_ATTRS: &'static str = "_safe_thing_attributes";
static SAFE_THING_ENTRY_K_TOPICS: &'static str = "_safe_thing_topics";
static SAFE_THING_ENTRY_K_ACTIONS: &'static str = "_safe_thing_actions";
static SAFE_THING_ENTRY_K_SUBSCRIPTIONS: &'static str = "_safe_thing_subscriptions";

#[derive(Debug)]
pub enum ThingStatus {
    Unknown,
    Registered,
    Published,
    Disabled
}

pub type ActionArgs = Vec<String>; // the values are opaque for the framework

pub struct SAFEthingComm {
    thing_id: String,
    safe_net: SAFENet,
    thing_mdata: MutableData,
    xor_name: String,

    // the following is temporary, we keep this in the safenet
    topic_events: BTreeMap<String, String>
}

#[allow(unused_variables)]
impl SAFEthingComm {
    pub fn new(thing_id: &str, auth_uri: &str) -> ResultReturn<SAFEthingComm> {
        let safe_thing_comm = SAFEthingComm {
            thing_id: String::from(thing_id),
            safe_net: SAFENet::connect(auth_uri)?, // Connect to the SAFE network using the auth URI
            thing_mdata: Default::default(),
            xor_name: Default::default(),

            // the following are temporary
            topic_events: BTreeMap::new(),
        };

        Ok(safe_thing_comm)
    }

    pub fn store_thing_entity(&mut self) -> ResultReturn<String> {
        let Digest(sha256) = sha256::hash(self.thing_id.as_bytes());
        let mut xor_name: [u8; 32] = Default::default();
        xor_name.copy_from_slice(sha256.as_ref());

        self.thing_mdata = self.safe_net.new_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)?;

        self.xor_name = sha256.as_ref().to_base64(config());
        Ok(self.xor_name.clone())
    }

    #[allow(dead_code)]
    pub fn addr_name(&self) -> ResultReturn<String> {
        Ok(self.xor_name.clone())
    }

    pub fn set_status(&mut self, status: ThingStatus) -> ResultReturn<()> {
        let status_str;
        match status {
            ThingStatus::Registered => status_str = SAFE_THING_ENTRY_V_STATUS_REGISTERED,
            ThingStatus::Published => status_str = SAFE_THING_ENTRY_V_STATUS_PUBLISHED,
            ThingStatus::Disabled => status_str = SAFE_THING_ENTRY_V_STATUS_DISABLED,
            _ => return Err(Error::new(ErrorCode::InvalidParameters,
                                                format!("Status param is invalid: {:?}", status).as_str()))
        }
        self.safe_net.mutable_data_set_value(self.thing_mdata, SAFE_THING_ENTRY_K_STATUS, status_str)?;
        Ok(())
    }

    pub fn get_status(&mut self) -> ResultReturn<ThingStatus> {
        let mut status = ThingStatus::Unknown;
        let status_str = self.safe_net.mutable_data_get_value(self.thing_mdata, SAFE_THING_ENTRY_K_STATUS)?;
        if status_str == SAFE_THING_ENTRY_V_STATUS_REGISTERED {
            status = ThingStatus::Registered;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_PUBLISHED {
            status = ThingStatus::Published;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_DISABLED {
            status = ThingStatus::Disabled;
        }
        Ok(status)
    }

    pub fn set_attributes(&mut self, attrs: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(self.thing_mdata, SAFE_THING_ENTRY_K_ATTRS, attrs)?;
        Ok(())
    }

    // private helper
    fn get_mdata(&self, thing_id: &str) -> ResultReturn<MutableData> {
        let Digest(sha256) = sha256::hash(thing_id.as_bytes());
        let mut xor_name: [u8; 32] = Default::default();
        xor_name.copy_from_slice(sha256.as_ref());
        self.safe_net.get_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)
    }

    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(thing_mdata, SAFE_THING_ENTRY_K_ATTRS)
    }

    pub fn set_topics(&mut self, topics: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(self.thing_mdata, SAFE_THING_ENTRY_K_TOPICS, topics)?;
        Ok(())
    }

    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(thing_mdata, SAFE_THING_ENTRY_K_TOPICS)
    }

    pub fn set_actions(&mut self, actions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(self.thing_mdata, SAFE_THING_ENTRY_K_ACTIONS, actions)?;
        Ok(())
    }

    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(thing_mdata, SAFE_THING_ENTRY_K_ACTIONS)
    }

    pub fn set_subscriptions(&mut self, subscriptions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(self.thing_mdata, SAFE_THING_ENTRY_K_SUBSCRIPTIONS, subscriptions)?;
        Ok(())
    }

    // TODO: store it in the network
    pub fn set_topic_events(&mut self, topic: &str, events: &str) -> ResultReturn<()> {
        self.topic_events.insert(String::from(topic), String::from(events));
        Ok(())
    }

    // TODO: read from the network
    #[allow(dead_code)]
    pub fn get_topic_events(&mut self, topic: &str) -> ResultReturn<(String)> {
        let events = self.topic_events.get(&String::from(topic)).unwrap();
        Ok(events.clone())
    }

    // TODO: store it in the network
    #[allow(dead_code)]
    pub fn send_action_request(&self, thing_id: &str, action: &str, args: ActionArgs) -> ResultReturn<String> {
        //self.events.push((String::from(topic), String::from(data)));
        Ok(String::from("response"))
    }
}

#[inline]
fn config() -> Config {
    Config {
        char_set: CharacterSet::UrlSafe,
        newline: Newline::LF,
        pad: true,
        line_length: None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
