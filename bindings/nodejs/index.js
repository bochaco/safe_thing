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
