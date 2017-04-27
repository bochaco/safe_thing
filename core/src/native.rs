extern crate safe_app;
extern crate safe_core;
extern crate tokio_timer;
extern crate futures;
//extern crate futures_cpupool;

use self::safe_app::App;
#[cfg(feature="testing")]
use self::safe_app::test_utils::test_create_app;
use self::safe_app::object_cache::{MDataInfoHandle, MDataPermissionSetHandle,
    MDataPermissionsHandle, SignKeyHandle, MDataEntryActionsHandle};
use self::safe_app::ffi::app_registered;
use self::safe_app::ffi::crypto::app_pub_sign_key;
use self::safe_app::ffi::ipc::decode_ipc_msg;
use self::safe_app::ffi::mdata_info::{mdata_info_new_public, mdata_info_new_private};
use self::safe_app::ffi::mutable_data::permissions::{MDataAction, mdata_permission_set_new,
    mdata_permissions_set_allow, mdata_permissions_new, mdata_permissions_insert};
use self::safe_app::ffi::mutable_data::{mdata_put, mdata_mutate_entries, mdata_get_value};
use self::safe_app::ffi::mutable_data::entry_actions::{mdata_entry_actions_new, mdata_entry_actions_insert};
use self::safe_core::ipc::resp::ffi::AuthGranted;
use self::tokio_timer::Timer;
use self::futures::Future;
//use self::futures_cpupool::CpuPool;

use errors::{ResultReturn, Error, ErrorCode};

use std::os::raw::{c_char, c_void, c_int};
use std::ptr::null_mut;
use std::ffi::CString;
use std::slice;
use std::time::Duration;


// BEGIN: MutableData retrieve values callbacks
extern "C" fn mdata_get_value_callback(safe_thing_c_void: *mut c_void, err: i32, value_ptr: *const u8, value_len: usize, version: u64) {
    unsafe {
        let value = slice::from_raw_parts(value_ptr, value_len).to_vec();
        let v = String::from_utf8(value).unwrap();
        println!("MutableData value retrieved {} {:?}", err, v);
    };
}
// END: MutableData retrieve values callbacks
/*
// BEGIN: MutableData Mutations callbacks
extern "C" fn mdata_mutate_entries_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("MutableData entry actions mutated {}", err);
    if err == -107 {
        println!("MutableData entry already exits");
    }
}

extern "C" fn mdata_entry_actions_insert_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("MutableData entry actions insert {}", err);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        mdata_mutate_entries(safe_net.safe_app, safe_net.mutable_data_h, safe_net.entry_actions_h, safe_thing_c_void, mdata_mutate_entries_callback);
    };
}

extern "C" fn mdata_entry_actions_callback(safe_thing_c_void: *mut c_void, err: i32, entry_actions_h: MDataEntryActionsHandle) {
    println!("MutableData entry actions {} {}", err, entry_actions_h);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        safe_net.entry_actions_h = entry_actions_h;
        let key = SAFEthing_ENTRY_APP_STATUS;
        let value = safe_net.status.clone();
        mdata_entry_actions_insert(safe_net.safe_app, safe_net.entry_actions_h,
            key.as_ptr(), key.len(),
            value.as_ptr(), value.len(),
            safe_thing_c_void, mdata_entry_actions_insert_callback);
    };
}
// END: MutableData Mutations callbacks
*/

// BEGIN: MutableData creation/retrieval callbacks
extern "C" fn mdata_put_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("MutableData put in network {}", err);
    if err == -104 {
        println!("MutableData already exits");
    }
}

extern "C" fn perms_insert_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("PermissionSet inserted in Permissions {}", err);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        mdata_put(safe_net.safe_app, safe_net.mutable_data_h, safe_net.perms_h, 0, safe_thing_c_void, mdata_put_callback);
        // FIXME: provide en empty but valid Entries object handle
    };
}

extern "C" fn new_perms_callback(safe_thing_c_void: *mut c_void, err: i32, perms_h: MDataPermissionsHandle) {
    println!("Permissions created {} {}", err, perms_h);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        safe_net.perms_h = perms_h;
        mdata_permissions_insert(safe_net.safe_app, safe_net.perms_h, safe_net.sign_key_h, safe_net.perm_set_h, safe_thing_c_void, perms_insert_callback);
    };
}

extern "C" fn perms_allow_update_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("PermissionSet set with action update done {}", err);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        mdata_permissions_new(safe_net.safe_app, safe_thing_c_void, new_perms_callback)
    };
}
extern "C" fn perms_allow_intert_callback(safe_thing_c_void: *mut c_void, err: i32) {
    println!("PermissionSet set with action insert done {}", err);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        let action = MDataAction::Update;
        mdata_permissions_set_allow(safe_net.safe_app, safe_net.perm_set_h, action, safe_thing_c_void, perms_allow_update_callback)
    };
}

extern "C" fn new_perm_set_callback(safe_thing_c_void: *mut c_void, err: i32, perm_set_h: MDataPermissionSetHandle) {
    println!("PermissionSet created {} {}", err, perm_set_h);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        safe_net.perm_set_h = perm_set_h;
        let action = MDataAction::Insert;
        mdata_permissions_set_allow(safe_net.safe_app, safe_net.perm_set_h, action, safe_thing_c_void, perms_allow_intert_callback)
    };
}

