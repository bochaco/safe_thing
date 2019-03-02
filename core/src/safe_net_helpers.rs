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

extern crate ffi_utils;
extern crate safe_app;
extern crate safe_core;

use self::ffi_utils::test_utils::{send_via_user_data, sender_as_user_data};
use self::ffi_utils::FfiResult;
use self::safe_app::ffi::mutable_data::mdata_get_value;
use self::safe_app::App;
use self::safe_core::ffi::MDataInfo;
#[cfg(not(feature = "fake-auth"))]
use self::safe_core::ipc::resp::AuthGranted;
#[cfg(not(feature = "fake-auth"))]
use self::safe_core::ipc::{decode_msg, IpcError, IpcMsg, IpcResp};
use std::os::raw::c_void;
use std::slice;
use std::sync::mpsc;

#[cfg(not(feature = "fake-auth"))]
pub fn decode_ipc_msg(ipc_msg: &str) -> Result<AuthGranted, IpcError> {
    let msg = decode_msg(&ipc_msg)?;
    match msg {
        IpcMsg::Resp {
            resp: IpcResp::Auth(res),
            req_id: _,
        } => match res {
            Ok(auth_granted) => Ok(auth_granted),
            _ => {
                return Err(IpcError::InvalidMsg.into());
            }
        },
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
    extern "C" fn get_value_cb(
        user_data: *mut c_void,
        res: *const FfiResult,
        val: *const u8,
        len: usize,
        version: u64,
    ) {
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
        mdata_get_value(
            app as *const App,
            mdata,
            key.as_ptr(),
            key.len(),
            sender_as_user_data(&tx, &mut ud),
            get_value_cb,
        )
    };

    let result = rx.recv().unwrap();
    result
}
