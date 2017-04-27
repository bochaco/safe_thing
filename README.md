# SAFEthing Framework
**S**ecure **A**ccess **F**or **E**very**thing**

### Project Goals
- Provide IoT software developers with an easy and straight forward way to integrate their IoT devices to the SAFE network without worrying about network, security and/or authentication & authorisation protocols.
- Create a project which can be used as a knowledge base for developers trying to learn and implement software for the SAFE network.
- Allow the SAFE network community to participate in a project which creates added value for the SAFE network, and which leads to help promoting it and to achieve mass adoption.
- Ultimatly, **have the next-gen IoT framework and communication protocol to be designed and developed by the SAFE network community itself!**


### The Library and API
The SAFEthing library is composed of several parts but its core is just a Rust crate with a simple and well defined Rust API.

Internally it contains all the mechanisms to communicate with the SAFE network thru the [safe_client_libs](https://github.com/maidsafe/safe_client_libs), abstracting the client application from all of it without the need for the application developer to even understand how the SAFE network works.

In an analogous way as to how the SAFE network itself provides different programming languages bindings, there will be a SAFEthing Rust FFI interface which can be used to interact with the API from any programming language, like C/C++, but also a set of different language bindings so people can develop their SAFEthings software even with JavaScript, Python, Lua, etc.

A WebService API will also be created on top of the Rust API to allow the communication with the SAFEthings network thru a REST interface. This is mainly intended to support smart home devices, and tools potentially needed to provision them.

![SAFEthing Library Stack](misc/SAFEthing_Stack.png)

### SAFEthings
When a SAFEthing registers to the network it provides a set of information which describes its behaviour, functionality and/or service it exposes.

#### Attributes
Attributes are exposed to provide information about the device/thing to other things that connect to the network. They can also be used by humans to identify the device and/or its functionalities when connecting to it thru a console/portal.

An attribute can be either static or dynamic. Some examples of static attributes are the firmware version, device name and model, and their values do not depend or change according to the thing's functioning or state.

On the other hand, some attributes could contain dynamic values which are updated by the SAFEthing, e.g. a temperature sensor could update a dynamic attribute with the current reading, or a SAFEthing with a more complex functionality could expose an attribute which described its current state.

#### Topics
A SAFEthing can expose a set of topics that other SAFEthings can subcribe to in order to receive notifications upon events.

Different events result in different type of notifications, a topic can describe a certain type of events, depending on how the SAFEthing is designed to expose them.

As an example, the temperature sensor can expose a "temperature change" topic that another SAFEthing can subscribe to and receive notifications upon a temperature change event.

When subscribing to a topic, a set of filters can optionally be provided in order to reduce the notifications to be received to just those which the subscriber is really interested in. E.g. a SAFEthing might be interested in being notified only if the current temperature goes over a threshold.

TODO: describe subscriptions and notifications filters & parameters

#### Actions
Another way to interact with a SAFEthing is by requesting an action. The set of actions are usually static but there could be cases that a SAFEthing wants to expose some actions only in certain moments or periods of time.

Each action is exposed with a name, a set of input parameters it expects and/or supports, and the definition of its output.

The execution of an action is asynchronous. When an action is requested to a SAFEthing, it is added to its actions requests queue. The order and/or priority of execution of each of the actions is application specific, although the framework will provide some utilities to retrieve them in the order it was pre-defined for the SAFEthing.

#### Access Type
SAFEthing's Attributes, Topics, and Actions, are associated to an Access Type. The Access Type defines the set of SAFEthings that are allow to access the exposed functionality and information.

As an example, the data you send to a SAFEthing printer should be encrypted and available to access by the sender and the printer devices only. Or if you have a set of devices at home that interact among them, you will want that only your devices can see each other's information and functionalities but no one else.


### The Communication Protocol
TODO

#### Use cases

##### Register and Publish
![Register and Publish](misc/UC_register_and_publish.png)

##### Subscriptions and Notifications
![Subscriptions and Notifications](misc/UC_subscriptions_and_notifications.png)

##### Wallet
TODO

### Example of SAFEthing Rust client

``` rust
extern crate safe_thing;

use safe_thing::{SAFEthing, ThingAttr, Topic, ActionDef, AccessType};

fn print_requested_notif(thing_id: &str, topic: &str, data: &str) {
    println!("Notification received from thing_id: {}, topic: {}, data: {}", thing_id, topic, data)
}

pub fn main() {
    let id = "printer-serial-number-01010101";
    let auth_uri = "safe-bmv0lm1hawrzywzllmv4yw1wbgvzlm1krxhhbxbszq:AQAAAOIDI_gAAAAAAAAAACAAAAAAAAAAGWzDHH2GG-TUtS_qLvytHNXrAPWGtI6QLDuoP28EE_0gAAAAAAAAALPyoRvbtvPKs9bWYhkdhfkltybFTBJerAWEARetysrtvsjSRTHVRTA_a6ysxSGIUWz9pOLlq9hRMM-EJQctDpVkhRTXPar-W0AAAAAAAAAA-O8HsVV5ZZbiAwWTTFXQeNX7pSYtLmZXRHnrdVyXZvv_a6ysxSGIUWz9pOLlq9hRMM-EJQctDpVkhRTXPar-WyAAAAAAAAAAUnTeCf39C-KDfioarbgDedqYhu_ZEpCHK_CatkiYNFUgAAAAAAAAAOTkFE7GibxaH0egTV1NtczggZkyAsCVRY6AcbceiSNfAAAAAAAAAAAAAAAAAAAAAAAAAAAAMCralz2EJh0ML2wMZLBhh0hELI1dIQUlVtaWHqIClqmYOgAAAAAAABgAAAAAAAAA2lo16ByCIq4SnojMIRPV_RSvQIOelGUD";

    let attributes = vec![
        ThingAttr::new("name", "Printer at home"),
        ThingAttr::new("model", "HP LaserJet 400 M401"),
        ThingAttr::new("firmware", "v1.3.0"),
        ThingAttr::new("status", "on"),
        ThingAttr::new("ink-level", "%")
    ];
    let topics = vec![
        Topic::new("printRequested"),
        Topic::new("printSuccess"),
        Topic::new("printFail"),
        Topic::new("outOfInk"),
    ];
    let actions = vec![
        ActionDef::new("turnOn", AccessType::Owner, vec![]),
        ActionDef::new("turnOff", AccessType::Owner, vec!["timer"]),
        ActionDef::new("print", AccessType::Owner, vec!["data", "copies"]),
        ActionDef::new("orderInk", AccessType::Group, vec![])
    ];

    let mut safe_thing: SAFEthing;
    match SAFEthing::new(id, auth_uri, print_requested_notif) {
        Ok(s) => safe_thing = s,
        Err(e) => panic!("Couldn't create SAFEthing instance: {}", e)
    }

    let _ = safe_thing.register_thing(attributes, topics, actions);

    match safe_thing.get_thing_status(id) {
        Ok(status) => println!("\nCurrent status: {:?}", status),
        Err(e) => println!("Failed getting status: {}", e)
    }

    ...
}
```

### Project Development Roadmap

- [ ] Cross-compilation tools for MIPS
- [ ] Cross-compilation tools for ARM
- [ ] Creation of test suite for API
- [ ] Implementation of FFI interface
- [ ] Implementation of Javascript binding
- [ ]
- [ ] Implementation of WebService API
