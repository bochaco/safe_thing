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

extern crate log;

use log::{debug, trace, warn};

extern crate ffi_utils;
extern crate safe_app;
extern crate safe_core;

use self::safe_app::ffi::crypto::{app_pub_sign_key, sha3_hash};
use self::safe_app::ffi::object_cache::{
    MDataEntryActionsHandle, MDataPermissionsHandle, SignPubKeyHandle,
};
#[cfg(feature = "fake-auth")]
use self::safe_app::test_utils::create_app;
use self::safe_app::App;
//use self::safe_app::ffi::mdata_info::{mdata_info_new_private};
use self::safe_app::ffi::mutable_data::entry_actions::{
    mdata_entry_actions_insert, mdata_entry_actions_new, mdata_entry_actions_update,
};
use self::safe_app::ffi::mutable_data::permissions::{
    mdata_permissions_insert, mdata_permissions_new, USER_ANYONE,
};
use self::safe_app::ffi::mutable_data::{mdata_mutate_entries, mdata_put, ENTRIES_EMPTY};

#[cfg(feature = "use-mock-routing")]
use self::safe_app::ffi::test_utils::test_simulate_network_disconnect;
use self::safe_core::ffi::MDataInfo;
//use self::safe_core::ffi::arrays::{SymSecretKey, SymNonce};
use self::ffi_utils::test_utils::{call_0, call_1 /*, call_vec*/, call_vec_u8};
use self::safe_core::ffi::ipc::req::PermissionSet;
#[cfg(not(feature = "fake-auth"))]
use self::safe_core::ipc::{AppExchangeInfo, AuthReq, IpcReq};

#[cfg(not(feature = "fake-auth"))]
use std::collections::HashMap;
#[cfg(not(feature = "fake-auth"))]
use std::io::Read;
use std::{fmt, str};

use errors::{Error, ErrorCode, ResultReturn};

// TODO: these should be imported from safe_app::errors::codes
// but `errors` module is currently private
const ERR_DATA_EXISTS: i32 = -104;
const ERR_NO_SUCH_ENTRY: i32 = -106;

// URL where to send a GET request to the authenticator webservice for authorising the SAFE app
#[cfg(not(feature = "fake-auth"))]
const SAFE_AUTH_WEBSERVICE_BASE_URL: &str = "http://localhost:41805/authorise/";

use safe_net_helpers as SAFENetHelpers;

#[derive(Clone)]
pub struct MutableData(MDataInfo);

impl Default for MutableData {
    fn default() -> MutableData {
        MutableData(MDataInfo {
            name: Default::default(),
            type_tag: 0,
            has_enc_info: false,
            enc_key: Default::default(),
            enc_nonce: Default::default(),
            has_new_enc_info: false,
            new_enc_key: Default::default(),
            new_enc_nonce: Default::default(),
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConnStatus {
    Init,
    Disconnected,
    Connected,
    Failed,
}

impl fmt::Display for ConnStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                ConnStatus::Init => "Init",
                ConnStatus::Disconnected => "Disconnected",
                ConnStatus::Connected => "Connected",
                ConnStatus::Failed => "Failed",
            }
        )
    }
}

pub struct SAFENet {
    safe_app: Option<App>,
    conn_status: ConnStatus,
    sign_pub_key_h: SignPubKeyHandle,
}

impl SAFENet {
    // Generate an authorisation request string that can be sent to a SAFE Authenticator
    #[cfg(not(feature = "fake-auth"))]
    pub fn gen_auth_request(thing_id: &str) -> ResultReturn<String> {
        // TODO: allow the caller to provide the name and vendor strings
        let ipc_req = IpcReq::Auth(AuthReq {
            app: AppExchangeInfo {
                id: thing_id.to_string(),
                scope: None,
                name: "SAFEthing-".to_string() + thing_id,
                vendor: "SAFEthing Framework".to_string(),
            },
            app_container: false,
            containers: HashMap::new(),
        });

        match SAFENetHelpers::encode_ipc_msg(ipc_req) {
            Ok(auth_req_str) => {
                trace!(
                    "Authorisation request generated successfully: {}",
                    auth_req_str
                );
                let authenticator_webservice_url =
                    SAFE_AUTH_WEBSERVICE_BASE_URL.to_string() + &auth_req_str;
                let mut res = reqwest::get(&authenticator_webservice_url).unwrap();
                let mut auth_res = String::new();
                res.read_to_string(&mut auth_res).unwrap();
                debug!("Authorisation response: {}", auth_res);
                Ok(auth_res)
            }
            Err(e) => Err(Error::new(
                ErrorCode::InvalidArgument,
                format!("Failed encoding the auth URI: {:?}", e).as_str(),
            )),
        }
    }

    #[cfg(feature = "fake-auth")]
    pub fn gen_auth_request(thing_id: &str) -> ResultReturn<String> {
        Ok(thing_id.to_string())
    }

    // private helper function
    #[cfg(feature = "fake-auth")]
    fn register(&mut self, _: &str, _: &str) -> ResultReturn<()> {
        warn!("Using fake authorisation for testing...");
        self.safe_app = Some(create_app());
        self.conn_status = ConnStatus::Connected;
        Ok(())
    }

