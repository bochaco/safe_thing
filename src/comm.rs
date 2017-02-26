//use rust_sodium::crypto::hash::sha256;
//use std::fmt;
use std::collections::HashMap;

enum FilterOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan
}

struct EventFilter {
    arg_name: String,
    arg_op: FilterOperator,
    arg_value: String
}

struct Subscription {
    topic: String,
    filters: Vec<EventFilter>
}

// map of thing_id => vector of Subscription
type Subscriptions = HashMap<String, Vec<Subscription>>;
// vector of (topic name, data)
type Events = Vec<(String, String)>;


pub struct SAFEoTComm {
    thing_id: String,

    // the following is temporary since the safe_app already provides a cache
    subscriptions: Subscriptions,
    events: Events
}

impl SAFEoTComm {
    pub fn new(thing_id: &str) -> SAFEoTComm {
        SAFEoTComm {
            thing_id: String::from(thing_id),
            subscriptions: Subscriptions::new(),
            events: Events::new()
        }
    }

    pub fn addSubscription(&mut self, thing_id: &str, topic: &str, /*filters*/) -> Option<String> {
        self.subscriptions.get_mut(thing_id).map(|mut topics| {
            topics.push(Subscription{topic: String::from(topic), filters: vec![]})
        });
        None
    }

    pub fn publishEvent(&mut self, topic: &str, data: &str) -> Option<String> {
        self.events.push((String::from(topic), String::from(data)));
        None
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
