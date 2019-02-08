extern crate safe_app;
extern crate safe_core;
extern crate ffi_utils;
//extern crate routing;

use self::safe_app::App;
#[cfg(feature="fake-auth")]
use self::safe_app::test_utils::create_app;
use self::safe_app::ffi::object_cache::{MDataPermissionsHandle,
    SignPubKeyHandle, MDataEntryActionsHandle};
use self::safe_app::ffi::crypto::{app_pub_sign_key, sha3_hash};
//use self::safe_app::ffi::mdata_info::{mdata_info_new_private};
use self::safe_app::ffi::mutable_data::permissions::{mdata_permissions_new,
    mdata_permissions_insert/*, USER_ANYONE, MDataAction*/};
use self::safe_app::ffi::mutable_data::{ENTRIES_EMPTY, mdata_put, mdata_mutate_entries};
use self::safe_app::ffi::mutable_data::entry_actions::{mdata_entry_actions_new,
    mdata_entry_actions_insert, mdata_entry_actions_update};

use self::safe_core::ffi::MDataInfo;
//use self::safe_core::ffi::arrays::{SymSecretKey, SymNonce};
use self::safe_core::ffi::ipc::req::{PermissionSet};
use self::ffi_utils::test_utils::{call_0, call_1/*, call_vec*/, call_vec_u8};

use std::{fmt, str};

use errors::{ResultReturn, Error, ErrorCode};
const ERR_DATA_EXISTS: i32 = -104;
const ERR_NO_SUCH_ENTRY: i32 = -106;

use safe_net_helpers as SAFENetHelpers;

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
            new_enc_nonce: Default::default()
        })
    }
}

#[derive(Debug)]
pub enum ConnStatus {
    Init,
    //Disconnected,
    Connected,
}

impl fmt::Display for ConnStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            ConnStatus::Init => "Init",
            //ConnStatus::Disconnected => "Disconnected",
            ConnStatus::Connected => "Connected",
        })
    }
}

pub struct SAFENet {
    safe_app: Option<App>,
    conn_status: ConnStatus,
    sign_pub_key_h: SignPubKeyHandle
}

impl SAFENet {
    // private helper function
    #[cfg(feature = "fake-auth")]
    fn register(&mut self, _: &str, _: &str) -> ResultReturn<()> {
        println!("Using fake authorisation for testing...");
        self.safe_app = Some(create_app());
        self.conn_status = ConnStatus::Connected;
        Ok(())
    }

    #[cfg(not(feature = "fake-auth"))]
    fn register(&mut self, app_id: &str, uri: &str) -> ResultReturn<()> {
        match SAFENetHelpers::decode_ipc_msg(&uri) {
            Ok(auth_granted) => {
                match App::registered(String::from(app_id), auth_granted, || {
                        //self.conn_status = ConnStatus::Disconnected;
                        println!("Connection with the SAFE network was lost");
                    }) {
                    Ok(app) => {
                        self.safe_app = Some(app);
                        self.conn_status = ConnStatus::Connected;
                        Ok(())
                    },
                    Err(e) => Err(Error::new(ErrorCode::ConnectionErr,
                                format!("Failed to connect to the SAFE network: {:?}", e).as_str()))
                }
            },
            Err(e) => Err(Error::new(ErrorCode::InvalidParameters,
                        format!("Failed decoding the auth URI provided: {:?}", e).as_str()))
        }
    }

    // Connect to the SAFE Network using the provided app id and auth URI
    pub fn connect(app_id: &str, auth_uri: &str) -> ResultReturn<SAFENet> {
        let mut safe_net = SAFENet {
            safe_app: None,
            conn_status: ConnStatus::Init,
            sign_pub_key_h: Default::default()
        };

        safe_net.register(&app_id, &auth_uri)?;

        // Retrieve app's public sign key
        let app: *const App = safe_net.safe_app.as_ref().unwrap();
        safe_net.sign_pub_key_h = unsafe {
            call_1(|ud, cb| app_pub_sign_key(app, ud, cb)).unwrap()
        };
        Ok(safe_net)
    }

    #[allow(dead_code)]
    pub fn get_conn_status(&self) -> &ConnStatus {
        &self.conn_status
    }

    #[allow(dead_code)]
    pub fn gen_xor_name(&self, in_str: &str) -> [u8; 32] {
        let sha3 = unsafe {
            call_vec_u8(
                |ud, cb| sha3_hash(in_str.as_ptr(), in_str.len(), ud, cb),
            ).unwrap()
        };
        let mut arr: [u8; 32] = Default::default();
        for i in 0..32 {
            arr[i] = sha3[i];
        }
        arr
    }