extern "C" fn new_md_callback(safe_thing_c_void: *mut c_void, err: i32, md_h: MDataInfoHandle) {
    println!("MutableData created {} {}", err, md_h);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        safe_net.mutable_data_h = md_h;
        mdata_permission_set_new(safe_net.safe_app, safe_thing_c_void, new_perm_set_callback);
    };
}
// END: MutableData callbacks

// BEGIN: Auth callbacks
extern "C" fn app_sign_key_callback(safe_thing_c_void: *mut c_void, err: i32, sign_key_h: SignKeyHandle) {
    println!("App's pub sign key retrieved {} {}", err, sign_key_h);
    unsafe {
        let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
        safe_net.sign_key_h = sign_key_h;
    };
}

extern "C" fn callback(user_data: *mut c_void, err: i32, state: i32) {
    println!("App registered {} {}", err, state);
}

extern "C" fn auth_cb(safe_thing_c_void: *mut c_void, err: u32, auth_granted: *const AuthGranted) {
   println!("App was authorised {}", err);
   let app_id = CString::new("net.safe_thing.framework.id").unwrap();
   unsafe {
       let safe_net: &mut SAFENet = &mut *(safe_thing_c_void as *mut SAFENet);
       let r = app_registered(app_id.as_ptr(), auth_granted, safe_thing_c_void, callback, &mut safe_net.safe_app);
       println!("Registering app {}", r);
       safe_net.conn_status = ConnStatus::REGISTERED;
       app_pub_sign_key(safe_net.safe_app, safe_thing_c_void, app_sign_key_callback);
   };
}

extern "C" fn containers_cb(user_data: *mut c_void, a: u32) {
    println!("containers callback {}", a);
}

extern "C" fn revoked_cb(user_data: *mut c_void) {
    println!("app revoked");
}

extern "C" fn error_cb(user_data: *mut c_void, err: i32, b: u32) {
    println!("error {} {}", err, b);
}
// END: Auth callbacks


#[derive(Debug)]
enum ConnStatus {
    INIT,
    UNREGISTERED,
    REGISTERED,
}

pub struct SAFENet {
    safe_app: *mut App,
    conn_status: ConnStatus,
    sign_key_h: SignKeyHandle,
    mutable_data_h: MDataInfoHandle,
    perm_set_h: MDataPermissionSetHandle,
    perms_h: MDataPermissionsHandle,
    entry_actions_h: MDataEntryActionsHandle
}

impl SAFENet {

    // FIXME: this is temporary until futures are implemented
    fn sleep(m: u64) {
        let timer = Timer::default();
        let _ = timer.sleep(Duration::from_millis(m))
            .wait();
    }

    #[cfg(feature = "testing")]
    fn register(safe_net: &mut SAFENet, _: CString) {
        println!("Using fake authorisation for testing...");
        test_create_app(&mut safe_net.safe_app);
    }
    #[cfg(not(feature = "testing"))]
    fn register(safe_net: &mut SAFENet, uri: CString) {
        let safe_net_c_void_ptr = safe_net as *mut _ as *mut c_void;
        unsafe {
            decode_ipc_msg(uri.as_ptr(), safe_net_c_void_ptr, auth_cb, containers_cb, revoked_cb, error_cb);
        };
        SAFENet::sleep(2000);
    }

    pub fn connect(auth_uri: &str) -> ResultReturn<SAFENet> {
        let mut safe_net = SAFENet {
            safe_app: null_mut(),
            conn_status: ConnStatus::INIT,
            sign_key_h: Default::default(),
            mutable_data_h: Default::default(),
            perm_set_h: Default::default(),
            perms_h: Default::default(),
            entry_actions_h: Default::default()
        };

        let mut uri: CString = Default::default();
        match CString::new(auth_uri) {
            Ok(v) => uri = v,
            Err(e) => return Err(Error::new(ErrorCode::InvalidParameters,
                                                format!("Auth token is invalid: {}", e).as_str()))
        };

        SAFENet::register(&mut safe_net, uri);

        Ok(safe_net)
    }

    #[allow(dead_code)]
    pub fn get_conn_status(&self) {
        match self.conn_status {
            ConnStatus::INIT => println!("CONN STATUS: INIT"),
            ConnStatus::UNREGISTERED => println!("CONN STATUS: UNREGISTERED"),
            ConnStatus::REGISTERED => println!("CONN STATUS: REGISTERED"),
        }
    }

    pub fn new_pub_mutable_data(&mut self, xor_name: [u8; 32], type_tag: u64) {
        let safe_net_c_void_ptr = self as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_public(self.safe_app, &xor_name, type_tag, safe_net_c_void_ptr, new_md_callback);
        };

        SAFENet::sleep(2000);
    }

    pub fn new_priv_mutable_data(&mut self, xor_name: [u8; 32], type_tag: u64) {
        let safe_net_c_void_ptr = self as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_private(self.safe_app, &xor_name, type_tag, safe_net_c_void_ptr, new_md_callback);
        };

        SAFENet::sleep(2000);
    }

}
