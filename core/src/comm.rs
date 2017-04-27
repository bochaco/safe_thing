extern crate safe_app;
extern crate safe_core;
extern crate rust_sodium;
extern crate rustc_serialize;

use self::rust_sodium::crypto::hash::sha256;
use self::rust_sodium::crypto::hash::sha256::Digest;
use self::rustc_serialize::base64::{CharacterSet, Config, FromBase64, FromBase64Error, Newline, ToBase64};

use std::collections::BTreeMap;

use errors::{ResultReturn, Error, ErrorCode};

// Functions to access the SAFE network
use native::SAFENet;


const SAFE_THING_TYPE_TAG: u64 = 270417;
static SAFE_THING_ENTRY_APP_STATUS: &'static str = "_safe_thing_app_status";

pub type ActionArgs = Vec<String>; // the values are opaque for the framework

pub struct SAFEthingComm {
    thing_id: String,
    safe_net: SAFENet,
    xor_name: String,

    // the following is temporary, we keep this in the safenet
    status: String,
    attrs: String,
    topics: String,
    actions: String,
    subscriptions: String,
    topic_events: BTreeMap<String, String>
}

#[allow(unused_variables)]
impl SAFEthingComm {
    pub fn new(thing_id: &str, auth_uri: &str) -> ResultReturn<SAFEthingComm> {
        let mut safe_thing_comm = SAFEthingComm {
            thing_id: String::from(thing_id),
            safe_net: SAFENet::connect(auth_uri)?, // Connect to the SAFE network using the auth URI
            xor_name: Default::default(),
            // These are attributes of the SAFEthing itself which are just cached here
            status: String::from("Unknown"),

            // the following are temporary
            attrs: String::from("[]"),
            topics: String::from("[]"),
            actions: String::from("[]"),
            subscriptions: String::from("[]"),
            topic_events: BTreeMap::new(),
        };

        Ok(safe_thing_comm)
    }

    pub fn store_thing_entity(&mut self) -> ResultReturn<String> {
        let Digest(sha256) = sha256::hash(self.thing_id.as_bytes());
        let mut xor_name: [u8; 32] = Default::default();
        xor_name.copy_from_slice(sha256.as_ref());

        self.safe_net.new_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG);

        self.xor_name = sha256.as_ref().to_base64(config());
        Ok(self.xor_name.clone())
    }

    pub fn get_own_addr_name(&self) -> ResultReturn<String> {
        Ok(self.xor_name.clone())
    }

    // TODO: read from the network
    pub fn get_thing_addr_name(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.xor_name.clone())
    }

    pub fn set_status(&mut self, status: &str) -> ResultReturn<()> {
        self.status = String::from(status);
        unsafe {
            //mdata_entry_actions_new(self.safe_app, safe_thing_comm_c_void_ptr, mdata_entry_actions_callback);
        };
        Ok(())
    }

    // TODO: read from the network
    pub fn get_thing_status(&self, thing_id: &str) -> ResultReturn<String> {
        unsafe {
            let key = SAFE_THING_ENTRY_APP_STATUS;
            //mdata_get_value(self.safe_app, self.mutable_data_h, key.as_ptr(), key.len(), safe_thing_comm_c_void_ptr, mdata_get_value_callback);
        };
        Ok(self.status.clone())
    }

    // TODO: store it in the network
    pub fn set_attributes(&mut self, attrs: &str) -> ResultReturn<()> {
        self.attrs = String::from(attrs);
        Ok(())
    }

    // TODO: read from the network
    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.attrs.clone())
    }

    // TODO: store it in the network
    pub fn set_topics(&mut self, topics: &str) -> ResultReturn<()> {
        self.topics = String::from(topics);
        Ok(())
    }

    // TODO: read from the network
    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.topics.clone())
    }

    // TODO: store it in the network
    pub fn set_actions(&mut self, actions: &str) -> ResultReturn<()> {
        self.actions = String::from(actions);
        Ok(())
    }

    // TODO: read from the network
    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.actions.clone())
    }

    // TODO: store it in the network
    pub fn set_subscriptions(&mut self, subscriptions: &str) -> ResultReturn<()> {
        self.subscriptions = String::from(subscriptions);
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