    pub fn new_pub_mutable_data(&self, xor_name: [u8; 32], type_tag: u64) -> ResultReturn<MutableData> {
        let app: *const App = self.safe_app.as_ref().unwrap();
        let perm_set = PermissionSet {
            read: true,
            insert: true,
            update: true,
            delete: true,
            manage_permissions: true
        };

        // Create permissions
        let perms_h: MDataPermissionsHandle =
            unsafe { call_1(|ud, cb| mdata_permissions_new(app, ud, cb)).unwrap() };

        unsafe {
            call_0(|ud, cb| {
                mdata_permissions_insert(app, perms_h, self.sign_pub_key_h, &perm_set, ud, cb)
            }).unwrap();
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
            new_enc_nonce: Default::default()
        };

        unsafe {
            match call_0(|ud, cb| {
                mdata_put(app, &md_info_pub, perms_h, ENTRIES_EMPTY, ud, cb)
            }) {
                Ok(()) => Ok(MutableData(md_info_pub)),
                Err(error_code) => {
                    if error_code == ERR_DATA_EXISTS {
                        println!("MutableData already exits");
                        Ok(MutableData(md_info_pub))
                    } else {
                        Err(Error::new(ErrorCode::NetworkErr,
                            format!("Failed to commit MutableData to the SAFE network: {:?}", error_code).as_str()))
                    }
                }
            }
        }
    }

    pub fn get_pub_mutable_data(&self, xor_name: [u8; 32], type_tag: u64) -> ResultReturn<MutableData> {
        // Create a public MutableData object
        let md_info_pub = MDataInfo {
            name: xor_name,
            type_tag: type_tag,
            has_enc_info: false,
            enc_key: Default::default(),
            enc_nonce: Default::default(),
            has_new_enc_info: false,
            new_enc_key: Default::default(),
            new_enc_nonce: Default::default()
        };

        Ok(MutableData(md_info_pub))
    }

    #[allow(dead_code)]
    pub fn new_priv_mutable_data(&self, _xor_name: [u8; 32], _type_tag: u64) -> ResultReturn<MutableData> {
//        let mut ctx = CallbackContext::new(self.safe_app);
//        let _ctx_ptr = &mut ctx as *mut _ as *mut c_void;
//        unsafe {
//            mdata_info_new_private(self.safe_app, &xor_name, type_tag, ctx_ptr, new_md_callback);
//        };

        Ok(Default::default())
    }

    pub fn mutable_data_get_value(&self, mdata: &MutableData, key: &str) -> ResultReturn<String> {
        let app = self.safe_app.as_ref().unwrap();
        match SAFENetHelpers::mdata_get(app, &mdata.0, key) {
            Ok((value, _)) => Ok(String::from_utf8(value).unwrap()),
            Err(error_code) => {
                return Err(Error::new(ErrorCode::NetworkErr,
                    format!("Failed to retrieve value from MutableData: {:?}", error_code).as_str()))
            }
        }
    }

    pub fn mutable_data_set_value(&self, mdata: &MutableData, key: &str, value: &str) -> ResultReturn<()> {
        let app = self.safe_app.as_ref().unwrap();
        let mdata_actions_h: MDataEntryActionsHandle = unsafe { call_1(|ud, cb| mdata_entry_actions_new(app, ud, cb)).unwrap() };

        match SAFENetHelpers::mdata_get(app, &mdata.0, key) {
            Ok((v, version)) => {
                let str: String = String::from_utf8(v).unwrap();
                println!("Entry already exists: '{}' => '{}' (version {})", key, str, version);
                println!("Let's update it with: '{}' (version {})", value, version + 1);
                unsafe {
                    call_0(|ud, cb| {
                        mdata_entry_actions_update(app, mdata_actions_h, key.as_ptr(), key.len(),
                                                    value.as_ptr(), value.len(), version + 1, ud, cb)
                    }).unwrap();
                }
            },
            Err(error_code) => {
                if error_code == ERR_NO_SUCH_ENTRY {
                    println!("Entry doesn't exist");
                    println!("Let's insert: '{}' '{}'", key, value);
                    unsafe {
                        call_0(|ud, cb| {
                            mdata_entry_actions_insert(app, mdata_actions_h, key.as_ptr(), key.len(),
                                                        value.as_ptr(), value.len(), ud, cb)
                        }).unwrap();
                    }
                } else {
                    return Err(Error::new(ErrorCode::NetworkErr,
                        format!("Failed to retrieve value from MutableData: {:?}", error_code).as_str()))
                }
            }
        }

        unsafe {
            call_0(|ud, cb| {
                mdata_mutate_entries(app, &mdata.0, mdata_actions_h, ud, cb)
            }).unwrap()
        }

        Ok(())
    }
}
