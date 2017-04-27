const safeThingLib = require('../bindings/nodejs');

let id = "thingA_id";

let safeThing = safeThingLib.newSAFEthing(id);
console.log("Thing A: ", safeThing);

let attrs = [{attr: "5", value: "67"}, {attr: "6", value: "68"}];
let register_out = safeThing.register_thing(attrs);
console.log("Register: ", register_out);

let publish_out = safeThing.publish_thing(id);
console.log("Publish: ", publish_out);

// this is not needed
safeThing.destroy();
