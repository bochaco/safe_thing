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

extern crate safe_thing;

use safe_thing::{SAFEthing, ThingAttr};
use std::slice;
use std::ffi::CStr;
use std::os::raw::c_char;

pub type SAFEthingHandle = *mut SAFEthing;

pub type ErrorCode = i32;

#[derive(Clone, Debug)]
pub struct FfiThingAttr {
    attr: *const c_char,
    value: *const c_char
}

#[no_mangle]
pub extern "C" fn safe_thing_new(thing_id: *const c_char) -> SAFEthingHandle {
    unsafe {
        let id = CStr::from_ptr(thing_id);
        let id_str = id.to_str().unwrap();
        let safe_thing: Box<SAFEthing> = Box::new(SAFEthing::new(id_str).unwrap());
        let _handle = Box::into_raw(safe_thing);
        _handle
    }
}

#[no_mangle]
pub extern "C" fn safe_thing_register_thing(handle: SAFEthingHandle,
                                    attrs: *const FfiThingAttr, attrs_len: usize) -> ErrorCode {
    unsafe {
        let ffi_attrs = slice::from_raw_parts(attrs, attrs_len).to_vec();
        let attr = CStr::from_ptr(ffi_attrs[0].attr);
        let attr_str = attr.to_str().unwrap();

        let value = CStr::from_ptr(ffi_attrs[0].value);
        let value_str = value.to_str().unwrap();

        let thing_attr = ThingAttr::new(attr_str, value_str);
        (*handle).register_thing(vec![thing_attr], vec![], vec![]);
        0
    }
}

#[no_mangle]
pub extern "C" fn safe_thing_publish_thing(handle: SAFEthingHandle,
                                            thing_id: *const c_char) -> ErrorCode {
    unsafe {
        let id = CStr::from_ptr(thing_id);
        let id_str = id.to_str().unwrap();
        (*handle).publish_thing(id_str);
        0
    }
}

#[no_mangle]
pub extern "C" fn safe_thing_delete(handle: SAFEthingHandle) {
    unsafe {
        if handle.is_null() {return}
        let _ = Box::from_raw(handle);
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
//        hello_rust(5);
    }
}
