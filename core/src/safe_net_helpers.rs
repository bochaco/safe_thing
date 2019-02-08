extern crate safe_app;
extern crate safe_core;
extern crate ffi_utils;

use self::safe_app::App;
use self::safe_app::ffi::mutable_data::{mdata_get_value};
use self::safe_core::ffi::MDataInfo;
use self::ffi_utils::FfiResult;
use self::ffi_utils::test_utils::{send_via_user_data, sender_as_user_data};
use std::{slice};
use std::sync::mpsc;
use std::os::raw::{/*c_char, */c_void};
#[cfg(not(feature="fake-auth"))]
use self::safe_core::ipc::resp::AuthGranted;
#[cfg(not(feature="fake-auth"))]
use self::safe_core::ipc::{decode_msg, IpcMsg, IpcResp, IpcError};

#[cfg(not(feature = "fake-auth"))]
pub fn decode_ipc_msg(ipc_msg: &str) -> Result<AuthGranted, IpcError> {
    let msg = decode_msg(&ipc_msg)?;
    match msg {
        IpcMsg::Resp {
            resp: IpcResp::Auth(res),
            req_id: _
        } => {
            match res {
                Ok(auth_granted) => {
                    Ok(auth_granted)
                }
                _ => {
                    return Err(IpcError::InvalidMsg.into());
                }
            }
        }
        IpcMsg::Revoked { .. } => {
            return Err(IpcError::InvalidMsg.into());
        }
        _ => {
            return Err(IpcError::InvalidMsg.into());
        }
    }
}

// Retrieve the value mapped to the provided key from a MutableData
pub fn mdata_get(app: &App, mdata: &MDataInfo, key: &str) -> Result<(Vec<u8>, u64), i32> {
    extern "C" fn get_value_cb(user_data: *mut c_void, res: *const FfiResult,
                                    val: *const u8, len: usize, version: u64) {
        unsafe {
            let result: Result<(Vec<u8>, u64), i32> = if (*res).error_code == 0 {
                let value = slice::from_raw_parts(val, len).to_vec();
                Ok((value, version))
            } else {
                Err((*res).error_code)
            };

            send_via_user_data(user_data, result);
        }
    }

    let (tx, rx) = mpsc::channel::<Result<(Vec<u8>, u64), i32>>();
    let mut ud = Default::default();
    unsafe {
        mdata_get_value(app as *const App, mdata, key.as_ptr(), key.len(),
                            sender_as_user_data(&tx, &mut ud), get_value_cb)
    };

    let result = rx.recv().unwrap();
    result
}
