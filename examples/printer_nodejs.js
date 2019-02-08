// Copyright 2017-2019 Gabriel Viganotti <@bochaco>.
//
// This file is part of the SAFEthing Framework.
//
// The SAFEthing Framework is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The SAFEthing Framework is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with the SAFEthing Framework. If not, see <https://www.gnu.org/licenses/>.

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
