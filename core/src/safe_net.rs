extern crate safe_app;
extern crate safe_core;
extern crate ffi_utils;
extern crate tokio_timer;
extern crate futures;
//extern crate futures_cpupool;

use self::safe_app::App;
#[cfg(feature="testing")]
use self::safe_app::test_utils::test_create_app;
use self::safe_app::object_cache::{MDataInfoHandle, MDataPermissionSetHandle,
    MDataPermissionsHandle, SignKeyHandle, MDataEntryActionsHandle};
use self::ffi_utils::FfiResult;
#[cfg(not(feature="testing"))]
use self::safe_app::ffi::app_registered;
#[cfg(not(feature="testing"))]
use self::safe_app::ffi::crypto::app_pub_sign_key;
use self::safe_app::ffi::crypto::sha3_hash;
#[cfg(not(feature="testing"))]
use self::safe_app::ffi::ipc::decode_ipc_msg;
use self::safe_app::ffi::mdata_info::{mdata_info_new_public, mdata_info_new_private};
use self::safe_app::ffi::mutable_data::permissions::{MDataAction, mdata_permission_set_new,
    mdata_permissions_set_allow, mdata_permissions_new, mdata_permissions_insert};
use self::safe_app::ffi::mutable_data::{mdata_put, mdata_mutate_entries, mdata_get_value};
use self::safe_app::ffi::mutable_data::entry_actions::{mdata_entry_actions_new,
    mdata_entry_actions_insert, mdata_entry_actions_update};
#[cfg(not(feature="testing"))]
use self::safe_core::ipc::resp::ffi::AuthGranted;
use self::tokio_timer::Timer;
use self::futures::Future;
//use self::futures_cpupool::CpuPool;

use errors::{ResultReturn, Error, ErrorCode};

use std::os::raw::{c_char, c_void};
use std::ptr::null_mut;
use std::ffi::{CString, CStr};
use std::slice;
use std::time::Duration;

fn get_error_str(c_buf: *const c_char) -> String {
    unsafe {
        CStr::from_ptr(c_buf).to_string_lossy().into_owned()
    }
}

// BEGIN: MutableData retrieve values callbacks
extern "C" fn mdata_get_value_callback(ctx_ptr: *mut c_void, result: FfiResult, value_ptr: *const u8, value_len: usize, version: u64) {
    if result.error_code == -106 {
        println!("MutableData entry not found: {} {}", result.error_code, get_error_str(result.description));
    } else {
        unsafe {
            let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
            let value = slice::from_raw_parts(value_ptr, value_len).to_vec();
            ctx.output.str_1 = String::from_utf8(value).unwrap();
            ctx.output.u64_1 = version;
            //println!("MutableData value retrieved {} - {}", ctx.output.str_1, ctx.output.u64_1);
        };
    }
}
// END: MutableData retrieve values callbacks

// BEGIN: Crypto helpers callbacks
extern "C" fn sha3_hash_callback(ctx_ptr: *mut c_void, result: FfiResult, value_ptr: *const u8, value_len: usize) {
    //println!("sha3: {}", result.error_code);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.output.str_1 = String::from_raw_parts(value_ptr as *mut _, value_len, value_len);
        //println!("sha3 generated {}", ctx.output.str_1);
    };
}
// END: Crypto helpers callbacks

// BEGIN: MutableData Mutations callbacks
extern "C" fn mdata_mutate_entries_callback(_: *mut c_void, result: FfiResult) {
    //println!("MutableData entry actions mutated {}", result.error_code);
    if result.error_code == -107 {
        //println!("MutableData entry already exits");
    }
}

extern "C" fn mdata_entry_actions_set_callback(ctx_ptr: *mut c_void, result: FfiResult) {
    //println!("MutableData entry actions set {}", result.error_code);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        mdata_mutate_entries(ctx.safe_app, ctx.mutable_data_h, ctx.entry_actions_h, ctx_ptr, mdata_mutate_entries_callback);
    };
}

extern "C" fn mdata_entry_actions_callback_2insert(ctx_ptr: *mut c_void, result: FfiResult, entry_actions_h: MDataEntryActionsHandle) {
    //println!("MutableData entry actions 2insert {} {}", result.error_code, entry_actions_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.entry_actions_h = entry_actions_h;
        let ref key = ctx.input.str_1;
        let ref value = ctx.input.str_2;
        mdata_entry_actions_insert(ctx.safe_app, ctx.entry_actions_h,
            key.as_ptr(), key.len(),
            value.as_ptr(), value.len(),
            ctx_ptr, mdata_entry_actions_set_callback);
    };
}

