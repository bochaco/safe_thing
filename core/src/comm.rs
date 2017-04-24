extern crate safe_app;
extern crate safe_core;
extern crate rust_sodium;
extern crate rustc_serialize;

use self::safe_app::App;
use self::safe_app::object_cache::{MDataInfoHandle, MDataPermissionSetHandle,
    MDataPermissionsHandle, SignKeyHandle, MDataEntryActionsHandle};
use self::safe_app::ffi::app_registered;
use self::safe_app::ffi::ipc::decode_ipc_msg;
use self::safe_app::ffi::mdata_info::mdata_info_new_public;
use self::safe_app::ffi::mutable_data::permissions::{MDataAction, mdata_permission_set_new,
    mdata_permissions_set_allow, mdata_permissions_new, mdata_permissions_insert};
use self::safe_app::ffi::mutable_data::{mdata_put, mdata_mutate_entries};
use self::safe_app::ffi::mutable_data::entry_actions::{mdata_entry_actions_new, mdata_entry_actions_insert};
use self::safe_app::ffi::misc::app_pub_sign_key;
use self::safe_core::ipc::resp::ffi::AuthGranted;
use self::rust_sodium::crypto::hash::sha256;
use self::rust_sodium::crypto::hash::sha256::Digest;
use self::rustc_serialize::base64::{CharacterSet, Config, FromBase64, FromBase64Error, Newline, ToBase64};

use errors::{ResultReturn, Error, ErrorCode};
use std::collections::BTreeMap;
use std::os::raw::{c_char, c_void, c_int};
use std::ptr::null_mut;
use std::ffi::CString;

static SAFE_o_T_ENTRY_APP_STATUS: &'static str = "_safeot_app_status";


pub type ActionArgs = Vec<String>; // the values are opaque for the framework

#[derive(Debug)]
enum ConnStatus {
    INIT,
    UNREGISTERED,
    REGISTERED,
}

#[derive(Debug)]
pub struct SAFEoTComm {
    thing_id: String,
    conn_status: ConnStatus,
    safe_app: *mut App,
    sign_key_h: SignKeyHandle,
    mutable_data_h: MDataInfoHandle,
    perm_set_h: MDataPermissionSetHandle,
    perms_h: MDataPermissionsHandle,
    entry_actions_h: MDataEntryActionsHandle,
    xor_name: String,

    // the following is temporary, we keep this in the safenet
    status: String,
    attrs: String,
    topics: String,
    actions: String,
    subscriptions: String,
    topic_events: BTreeMap<String, String>
}

// BEGIN: MutableData Mutations callbacks
extern "C" fn mdata_mutate_entries_callback(safeot_c_void: *mut c_void, err: i32) {
    println!("MutableData entry actions mutated {}", err);
}

extern "C" fn mdata_entry_actions_insert_callback(safeot_c_void: *mut c_void, err: i32) {
    println!("MutableData entry actions insert {}", err);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        mdata_mutate_entries(safeot.safe_app, safeot.mutable_data_h, safeot.entry_actions_h, safeot_c_void, mdata_mutate_entries_callback);
    };
}

extern "C" fn mdata_entry_actions_callback(safeot_c_void: *mut c_void, err: i32, entry_actions_h: MDataEntryActionsHandle) {
    println!("MutableData entry actions {} {}", err, entry_actions_h);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        safeot.entry_actions_h = entry_actions_h;
        let key = SAFE_o_T_ENTRY_APP_STATUS;
        let value = safeot.status.clone();
        mdata_entry_actions_insert(safeot.safe_app, safeot.entry_actions_h,
            key.as_ptr(), key.len(),
            value.as_ptr(), value.len(),
            safeot_c_void, mdata_entry_actions_insert_callback);
    };
}
// END: MutableData Mutations callbacks


// BEGIN: MutableData callbacks
extern "C" fn mdata_put_callback(safeot_c_void: *mut c_void, err: i32) {
    println!("MutableData put in network {}", err);
}

