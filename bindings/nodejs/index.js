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

const weak = require('weak');
const fastcall = require('fastcall');
const ref = fastcall.ref;
const StructType = fastcall.StructType;

const os = require('os');
const path = require('path');

const dir = path.dirname(__filename);

const LIB_FILENAME = {
  win32: 'safe_thing.dll',
  darwin: 'libsafe_thing.dylib',
  linux: 'libsafe_thing.so'
}[os.platform()];

const safe_thing_lib = new fastcall.Library(path.join(dir, LIB_FILENAME));

// Definition of the ThingAttr
safe_thing_lib.struct({
  ThingAttr: {
    attr: 'string',
    value: 'string'
  }
});
safe_thing_lib.array({ThingAttrArray: 'ThingAttr'});

safe_thing_lib.function({safe_thing_new: ['pointer', ['string'] ]})
          .function({safe_thing_register_thing: [ref.types.int, ['pointer', 'ThingAttrArray', ref.types.size_t] ]})
          .function({safe_thing_publish_thing: [ref.types.int, ['pointer', 'string'] ]})
          .function({safe_thing_delete: ['void', ['pointer'] ]});

module.exports.newSAFEoT = function(thing_id)  {
	let handle = safe_thing_lib.interface.safe_thing_new(thing_id);

  let safe_thing = new SAFEthing(handle);
  let ref = weak(safe_thing, function () {
    console.log('"obj" has been garbage collected!')
    safe_thing.destroy();
  });

	return ref;
};

class SAFEthing {
  constructor(handle) {
    this.handle = handle;
  };

  register_thing(attrs) {
    return safe_thing_lib.interface.safe_thing_register_thing(this.handle, attrs, attrs.length);
  };

  publish_thing(thing_id) {
    return safe_thing_lib.interface.safe_thing_publish_thing(this.handle, thing_id);
  };

  destroy() {
    safe_thing_lib.interface.safe_thing_delete(this.handle);
    this.handle = null;
  };
};
