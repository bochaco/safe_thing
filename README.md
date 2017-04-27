### SAFEthings

When a SAFEthing registers to the network it provides a set of information which describes its behaviour, functionality and/or service it exposes.

### Attributes

Attributes are exposed to provide information about the device/thing to other things that connect to the network. They can also be used by humans to identify the device and/or its functionalities when connecting to it thru a console/portal.

An attribute can be either static or dynamic. Some examples of static attributes are the firmware version, device name and model, and their values do not depend or change according to the thing's functioning or state.

On the other hand, some attributes could contain dynamic values which are updated by the SAFEthing, e.g. a temperature sensor could update a dynamic attribute with the current reading, or a SAFEthing with a more complex functionality could expose an attribute which described its current state.

### Topics

A SAFEthing can expose a set of topics that other SAFEthings can subcribe to in order to receive notifications upon events.

Different events result in different type of notifications, a topic can describe a certain type of events, depending how the SAFEthing s design to expose them.

As an example, the temperature sensor can expose a 'temperature change" topic that another SAFEthing can subscribe to and receive noifications upon a temperature change event.

When subscribing to a topic, a set of filters can optionally be provided in order to reduce the notifications to be received to just those which the subscriber is really interested in. E.g. a SAFEthing might be interested in being notified only if the current temperature goes over a threshold.

TODO: talk about subscribtions and notifications filters & parameters

### Actions

Another way to interact with a SAFEthing is by requesting an action. The set of actions are usually static but there could be cases that a SAFEthing wants to expose some actions only in certain moments or periods of time.

Each action is exposed with a name, a set of input parameters it expects and/or supports, and the definition of its output.

The execution of an action is asynchronous. When an action is requested to a SAFEthing, it is added to its actions requests queue. The order and/or priority of execution of each of the actions is application specific, although the framework will provide some utilities to retrieve them in the order it was pre-defined for the SAFEthing.

## Access Type

SAFEthing's Attributes, Topics, and Actions, are associated to an Access Type. The Access Type defines the set of SAFEthings that are allow to access the exposed functionality and information.

There a different levels defined for the Access Type:
#### Thing
Only the SAFEthing itself can access. This is the default and lowest level of access type.
#### Owner
access also is allowed to an individual, application or system that is the actual owner of the Thing, plus the Thing itself.
#### Group
access to a group of individuals or Things, plus the Owner and the Thing itself.
#### All
access is allowed to anyone or anything, including the Thing itself.
