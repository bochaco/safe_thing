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

use ffi_utils::test_utils::{send_via_user_data, sender_as_user_data};
use ffi_utils::FfiResult;
use safe_app::ffi::mutable_data::{
    entries::mdata_list_entries, mdata_entries, mdata_get_value,
};
use safe_app::App;
#[cfg(not(feature = "fake-auth"))]
use safe_app::AppError;
use safe_core::ffi::MDataInfo;
#[cfg(not(feature = "fake-auth"))]
use safe_core::ipc::resp::AuthGranted;
#[cfg(not(feature = "fake-auth"))]
use safe_core::ipc::{decode_msg, encode_msg, gen_req_id, IpcError, IpcMsg, IpcReq, IpcResp};
use safe_core::MDataEntry;
use std::os::raw::c_void;
use std::slice;
use std::sync::mpsc;

#[cfg(not(feature = "fake-auth"))]
pub fn encode_ipc_msg(req: IpcReq) -> Result<String, AppError> {
    let req_id: u32 = gen_req_id();
    let encoded = encode_msg(&IpcMsg::Req { req_id, req })?;
    Ok(encoded)
}

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

// Retrieve the list of entries from a MutableData
pub fn mdata_get_entries(app: &App, mdata: &MDataInfo) -> Result<Vec<(Vec<u8>, Vec<u8>)>, i32> {
    extern "C" fn mdata_entries_cb(user_data: *mut c_void, res: *const FfiResult, entries_h: u64) {
        unsafe {
            let result: Result<u64, i32> = if (*res).error_code == 0 {
                Ok(entries_h)
            } else {
                Err((*res).error_code)
            };

            send_via_user_data(user_data, result);
        }
    }

    let (tx, rx) = mpsc::channel::<Result<u64, i32>>();
    let mut ud = Default::default();
    unsafe {
        mdata_entries(
            app as *const App,
            mdata,
            sender_as_user_data(&tx, &mut ud),
            mdata_entries_cb,
        )
    };

    let mdata_entries_handle = rx.recv().unwrap().unwrap();

    // now that we have the mdta entries handle, let's get the list
    extern "C" fn mdata_list_entries_cb(
        user_data: *mut c_void,
        res: *const FfiResult,
        entries: *const MDataEntry,
        entries_len: usize,
    ) {
        unsafe {
            let result: Result<Vec<(Vec<u8>, Vec<u8>)>, i32> = if (*res).error_code == 0 {
                let entries_slice = slice::from_raw_parts(entries, entries_len);
                let entries_vec: Vec<(Vec<u8>, Vec<u8>)> = entries_slice
                    .iter()
                    .map(|entry| {
                        let key = slice::from_raw_parts(entry.key.key, entry.key.key_len).to_vec();
                        let value =
                            slice::from_raw_parts(entry.value.content, entry.value.content_len)
                                .to_vec();
                        (key, value)
                    })
                    .collect();

                Ok(entries_vec)
            } else {
                Err((*res).error_code)
            };

            send_via_user_data(user_data, result);
        }
    }

    let (tx, rx) = mpsc::channel::<Result<Vec<(Vec<u8>, Vec<u8>)>, i32>>();
    let mut ud = Default::default();
    unsafe {
        mdata_list_entries(
            app as *const App,
            mdata_entries_handle,
            sender_as_user_data(&tx, &mut ud),
            mdata_list_entries_cb,
        )
    };

    let mdata_entries = rx.recv().unwrap();
    mdata_entries
}
