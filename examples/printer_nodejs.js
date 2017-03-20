const safeotlib = require('../bindings/nodejs');

let id = "thingA_id";

let safeot = safeotlib.newSAFEoT(id);
console.log("Thing A: ", safeot);

let attrs = [{attr: "5", value: "67"}, {attr: "6", value: "68"}];
let register_out = safeot.register_thing(attrs);
console.log("Register: ", register_out);

let publish_out = safeot.publish_thing(id);
console.log("Publish: ", publish_out);

// this is not needed
safeot.destroy();
