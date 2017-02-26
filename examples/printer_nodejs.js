const safeotlib = require('../bindings/nodejs');

let id = "thingA_id";
let safeot = safeotlib.newSAFEoT(id);
console.log("Thing A: ", safeot);