extern "C" fn perms_insert_callback(safeot_c_void: *mut c_void, err: i32) {
    println!("PermissionSet inserted in Permissions {}", err);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        mdata_put(safeot.safe_app, safeot.mutable_data_h, safeot.perms_h, 0, safeot_c_void, mdata_put_callback);
        // FIXME: provide en empty but valid Entries object handle
    };
}

extern "C" fn new_perms_callback(safeot_c_void: *mut c_void, err: i32, perms_h: MDataPermissionsHandle) {
    println!("Permissions created {} {}", err, perms_h);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        safeot.perms_h = perms_h;
        mdata_permissions_insert(safeot.safe_app, safeot.perms_h, safeot.sign_key_h, safeot.perm_set_h, safeot_c_void, perms_insert_callback);
    };
}

extern "C" fn perms_set_action_callback2(safeot_c_void: *mut c_void, err: i32) {
    println!("PermissionSet set with action update done {}", err);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        mdata_permissions_new(safeot.safe_app, safeot_c_void, new_perms_callback)
    };
}
extern "C" fn perms_set_action_callback(safeot_c_void: *mut c_void, err: i32) {
    println!("PermissionSet set with action insert done {}", err);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        let action = MDataAction::Update;
        mdata_permissions_set_allow(safeot.safe_app, safeot.perm_set_h, action, safeot_c_void, perms_set_action_callback2)
    };
}

extern "C" fn new_perm_set_callback(safeot_c_void: *mut c_void, err: i32, perm_set_h: MDataPermissionSetHandle) {
    println!("PermissionSet created {} {}", err, perm_set_h);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        safeot.perm_set_h = perm_set_h;
        let action = MDataAction::Insert;
        mdata_permissions_set_allow(safeot.safe_app, safeot.perm_set_h, action, safeot_c_void, perms_set_action_callback)
    };
}

extern "C" fn new_md_callback(safeot_c_void: *mut c_void, err: i32, md_h: MDataInfoHandle) {
    println!("MutableData created {} {}", err, md_h);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        safeot.mutable_data_h = md_h;
        mdata_permission_set_new(safeot.safe_app, safeot_c_void, new_perm_set_callback);
    };
}
// END: MutableData callbacks

// BEGIN: Auth callbacks
extern "C" fn app_sign_key_callback(safeot_c_void: *mut c_void, err: i32, sign_key_h: SignKeyHandle) {
    println!("App's pub sign key retrieved {} {}", err, sign_key_h);
    unsafe {
        let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
        safeot.sign_key_h = sign_key_h;
    };
}

extern "C" fn callback(user_data: *mut c_void, err: i32, state: i32) {
    println!("App registered {} {}", err, state);
}

