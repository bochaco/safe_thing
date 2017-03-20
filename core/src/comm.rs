use errors::{ResultReturn, Error, ErrorCode};
use std::collections::BTreeMap;

pub type ActionArgs = Vec<String>; // the values are opaque for the framework

pub struct SAFEoTComm {
    thing_id: String,

    // the following is temporary, we keep this in the safenet
    status: String,
    addr_name: String,
    attrs: String,
    topics: String,
    actions: String,
    subscriptions: String,
    topic_events: BTreeMap<String, String>
}

#[allow(unused_variables)]
impl SAFEoTComm {
    pub fn new(thing_id: &str) -> ResultReturn<SAFEoTComm> {
        let safeot_comm = SAFEoTComm {
            thing_id: String::from(thing_id),
            status: String::from("Unknown"),
            addr_name: String::from("x47dhfh376gd7xnxhcohth3uicuiqhco4iuhc34uio"),
            attrs: String::from("[]"),
            topics: String::from("[]"),
            actions: String::from("[]"),
            subscriptions: String::from("[]"),
            topic_events: BTreeMap::new(),
        };

        Ok(safeot_comm)
    }

    pub fn get_thing_status(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.status.clone())
    }

    pub fn get_thing_addr_name(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.addr_name.clone())
    }

    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.attrs.clone())
    }

    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.topics.clone())
    }

    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.actions.clone())
    }

    pub fn store_thing_entity(&mut self, type_tag: u64) -> ResultReturn<String> {
        //let Digest(digest) = sha256::hash(id.as_bytes());
        let mut id = self.thing_id.clone();
        id.push_str("0101010");
        Ok(id)
    }

    pub fn set_status(&mut self, status: &str) -> ResultReturn<()> {
        self.status = String::from(status);
        Ok(())
    }

    pub fn set_attributes(&mut self, attrs: &str) -> ResultReturn<()> {
        self.attrs = String::from(attrs);
        Ok(())
    }

    pub fn set_topics(&mut self, topics: &str) -> ResultReturn<()> {
        self.topics = String::from(topics);
        Ok(())
    }

    pub fn set_actions(&mut self, actions: &str) -> ResultReturn<()> {
        self.actions = String::from(actions);
        Ok(())
    }

    pub fn set_subscriptions(&mut self, subscriptions: &str) -> ResultReturn<()> {
        self.subscriptions = String::from(subscriptions);
        Ok(())
    }

    pub fn set_topic_events(&mut self, topic: &str, events: &str) -> ResultReturn<()> {
        self.topic_events.insert(String::from(topic), String::from(events));
        Ok(())
    }

    pub fn get_topic_events(&mut self, topic: &str) -> ResultReturn<(String)> {
        let events = self.topic_events.get(&String::from(topic)).unwrap();
        Ok(events.clone())
    }

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