extern "C" fn mdata_entry_actions_callback_2update(ctx_ptr: *mut c_void, result: FfiResult, entry_actions_h: MDataEntryActionsHandle) {
    //println!("MutableData entry actions 2update {} {}", result.error_code, entry_actions_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.entry_actions_h = entry_actions_h;
        let ref key = ctx.input.str_1;
        let ref value = ctx.input.str_2;
        let version = ctx.input.u64_1 + 1;
        mdata_entry_actions_update(ctx.safe_app, ctx.entry_actions_h,
            key.as_ptr(), key.len(),
            value.as_ptr(), value.len(),
            version, ctx_ptr, mdata_entry_actions_set_callback);
    };
}
// END: MutableData Mutations callbacks


// BEGIN: MutableData creation/retrieval callbacks
extern "C" fn mdata_put_callback(_: *mut c_void, result: FfiResult) {
    //println!("MutableData put in network {}", result.error_code);
    if result.error_code == -104 {
        //println!("MutableData already exits");
    }
}

extern "C" fn perms_insert_callback(ctx_ptr: *mut c_void, result: FfiResult) {
    //println!("PermissionSet inserted in Permissions {}", result.error_code);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        mdata_put(ctx.safe_app, ctx.mutable_data_h, ctx.perms_h, 0, ctx_ptr, mdata_put_callback);
        // FIXME: provide en empty but valid Entries object handle
    };
}

extern "C" fn new_perms_callback(ctx_ptr: *mut c_void, result: FfiResult, perms_h: MDataPermissionsHandle) {
    //println!("Permissions created {} {}", result.error_code, perms_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.perms_h = perms_h;
        mdata_permissions_insert(ctx.safe_app, ctx.perms_h, ctx.sign_key_h, ctx.perm_set_h, ctx_ptr, perms_insert_callback);
    };
}

extern "C" fn perms_allow_update_callback(ctx_ptr: *mut c_void, result: FfiResult) {
    //println!("PermissionSet set with action update done {}", result.error_code);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        mdata_permissions_new(ctx.safe_app, ctx_ptr, new_perms_callback)
    };
}
extern "C" fn perms_allow_intert_callback(ctx_ptr: *mut c_void, result: FfiResult) {
    //println!("PermissionSet set with action insert done {}", result.error_code);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        let action = MDataAction::Update;
        mdata_permissions_set_allow(ctx.safe_app, ctx.perm_set_h, action, ctx_ptr, perms_allow_update_callback)
    };
}

extern "C" fn new_perm_set_callback(ctx_ptr: *mut c_void, result: FfiResult, perm_set_h: MDataPermissionSetHandle) {
    //println!("PermissionSet created {} {}", result.error_code, perm_set_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.perm_set_h = perm_set_h;
        let action = MDataAction::Insert;
        mdata_permissions_set_allow(ctx.safe_app, ctx.perm_set_h, action, ctx_ptr, perms_allow_intert_callback)
    };
}

extern "C" fn new_md_callback(ctx_ptr: *mut c_void, result: FfiResult, md_h: MDataInfoHandle) {
    //println!("MutableData created {} {}", result.error_code, md_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.mutable_data_h = md_h;
        mdata_permission_set_new(ctx.safe_app, ctx_ptr, new_perm_set_callback);
    };
}

extern "C" fn new_empty_perm_set_callback(ctx_ptr: *mut c_void, result: FfiResult, perm_set_h: MDataPermissionSetHandle) {
    //println!("PermissionSet empty created {} {}", result.error_code, perm_set_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.perm_set_h = perm_set_h;
        mdata_permissions_new(ctx.safe_app, ctx_ptr, new_perms_callback)
    };
}

extern "C" fn read_only_md_callback(ctx_ptr: *mut c_void, result: FfiResult, md_h: MDataInfoHandle) {
    //println!("MutableData for read only {} {}", result.error_code, md_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.mutable_data_h = md_h;
        mdata_permission_set_new(ctx.safe_app, ctx_ptr, new_empty_perm_set_callback);
    };
}
// END: MutableData callbacks