extern "C" fn auth_cb(safeot_c_void: *mut c_void, err: u32, auth_granted: *const AuthGranted) {
   println!("App was authorised {}", err);
   let app_id = CString::new("net.safeot.framework.id").unwrap();
   unsafe {
       let safeot: &mut SAFEoTComm = &mut *(safeot_c_void as *mut SAFEoTComm);
       let r = app_registered(app_id.as_ptr(), auth_granted, safeot_c_void, callback, &mut safeot.safe_app);
       println!("Registering app {}", r);
       safeot.conn_status = ConnStatus::REGISTERED;
       app_pub_sign_key(safeot.safe_app, safeot_c_void, app_sign_key_callback);
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

#[allow(unused_variables)]
impl SAFEoTComm {
    pub fn new(thing_id: &str, auth_token: &str) -> ResultReturn<SAFEoTComm> {
        let mut safeot_comm = SAFEoTComm {
            thing_id: String::from(thing_id),
            conn_status: ConnStatus::INIT,
            safe_app: null_mut(),
            sign_key_h: Default::default(),
            mutable_data_h: Default::default(),
            perm_set_h: Default::default(),
            perms_h: Default::default(),
            entry_actions_h: Default::default(),
            xor_name: Default::default(),
            // This are attributes of the app itself which are just cached here
            status: String::from("Unknown"),

            // the following are temporary
            attrs: String::from("[]"),
            topics: String::from("[]"),
            actions: String::from("[]"),
            subscriptions: String::from("[]"),
            topic_events: BTreeMap::new(),
        };
        let safeot_comm_c_void_ptr = &mut safeot_comm as *mut _ as *mut c_void;
        unsafe {
            let uri = CString::new(auth_token).unwrap();
            decode_ipc_msg(uri.as_ptr(), safeot_comm_c_void_ptr, auth_cb, containers_cb, revoked_cb, error_cb);
        };

        Ok(safeot_comm)
    }

    pub fn get_thing_status(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.status.clone())
    }

    pub fn get_conn_status(&self) {
        match (self.conn_status) {
            ConnStatus::INIT => println!("CONN STATUS: INIT"),
            ConnStatus::UNREGISTERED => println!("CONN STATUS: UNREGISTERED"),
            ConnStatus::REGISTERED => println!("CONN STATUS: REGISTERED"),
        }
    }

    pub fn get_thing_addr_name(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.xor_name.clone())
    }

    pub fn get_thing_attrs(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.attrs.clone())
    }

    pub fn get_thing_topics(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.topics.clone())
    }

    pub fn get_thing_actions(&self, thing_id: &str) -> ResultReturn<String> {
        Ok(self.actions.clone())
    }

    pub fn store_thing_entity(&mut self, type_tag: u64) -> ResultReturn<String> {
        let Digest(sha256) = sha256::hash(self.thing_id.as_bytes());
        let mut xor_name: [u8; 32] = Default::default();
        xor_name.copy_from_slice(sha256.as_ref());
        unsafe {
            let mut safeot_comm_c_void_ptr = self as *mut _ as *mut c_void;
            mdata_info_new_public(self.safe_app, &xor_name, type_tag, safeot_comm_c_void_ptr, new_md_callback);
        };
        self.xor_name = sha256.as_ref().to_base64(config());
        Ok(self.xor_name.clone())
    }

    pub fn set_status(&mut self, status: &str) -> ResultReturn<()> {
        self.status = String::from(status);
        unsafe {
            let mut safeot_comm_c_void_ptr = self as *mut _ as *mut c_void;
            mdata_entry_actions_new(self.safe_app, safeot_comm_c_void_ptr, mdata_entry_actions_callback);
        };
        Ok(())
    }

    pub fn set_attributes(&mut self, attrs: &str) -> ResultReturn<()> {
        self.attrs = String::from(attrs);
        Ok(())
    }

    pub fn set_topics(&mut self, topics: &str) -> ResultReturn<()> {
        self.topics = String::from(topics);
        Ok(())
    }

    pub fn set_actions(&mut self, actions: &str) -> ResultReturn<()> {
        self.actions = String::from(actions);
        Ok(())
    }

    pub fn set_subscriptions(&mut self, subscriptions: &str) -> ResultReturn<()> {
        self.subscriptions = String::from(subscriptions);
        Ok(())
    }

    pub fn set_topic_events(&mut self, topic: &str, events: &str) -> ResultReturn<()> {
        self.topic_events.insert(String::from(topic), String::from(events));
        Ok(())
    }

    pub fn get_topic_events(&mut self, topic: &str) -> ResultReturn<(String)> {
        let events = self.topic_events.get(&String::from(topic)).unwrap();
        Ok(events.clone())
    }

    pub fn send_action_request(&self, thing_id: &str, action: &str, args: ActionArgs) -> ResultReturn<String> {
        //self.events.push((String::from(topic), String::from(data)));
        Ok(String::from("response"))
    }

}

#[inline]
fn config() -> Config {
    Config {
        char_set: CharacterSet::UrlSafe,
        newline: Newline::LF,
        pad: true,
        line_length: None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}