    // private helper function
    #[cfg(not(feature = "fake-auth"))]
    fn register(&mut self, app_id: &str, uri: &str) -> ResultReturn<()> {
        let disconnect_cb = || {
            //self.conn_status = ConnStatus::Disconnected;
            warn!("Connection with the SAFE Network was lost");
        };

        match SAFENetHelpers::decode_ipc_msg(&uri) {
            Ok(auth_granted) => {
                match App::registered(app_id.to_string(), auth_granted, disconnect_cb) {
                    Ok(app) => {
                        self.safe_app = Some(app);
                        self.conn_status = ConnStatus::Connected;
                        Ok(())
                    }
                    Err(e) => {
                        self.conn_status = ConnStatus::Failed;
                        Err(Error::new(
                            ErrorCode::ConnectionErr,
                            format!("Failed to connect to the SAFE Network: {:?}", e).as_str(),
                        ))
                    }
                }
            }
            Err(e) => Err(Error::new(
                ErrorCode::InvalidArgument,
                format!("Failed decoding the auth URI provided: {:?}", e).as_str(),
            )),
        }
    }

    // Connect to the SAFE Network using the provided app id and auth URI
    pub fn connect(app_id: &str, auth_uri: &str) -> ResultReturn<SAFENet> {
        let mut safe_net = SAFENet {
            safe_app: None,
            conn_status: ConnStatus::Init,
            sign_pub_key_h: Default::default(),
        };

        safe_net.register(&app_id, &auth_uri)?;

        // Retrieve app's public sign key
        let app: *const App = safe_net.safe_app.as_ref().unwrap();
        safe_net.sign_pub_key_h =
            unsafe { call_1(|ud, cb| app_pub_sign_key(app, ud, cb)).unwrap() };
        Ok(safe_net)
    }

    #[allow(dead_code)]
    pub fn get_conn_status(&self) -> &ConnStatus {
        &self.conn_status
    }

    pub fn gen_xor_name(&self, in_str: &str) -> [u8; 32] {
        let sha3 = unsafe {
            call_vec_u8(|ud, cb| sha3_hash(in_str.as_ptr(), in_str.len(), ud, cb)).unwrap()
        };
        let mut arr: [u8; 32] = Default::default();
        for i in 0..32 {
            arr[i] = sha3[i];
        }
        arr
    }

    pub fn new_pub_mutable_data(
        &self,
        xor_name: [u8; 32],
        type_tag: u64,
    ) -> ResultReturn<MutableData> {
        let app: *const App = self.safe_app.as_ref().unwrap();

        // Create permissions object
        let perms_h: MDataPermissionsHandle =
            unsafe { call_1(|ud, cb| mdata_permissions_new(app, ud, cb)).unwrap() };

        unsafe {
            // First set the permissions for the owner
            let perm_set = PermissionSet {
                read: true,
                insert: true,
                update: true,
                delete: true,
                manage_permissions: true,
            };
            call_0(|ud, cb| {
                mdata_permissions_insert(app, perms_h, self.sign_pub_key_h, &perm_set, ud, cb)
            })
            .unwrap();

            // Now add permissions for other apps/users
            let perm_set = PermissionSet {
                read: true,
                insert: true,
                update: false,
                delete: false,
                manage_permissions: false,
            };
            call_0(|ud, cb| mdata_permissions_insert(app, perms_h, USER_ANYONE, &perm_set, ud, cb))
                .unwrap();
        };

        // Create an empty public MD
        let md_info_pub = MDataInfo {
            name: xor_name,
            type_tag: type_tag,
            has_enc_info: false,
            enc_key: Default::default(),
            enc_nonce: Default::default(),
            has_new_enc_info: false,
            new_enc_key: Default::default(),
            new_enc_nonce: Default::default(),
        };

        unsafe {
            match call_0(|ud, cb| mdata_put(app, &md_info_pub, perms_h, ENTRIES_EMPTY, ud, cb)) {
                Ok(()) => Ok(MutableData(md_info_pub)),
                Err(error_code) => {
                    if error_code == ERR_DATA_EXISTS {
                        debug!("MutableData already exits");
                        Ok(MutableData(md_info_pub))
                    } else {
                        Err(Error::new(
                            ErrorCode::NetworkErr,
                            format!(
                                "Failed to commit MutableData to the SAFE Network: {:?}",
                                error_code
                            )
                            .as_str(),
                        ))
                    }
                }
            }
        }
    }

    pub fn get_pub_mutable_data(
        &self,
        xor_name: [u8; 32],
        type_tag: u64,
    ) -> ResultReturn<MutableData> {
        // Create a public MutableData object
        let md_info_pub = MDataInfo {
            name: xor_name,
            type_tag: type_tag,
            has_enc_info: false,
            enc_key: Default::default(),
            enc_nonce: Default::default(),
            has_new_enc_info: false,
            new_enc_key: Default::default(),
            new_enc_nonce: Default::default(),
        };

        Ok(MutableData(md_info_pub))
    }

