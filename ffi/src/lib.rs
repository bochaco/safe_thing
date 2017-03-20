extern crate safe_o_t;

use safe_o_t::{SAFEoT, ThingAttr};
use std::slice;
use std::ffi::CStr;
use std::os::raw::c_char;

pub type SAFEoTHandle = *mut SAFEoT;

pub type ErrorCode = i32;

#[derive(Clone, Debug)]
pub struct FfiThingAttr {
    attr: *const c_char,
    value: *const c_char
}

#[no_mangle]
pub extern "C" fn safe_o_t_new(thing_id: *const c_char) -> SAFEoTHandle {
    unsafe {
        let id = CStr::from_ptr(thing_id);
        let id_str = id.to_str().unwrap();
        let safeot: Box<SAFEoT> = Box::new(SAFEoT::new(id_str).unwrap());
        let _handle = Box::into_raw(safeot);
        _handle
    }
}

#[no_mangle]
pub extern "C" fn safe_o_t_register_thing(handle: SAFEoTHandle,
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
pub extern "C" fn safe_o_t_publish_thing(handle: SAFEoTHandle,
                                            thing_id: *const c_char) -> ErrorCode {
    unsafe {
        let id = CStr::from_ptr(thing_id);
        let id_str = id.to_str().unwrap();
        (*handle).publish_thing(id_str);
        0
    }
}

#[no_mangle]
pub extern "C" fn safe_o_t_delete(handle: SAFEoTHandle) {
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
