const ref = require('ref');
const ffi = require('ffi');
const os = require('os');
const path = require('path');

const dir = path.dirname(__filename);

const LIB_FILENAME = {
  win32: 'safe_o_t.dll',
  darwin: 'libsafe_o_t.dylib',
  linux: 'libsafe_o_t.so'
}[os.platform()];

const safeot_lib = ffi.Library(path.join(dir, LIB_FILENAME), { "hello_rust": ['string', ['int'] ] });

module.exports.newSAFEoT = function newSAFEoT(thing_id) {
	let str = safeot_lib.hello_rust(thing_id);
	return str;
}