// BEGIN: Auth callbacks
#[allow(dead_code)]
extern "C" fn app_sign_key_callback(ctx_ptr: *mut c_void, result: FfiResult, sign_key_h: SignKeyHandle) {
    //println!("App's pub sign key retrieved {} {}", result.error_code, sign_key_h);
    unsafe {
        let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
        ctx.sign_key_h = sign_key_h;
    };
}

#[cfg(not(feature="testing"))]
extern "C" fn callback(_: *mut c_void, err: i32, state: i32) {
    //println!("App registered {} {}", err, state);
}

#[cfg(not(feature="testing"))]
extern "C" fn auth_cb(ctx_ptr: *mut c_void, err: u32, auth_granted: *const AuthGranted) {
   //println!("App was authorised {}", err);
   let app_id = CString::new("net.safe_thing.framework.id").unwrap();
   unsafe {
       let ctx: &mut CallbackContext = &mut *(ctx_ptr as *mut CallbackContext);
       let r = app_registered(app_id.as_ptr(), auth_granted, ctx_ptr, callback, &mut ctx.safe_app);
       //println!("Registering app {}", r);
       app_pub_sign_key(ctx.safe_app, ctx_ptr, app_sign_key_callback);
   };
}

#[cfg(not(feature="testing"))]
extern "C" fn containers_cb(_: *mut c_void, a: u32) {
    println!("containers callback {}", a);
}

#[cfg(not(feature="testing"))]
extern "C" fn revoked_cb(_: *mut c_void) {
    println!("app revoked");
}

#[cfg(not(feature="testing"))]
extern "C" fn error_cb(_: *mut c_void, result: FfiResult, b: u32) {
    println!("error {} {}", result.error_code, b);
}
// END: Auth callbacks


// START: Callback Utilities
type MDEntryValue = (String, u64);

struct ContextInput {
    str_1: String,
    str_2: String,
    u64_1: u64
}

struct ContextOutput {
    str_1: String,
    u64_1: u64
}

// FIXME: free the underlying objects
struct CallbackContext {
    safe_app: *const App,
    sign_key_h: SignKeyHandle,
    mutable_data_h: MDataInfoHandle,
    perm_set_h: MDataPermissionSetHandle,
    perms_h: MDataPermissionsHandle,
    entry_actions_h: MDataEntryActionsHandle,
    input: ContextInput,
    output: ContextOutput
}

impl CallbackContext {
    pub fn new(safe_app: *const App) -> CallbackContext {
        CallbackContext {
            safe_app: safe_app,
            sign_key_h: Default::default(),
            mutable_data_h: Default::default(),
            perm_set_h: Default::default(),
            perms_h: Default::default(),
            entry_actions_h: Default::default(),
            input: ContextInput {
                str_1: Default::default(),
                str_2: Default::default(),
                u64_1: Default::default()
            },
            output: ContextOutput {
                str_1: Default::default(),
                u64_1: Default::default()
            }
        }
    }
}
// END: Callback Utilities


pub type MutableData = MDataInfoHandle; //FIXME: free the underlying object

#[derive(Debug)]
enum ConnStatus {
    Init,
    Unregistered,
    Registered,
}

pub struct SAFENet {
    safe_app: *mut App,
    conn_status: ConnStatus,
    sign_key_h: SignKeyHandle
}

impl SAFENet {

    // FIXME: this is temporary until futures are implemented
    fn sleep() {
        let timer = Timer::default();
        let _ = timer.sleep(Duration::from_millis(1000))
            .wait();
    }

    // private helper function
    #[cfg(feature = "testing")]
    fn register(safe_net: &mut SAFENet, _: CString) {
        println!("Using fake authorisation for testing...");
        test_create_app(&mut safe_net.safe_app);
        /* FIXME
        let mut ctx = CallbackContext::new(safe_net.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            app_pub_sign_key(safe_net.safe_app, ctx_ptr, app_sign_key_callback);
        };
        SAFENet::sleep();
        safe_net.sign_key_h = ctx.sign_key_h;*/
    }
    #[cfg(not(feature = "testing"))]
    fn register(safe_net: &mut SAFENet, uri: CString) {
        let mut ctx = CallbackContext::new(safe_net.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            decode_ipc_msg(uri.as_ptr(), ctx_ptr, auth_cb, containers_cb, revoked_cb, error_cb);
        };
        SAFENet::sleep();
        safe_net.sign_key_h = ctx.sign_key_h;
    }

