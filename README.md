# SAFEthing Framework
**S**ecure **A**ccess **F**or **E**very**thing**

### Project Goals
- Provide IoT software developers with an easy and straight forward way to integrate their IoT devices to the SAFE Network without worrying about network, security and/or authentication & authorisation protocols.
- Create a project which can be used as a knowledge base for developers trying to learn and implement software for the SAFE Network.
- Allow the SAFE Network community to participate in a project which creates added value for the SAFE Network, and which leads to help promoting it and to achieve mass adoption.
- Ultimately, **have the next-gen IoT framework and communication protocol to be designed and developed by the SAFE Network community itself!**

### Incentive
Current IoT protocols and frameworks (e.g. MQTT, CoAP) have a similar set of problems to solve, their challenges seem to be mainly related to security and NAT (to connect to the devices located in different networks without public IPs). These protocols solve this by having servers/brokers in the network, which not just adds complexity to the development of the devices' software, but it also brings in big security and privacy concerns in relation to who you are sharing the information with when your IoT things need to communicate using servers, you are forced once again to trust third parties.

[The SAFE Network](https://safenetwork.tech/) has many properties which makes it an excellent solution to solve all these issues, and more, while at the same time developers and users don't need to cater for them in the first place, they can just focus on developing the main logic of the IoT devices!.

This framework allows developers to integrate their IoT devices onto the SAFE Network easily, without even needing to understand much of the SAFE Network API or technicalities.

### Run example SAFEthing applications
This project is in its very early stage, however there are already a couple of SAFEthing applications which showcase how the SAFEthing API can be used to implement a gardening system, having one SAFEthing which manages a soil moisture sensor and a water valve ([core/examples/gardening_device.rs](core/examples/gardening_device.rs)), and a second SAFEthing which acts as a monitoring and controller device for the gardening system ([core/examples/gardening_controller.rs](core/examples/gardening_controller.rs)).

Although the following is a summary of what these applications try to demonstrate, there are many more details of their functionality commented out in the code itself, so please refer to them for further explanations.

#### Prerequisites
- In order to be able to run these example applications, please make sure you have rustc v1.33.0 or later.

- You'll also need the [SAFE Authenticator CLI](https://github.com/maidsafe/safe-authenticator-cli) running locally and exposing its WebService interface for authorising applications, and also be logged in to a SAFE account created on the mock network (i.e. `MockVault` file). Each of these SAFEthings applications will send an authorisation request to `http://localhost:41805/authorise/` endpoint which can be made available by following the instructions in [this section of the safe_auth CLI documentation](https://github.com/maidsafe/safe-authenticator-cli#execute-authenticator-service-exposing-restful-api), making sure the port number you set is `41805`.

The first thing to be done is clone this repository and switch to the `core` subdirectory of it:
```
$ git clone https://github.com/bochaco/safe_thing.git
$ cd ./safe_thing/core
```

Now you can run one of the two SAFEthings, the one which manages a soil moisture sensor and water valve, by executing the following command:
```
$ cargo run --features use-mock-routing --example gardening_device
```

This SAFEthing application will publish a few attributes, e.g. the `moisture-level` and `valve-state` dynamic attributes, a couple of topics that other SAFEthing can register to, `VeryWetAlarm` and `VeryDryAlarm`, as well as a couple of actions which can be triggered, `OpenValve` and `CloseValve`, to open and close the water valve respectively. This SAFEthing application also simulates the soil moisture level being increased or decreased depending if the water valve is open or closed, just for the sake of being able to understand how another SAFEthing can subscribe to topics and dynamic attributes, as well as send action requests to it.

It is now time to run the second SAFEthing, the gardening controller. In order to do so, please open a new terminal window, make sure you are still in the same `safe_thing/core/` directory, and run it with the following command:
```
$ cargo run --features use-mock-routing --example gardening_controller
```

The gardening controller SAFEthing will subscribe to the dynamic attributes and topics exposed by the gardening device (the first SAFEthing application we ran), monitoring if the soil moisture level goes below a certain threshold. In such a case, it will send a request to open the water valve until the moisture level reaches an upper level which is meant to be healthy for our plants, at which point it will be sending a second request to close the water valve. This cycle will repeat indefinitely as the soil moisture level will automatically start dropping when the water valve is close.

You can see the output of both SAFEthing applications (in the two terminal consoles) to see how they react to the notifications and action requests being sent between them. If you prefer to see more level of details, you can enable any of the logging level, e.g. to see the `debug` level of logs generated by the `safe_thing` framework you can instead run them with the following commands:
```
$ RUST_LOG=safe_thing=debug cargo run --features use-mock-routing --example gardening_device
```
and
```
$ RUST_LOG=safe_thing=debug cargo run --features use-mock-routing --example gardening_controller
```

### The Library and API
The SAFEthing library is composed of several parts but its core is just a Rust crate with a simple and well defined Rust API.

Internally it contains all the mechanisms to communicate with the SAFE Network through the [safe_client_libs](https://github.com/maidsafe/safe_client_libs), abstracting the client application from all of it without the need for the application developer to even understand how the SAFE Network works.

In an analogous way as to how the SAFE Network itself provides different programming languages bindings, there will be a SAFEthing Rust FFI interface which can be used to interact with the API from any programming language, like C/C++, but also a set of different language bindings so people can develop their SAFEthings software even with JavaScript, Python, Lua, Go, etc.

A WebService API will also be created on top of the Rust API to allow the communication with the SAFEthings network through a REST interface. This is mainly intended to support smart home devices, and tools potentially needed to provision them.

![SAFEthing Library Stack](misc/SAFEthing_Stack.png)

### SAFEthings
When a SAFEthing registers to the network it provides a set of information which describes its behaviour, functionality and/or service it exposes.

#### Attributes
Attributes are exposed to provide information about the device/thing to other things that connect to the network. They can also be used by humans to identify the device and/or its functionalities when connecting to it through a console/portal.

An attribute can be either static or dynamic. Some examples of static attributes are the firmware version, device name and model, and their values do not depend or change according to the thing's functioning or state. Note these attributes could still be changed/modified by the user, but in the sense that their values are not tied to the device's state.

On the other hand, some attributes could contain dynamic values which are updated by the SAFEthing, e.g. a temperature sensor could update a dynamic attribute with the current reading, or a SAFEthing with a more complex functionality could expose an attribute which describes its current state.

#### Topics
A SAFEthing can expose a set of topics that other SAFEthings can subscribe to in order to receive notifications upon events.

Different events result in different type of notifications, a topic can describe a certain type of events, depending on how the SAFEthing is designed to expose them.

As an example, the temperature sensor can expose a "temperature change" topic that another SAFEthing can subscribe to and receive notifications upon a temperature change event.

When subscribing to a topic, a set of filters can optionally be provided in order to reduce the notifications to be received to just those which the subscriber is really interested in. E.g. a SAFEthing might be interested in being notified only if the current temperature goes over a threshold.

TODO: subscriptions to dynamic attributes vs. topics events
TODO: retained messages support
TODO: describe subscriptions and notifications filters & parameters

TODO: supporting birth/close/last-will/testament topics

#### Actions
Another way to interact with a SAFEthing is by requesting an action. The set of actions are usually static but there could be cases that a SAFEthing wants to expose some actions only in certain moments or periods of time.

Each action is exposed with a name, a set of input parameters it expects and/or supports, and the definition of its output.

The execution of an action is asynchronous. When an action is requested to a SAFEthing, it is added to its actions requests queue. The order and/or priority of execution of each of the actions is application specific, although the framework will provide some utilities to retrieve them in the order it was predefined for the SAFEthing.

#### Access Type
SAFEthing's Attributes, Topics, and Actions, are associated to an Access Type. The Access Type defines the set of SAFEthings that are allowed to access the exposed functionality and information.

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

### Snippet of SAFEthing Rust client

The following is a snippet of how a SAFEthing client application looks like, please refer to the [core examples folder](core/examples/) to see the complete code, and refer to [Run example SAFEthing applications](#run-example-safething-applications) for instructions to run them.

``` rust
use safe_thing::{AccessType, ActionDef, SAFEthing, ThingAttr, Topic};

pub fn main() {
    // Let's create a SAFEthing id for the gardening device, this could be the device serial number
    let id = "gardening-device-serial-number-01010101";

    // Let's have the SAFEthing framework to authorise the device with any
    // available SAFE Authenticator rather than providing the authorisation credentials
    let auth_uri = "";

    let attributes = [
        ThingAttr::new("name", "SAFEthing Gardening Device", false),
        ThingAttr::new("firmware", "v0.1.0", false),
        ThingAttr::new("moisture-level", "", true),
        ThingAttr::new("pressure-psi", "50", true),
        ThingAttr::new("valve-state", "closed", true),
    ];

    let topics = [
        Topic::new("VeryDryAlarm", AccessType::All),
        Topic::new("VeryWetAlarm", AccessType::All),
    ];

    let actions = [
        ActionDef::new("OpenValve", AccessType::All, &["psi"]),
        ActionDef::new("CloseValve", AccessType::All, &[]),
    ];

    // Let's create an instance of SAFEthing for this device.
    // We already provide the two callback functions to be called
    // for subscriptions notifications and action requests respectively.
    let mut safe_thing =
        SAFEthing::new(&id, auth_uri, &subscriptions_notif, &action_request_notif).unwrap();

    // Register the SAFEthing on the network, this won't make it active yet
    // but it will just store the device's data onto the network as a SAFEthing entity
    safe_thing
        .register(&attributes, &topics, &actions)
        .map(|()| println!("Gardening device registered on the network"))
        .unwrap();

    // Let's now make it active and ready for receiving action requests from other SAFEthings
    safe_thing.publish().expect("Failed to publish SAFEthing");

    // We now go into an infinite loop which contains the main logic of this device
    loop {
        let current_moisture_level: f32 = /* get soil moisture level from sensor reading */

        // Let's keep the published dynamic attribute up to date for any
        // other SAFEthing to be notified if it's interested in knowing about the new value
        println!(
            "Updating value of 'moisture-level' dynamic attribute to '{}'",
            current_moisture_level
        );
        safe_thing
            .set_attr_value("moisture-level", &current_moisture_level.to_string())
            .unwrap();

        // This device also supports two topics ("VeryWetAlarm" and "VeryDryAlarm") that
        // other SAFEthings can register to in order to receive notifications when it detects
        // that the soil moisture level goes beyond certain thresholds. Thus we should check
        // the current moisture level and send the notification for the corresponding topic accordingly.
        if current_moisture_level > 8.0 {
            let _ = safe_thing.notify("VeryWetAlarm", "");
        } else if current_moisture_level < 3.0 {
            let _ = safe_thing.notify("VeryDryAlarm", "");
        }

        // Let's wait for some time before going into a new cycle of reading the
        // current soil moisture level and publishing the new value
        thread::sleep(Duration::from_millis(3000));
    }
}
```

### Project Development Roadmap

- [ ] Document API
- [ ] Creation of test suite for API
- [ ] Creation of a showcasing app using the test SAFE Network
- [ ] Cross-compilation tools/doc for MIPS
- [ ] Cross-compilation tools/doc for ARM
- [ ] Implementation of FFI interface
- [ ] Implementation of Javascript binding
- [ ] Documentation of the communication protocol
- [ ] Implementation of WebService API


## License

General Public License (GPL), version 3 ([LICENSE](LICENSE))

Copyright (c) 2017-2019 Gabriel Viganotti <@bochaco>.

This file is part of the SAFEthing Framework.

The SAFEthing Framework is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

The SAFEthing Framework is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.