    #[allow(dead_code)]
    pub fn new_priv_mutable_data(
        &self,
        _xor_name: [u8; 32],
        _type_tag: u64,
    ) -> ResultReturn<MutableData> {
        /*
        let mut ctx = CallbackContext::new(self.safe_app);
        let _ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_private(self.safe_app, &xor_name, type_tag, ctx_ptr, new_md_callback);
        };
        */
        Ok(Default::default())
    }

    pub fn mutable_data_get_value(&self, mdata: &MutableData, key: &str) -> ResultReturn<String> {
        let app = self.safe_app.as_ref().unwrap();
        trace!("Getting entry with key {}", key);
        match SAFENetHelpers::mdata_get(app, &mdata.0, key) {
            Ok((value, version)) => {
                let val = String::from_utf8(value).unwrap();
                trace!("Got entry (version {}) with value: {}", version, val);
                Ok(val)
            }
            Err(error_code) => {
                trace!("Entry not found with key {}", key);
                Err(Error::new(
                    ErrorCode::NetworkErr,
                    format!(
                        "Failed to retrieve value from MutableData: {:?}",
                        error_code
                    )
                    .as_str(),
                ))
            }
        }
    }

    pub fn mutable_data_set_value(
        &self,
        mdata: &MutableData,
        key: &str,
        value: &str,
    ) -> ResultReturn<()> {
        let app = self.safe_app.as_ref().unwrap();
        let mdata_actions_h: MDataEntryActionsHandle =
            unsafe { call_1(|ud, cb| mdata_entry_actions_new(app, ud, cb)).unwrap() };

        match SAFENetHelpers::mdata_get(app, &mdata.0, key) {
            Ok((v, version)) => {
                let str: String = String::from_utf8(v).unwrap();
                trace!(
                    "Entry already exists: '{}' => '{}' (version {})",
                    key,
                    str,
                    version
                );
                trace!(
                    "Let's update it with: '{}' (version {})",
                    value,
                    version + 1
                );
                unsafe {
                    call_0(|ud, cb| {
                        mdata_entry_actions_update(
                            app,
                            mdata_actions_h,
                            key.as_ptr(),
                            key.len(),
                            value.as_ptr(),
                            value.len(),
                            version + 1,
                            ud,
                            cb,
                        )
                    })
                    .unwrap();
                };
            }
            Err(error_code) => {
                if error_code == ERR_NO_SUCH_ENTRY {
                    trace!("Entry doesn't exist. Let's insert: '{}' '{}'", key, value);
                    unsafe {
                        call_0(|ud, cb| {
                            mdata_entry_actions_insert(
                                app,
                                mdata_actions_h,
                                key.as_ptr(),
                                key.len(),
                                value.as_ptr(),
                                value.len(),
                                ud,
                                cb,
                            )
                        })
                        .unwrap();
                    };
                } else {
                    return Err(Error::new(
                        ErrorCode::NetworkErr,
                        format!(
                            "Failed to retrieve value from MutableData: {:?}",
                            error_code
                        )
                        .as_str(),
                    ));
                }
            }
        }

        unsafe {
            call_0(|ud, cb| mdata_mutate_entries(app, &mdata.0, mdata_actions_h, ud, cb)).unwrap();
        };

        Ok(())
    }

    /// Retrieve the list of all entries from a MutableData
    pub fn mutable_data_get_entries(
        &self,
        mdata: &MutableData,
    ) -> ResultReturn<Vec<(String, String)>> {
        let app = self.safe_app.as_ref().unwrap();
        trace!("Getting entries from MutableData");
        match SAFENetHelpers::mdata_get_entries(app, &mdata.0) {
            Ok(entries) => {
                let entries_list = entries
                    .iter()
                    .map(|(key, value)| {
                        let k = String::from_utf8(key.to_vec()).unwrap();
                        let val = String::from_utf8(value.to_vec()).unwrap();
                        trace!("Got entry with key {}: and value: {}", k, val);
                        (k, val)
                    })
                    .collect();

                Ok(entries_list)
            }
            Err(error_code) => {
                trace!("Failed to retrieve list of entries, error {}", error_code);
                Err(Error::new(
                    ErrorCode::NetworkErr,
                    format!(
                        "Failed to retrieve entries from MutableData: {:?}",
                        error_code
                    )
                    .as_str(),
                ))
            }
        }
    }

    // The following functions are mainly utilities for developers
    #[cfg(feature = "use-mock-routing")]
    pub fn sim_net_disconnect(&mut self) {
        if cfg!(not(feature = "fake-auth")) {
            let app: *mut App = self.safe_app.as_mut().unwrap();
            unsafe {
                call_0(|ud, cb| test_simulate_network_disconnect(app, ud, cb)).unwrap();
            };
        } else {
            panic!("Function `sim_net_disconnect` is not available with `fake-auth` feature on");
        }
    }

    #[cfg(not(feature = "use-mock-routing"))]
    pub fn sim_net_disconnect(&mut self) {
        panic!(
            "Function `sim_net_disconnect` is only available with `use-mock-routing` feature on"
        );
    }
}