    pub fn connect(auth_uri: &str) -> ResultReturn<SAFENet> {
        let mut safe_net = SAFENet {
            safe_app: null_mut(),
            conn_status: ConnStatus::Init,
            sign_key_h: Default::default()
        };

        let uri: CString;
        match CString::new(auth_uri) {
            Ok(v) => uri = v,
            Err(e) => {
                safe_net.conn_status = ConnStatus::Unregistered;
                return Err(Error::new(ErrorCode::InvalidParameters,
                                                format!("Auth URI is invalid: {}", e).as_str()));
            }
        };

        SAFENet::register(&mut safe_net, uri);
        safe_net.conn_status = ConnStatus::Registered;
        Ok(safe_net)
    }

    #[allow(dead_code)]
    pub fn get_conn_status(&self) {
        match self.conn_status {
            ConnStatus::Init => println!("CONN STATUS: Init"),
            ConnStatus::Unregistered => println!("CONN STATUS: Unregistered"),
            ConnStatus::Registered => println!("CONN STATUS: Registered"),
        }
    }

    #[allow(dead_code)]
    pub fn hash_string(in_str: &str) -> ResultReturn<String> {
        let mut ctx = CallbackContext::new(null_mut());
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            sha3_hash(in_str.as_ptr(), in_str.len(), ctx_ptr, sha3_hash_callback);
        };

        SAFENet::sleep();
        Ok(ctx.output.str_1)
    }

    pub fn new_pub_mutable_data(&self, xor_name: [u8; 32], type_tag: u64) -> ResultReturn<MutableData> {
        let mut ctx = CallbackContext::new(self.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_public(self.safe_app, &xor_name, type_tag, ctx_ptr, new_md_callback);
        };

        SAFENet::sleep();
        Ok(ctx.mutable_data_h)
    }

    pub fn get_pub_mutable_data(&self, xor_name: [u8; 32], type_tag: u64) -> ResultReturn<MutableData> {
        let mut ctx = CallbackContext::new(self.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_public(self.safe_app, &xor_name, type_tag, ctx_ptr, read_only_md_callback);
        };

        SAFENet::sleep();
        Ok(ctx.mutable_data_h)
    }

    #[allow(dead_code)]
    pub fn new_priv_mutable_data(&self, xor_name: [u8; 32], type_tag: u64) -> ResultReturn<MutableData> {
        let mut ctx = CallbackContext::new(self.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            mdata_info_new_private(self.safe_app, &xor_name, type_tag, ctx_ptr, new_md_callback);
        };

        SAFENet::sleep();
        Ok(ctx.mutable_data_h)
    }

    // private helper function
    fn helper_md_get_value(&self, mdata: MutableData, key: &str) -> ResultReturn<MDEntryValue> {
        let mut ctx = CallbackContext::new(self.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        unsafe {
            mdata_get_value(self.safe_app, mdata, key.as_ptr(), key.len(), ctx_ptr, mdata_get_value_callback);
        };
        SAFENet::sleep();
        Ok((ctx.output.str_1, ctx.output.u64_1))
    }

    pub fn mutable_data_get_value(&self, mdata: MutableData, key: &str) -> ResultReturn<String> {
        let entry_value = self.helper_md_get_value(mdata, key)?;
        Ok(entry_value.0)
    }

    pub fn mutable_data_set_value(&self, mdata: MutableData, key: &str, value: &str) -> ResultReturn<()> {
        let mut ctx = CallbackContext::new(self.safe_app);
        let ctx_ptr = &mut ctx as *mut _ as *mut c_void;
        ctx.input.str_1 = String::from(key);
        ctx.input.str_2 = String::from(value);
        unsafe {
            let current_value = self.helper_md_get_value(mdata, key)?;
            if current_value.0.is_empty() {
                mdata_entry_actions_new(self.safe_app, ctx_ptr, mdata_entry_actions_callback_2insert);
            } else {
                ctx.input.u64_1 = current_value.1;
                mdata_entry_actions_new(self.safe_app, ctx_ptr, mdata_entry_actions_callback_2update);
            }
            SAFENet::sleep();
        };
        Ok(())
    }
}
