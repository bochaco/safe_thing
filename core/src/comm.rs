use std::collections::BTreeMap;

use errors::{ResultReturn, Error, ErrorCode};

// Functions to access the SAFE network
use safe_net::{SAFENet, MutableData};


const SAFE_THING_TYPE_TAG: u64 = 270417;

static SAFE_THING_ENTRY_K_STATUS: &'static str = "_safe_thing_status";
static SAFE_THING_ENTRY_V_STATUS_CONNECTED: &'static str = "Connected";
static SAFE_THING_ENTRY_V_STATUS_PUBLISHED: &'static str = "Published";
static SAFE_THING_ENTRY_V_STATUS_DISABLED: &'static str = "Disabled";

static SAFE_THING_ENTRY_K_ATTRS: &'static str = "_safe_thing_attributes";
static SAFE_THING_ENTRY_K_TOPICS: &'static str = "_safe_thing_topics";
static SAFE_THING_ENTRY_K_ACTIONS: &'static str = "_safe_thing_actions";
static SAFE_THING_ENTRY_K_SUBSCRIPTIONS: &'static str = "_safe_thing_subscriptions";

#[derive(Debug)]
pub enum ThingStatus {
    Unknown,
    Connected,
    Published,
    Disabled
}

pub type ActionArgs = Vec<String>; // the values are opaque for the framework

pub struct SAFEthingComm {
    thing_id: String,
    safe_net: SAFENet,
    thing_mdata: MutableData,
    xor_name: [u8; 32],

    // the following is temporary, we keep this in the safenet
    topic_events: BTreeMap<String, String>
}

#[allow(unused_variables)]
impl SAFEthingComm {
    pub fn new(thing_id: &str, auth_uri: &str) -> ResultReturn<SAFEthingComm> {
        let safe_thing_comm = SAFEthingComm {
            thing_id: String::from(thing_id),
            /// TODO: pass a callback function for disconnection notif to reconnect
            safe_net: SAFENet::connect(thing_id, auth_uri)?, // Connect to the SAFE network using the auth URI
            thing_mdata: Default::default(),
            xor_name: Default::default(),

            // the following is temporary
            topic_events: BTreeMap::new(),
        };

        println!("SAFE Network connection status: {}", safe_thing_comm.safe_net.get_conn_status());

        Ok(safe_thing_comm)
    }

    pub fn store_thing_entity(&mut self) -> ResultReturn<String> {
        let xor_name = self.safe_net.gen_xor_name(self.thing_id.as_str());
        self.thing_mdata = self.safe_net.new_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)?;
        self.xor_name = xor_name;
        self.addr_name()
    }

    #[allow(dead_code)]
    pub fn addr_name(&self) -> ResultReturn<String> {
        let mut xor_name = String::new();
        for i in self.xor_name.iter() {
            let x = format!("{:x}", i);
            xor_name.push_str(x.as_str());
        }
        Ok(xor_name)
    }

    pub fn set_status(&mut self, status: ThingStatus) -> ResultReturn<()> {
        let status_str;
        // We don't allow status to be set to Unknown
        match status {
            ThingStatus::Connected => status_str = SAFE_THING_ENTRY_V_STATUS_CONNECTED,
            ThingStatus::Published => status_str = SAFE_THING_ENTRY_V_STATUS_PUBLISHED,
            ThingStatus::Disabled => status_str = SAFE_THING_ENTRY_V_STATUS_DISABLED,
            _ => return Err(Error::new(ErrorCode::InvalidParameters,
                                        format!("Status param is invalid: {:?}", status).as_str()))
        }
        self.safe_net.mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_STATUS, status_str)?;
        Ok(())
    }

    pub fn get_status(&mut self) -> ResultReturn<ThingStatus> {
        let mut status = ThingStatus::Unknown;
        let status_str = self.safe_net.mutable_data_get_value(&self.thing_mdata, SAFE_THING_ENTRY_K_STATUS)?;
        if status_str == SAFE_THING_ENTRY_V_STATUS_CONNECTED {
            status = ThingStatus::Connected;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_PUBLISHED {
            status = ThingStatus::Published;
        } else if status_str == SAFE_THING_ENTRY_V_STATUS_DISABLED {
            status = ThingStatus::Disabled;
        }
        Ok(status)
    }

    pub fn set_attributes(&mut self, attrs: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_ATTRS, attrs)?;
        Ok(())
    }

    // private helper
    fn get_mdata(&self, thing_id: &str) -> ResultReturn<MutableData> {
        let xor_name = self.safe_net.gen_xor_name(thing_id);
        self.safe_net.get_pub_mutable_data(xor_name, SAFE_THING_TYPE_TAG)
    }

    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_ATTRS)
    }

    pub fn set_topics(&mut self, topics: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_TOPICS, topics)?;
        Ok(())
    }

    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_TOPICS)
    }

    pub fn set_actions(&mut self, actions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_ACTIONS, actions)?;
        Ok(())
    }

    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        let thing_mdata = self.get_mdata(thing_id)?;
        self.safe_net.mutable_data_get_value(&thing_mdata, SAFE_THING_ENTRY_K_ACTIONS)
    }

    pub fn set_subscriptions(&mut self, subscriptions: &str) -> ResultReturn<()> {
        self.safe_net.mutable_data_set_value(&self.thing_mdata, SAFE_THING_ENTRY_K_SUBSCRIPTIONS, subscriptions)?;
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
