const weak = require('weak');
const fastcall = require('fastcall');
const ref = fastcall.ref;
const StructType = fastcall.StructType;

const os = require('os');
const path = require('path');

const dir = path.dirname(__filename);

const LIB_FILENAME = {
  win32: 'safe_o_t.dll',
  darwin: 'libsafe_o_t.dylib',
  linux: 'libsafe_o_t.so'
}[os.platform()];

const safeot_lib = new fastcall.Library(path.join(dir, LIB_FILENAME));

// Definition of the ThingAttr
safeot_lib.struct({
  ThingAttr: {
    attr: 'string',
    value: 'string'
  }
});
safeot_lib.array({ThingAttrArray: 'ThingAttr'});

safeot_lib.function({safe_o_t_new: ['pointer', ['string'] ]})
          .function({safe_o_t_register_thing: [ref.types.int, ['pointer', 'ThingAttrArray', ref.types.size_t] ]})
          .function({safe_o_t_publish_thing: [ref.types.int, ['pointer', 'string'] ]})
          .function({safe_o_t_delete: ['void', ['pointer'] ]});

module.exports.newSAFEoT = function(thing_id)  {
	let handle = safeot_lib.interface.safe_o_t_new(thing_id);

  let safeot = new SAFEoT(handle);
  let ref = weak(safeot, function () {
    console.log('"obj" has been garbage collected!')
    safeot.destroy();
  });

	return ref;
};

class SAFEoT {
  constructor(handle) {
    this.handle = handle;
  };

  register_thing(attrs) {
    return safeot_lib.interface.safe_o_t_register_thing(this.handle, attrs, attrs.length);
  };

  publish_thing(thing_id) {
    return safeot_lib.interface.safe_o_t_publish_thing(this.handle, thing_id);
  };

  destroy() {
    safeot_lib.interface.safe_o_t_delete(this.handle);
    this.handle = null;
  };
};
