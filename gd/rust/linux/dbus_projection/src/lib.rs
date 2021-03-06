//! This crate provides tools to automatically project generic API to D-Bus RPC.
//!
//! For D-Bus projection to work automatically, the API needs to follow certain restrictions:
//!
//! * API does not use D-Bus specific features: Signals, Properties, ObjectManager.
//! * Interfaces (contain Methods) are hosted on statically allocated D-Bus objects.
//! * When the service needs to notify the client about changes, callback objects are used. The
//!   client can pass a callback object obeying a specified Interface by passing the D-Bus object
//!   path.
//!
//! A good example is in
//! [`manager_service`](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt)
//! crate:
//!
//! * Define RPCProxy like in
//! [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/lib.rs)
//! (TODO: We should remove this requirement in the future).
//! * Generate `DBusArg` trait like in
//! [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/bin/btmanagerd/dbus_arg.rs).
//! This trait is generated by a macro and cannot simply be imported because of Rust's
//! [Orphan Rule](https://github.com/Ixrec/rust-orphan-rules).
//! * Define D-Bus-agnostic traits like in
//! [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/iface_bluetooth_manager.rs).
//! These traits can be projected into D-Bus Interfaces on D-Bus objects.  A method parameter can
//! be of a Rust primitive type, structure, enum, or a callback specially typed as
//! `Box<dyn SomeCallbackTrait + Send>`. Callback traits implement `RPCProxy`.
//! * Implement the traits like in
//! [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/bin/btmanagerd/bluetooth_manager.rs),
//! also D-Bus-agnostic.
//! * Define D-Bus projection mappings like in
//! [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/bin/btmanagerd/bluetooth_manager_dbus.rs).
//!   * Add [`generate_dbus_exporter`](dbus_macros::generate_dbus_exporter) macro to an `impl` of a
//!     trait.
//!   * Define a method name of each method with [`dbus_method`](dbus_macros::dbus_method) macro.
//!   * Similarly, for callbacks use [`dbus_proxy_obj`](dbus_macros::dbus_proxy_obj) macro to define
//!     the method mappings.
//!   * Rust primitive types can be converted automatically to and from D-Bus types.
//!   * Rust structures require implementations of `DBusArg` for the conversion. This is made easy
//!     with the [`dbus_propmap`](dbus_macros::dbus_propmap) macro.
//!   * Rust enums require implementations of `DBusArg` for the conversion. This is made easy with
//!     the [`impl_dbus_arg_enum`](impl_dbus_arg_enum) macro.
//! * To project a Rust object to a D-Bus, call the function generated by
//!   [`generate_dbus_exporter`](dbus_macros::generate_dbus_exporter) like in
//!   [here](https://android.googlesource.com/platform/system/bt/+/refs/heads/master/gd/rust/linux/mgmt/src/bin/btmanagerd/main.rs)
//!   passing in the object path, D-Bus connection, Crossroads object, the Rust object to be
//!   projected, and a [`DisconnectWatcher`](DisconnectWatcher) object.

use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus::strings::BusName;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A D-Bus "NameOwnerChanged" handler that continuously monitors client disconnects.
pub struct DisconnectWatcher {
    callbacks: Arc<Mutex<HashMap<BusName<'static>, Vec<Box<dyn Fn() + Send>>>>>,
}

impl DisconnectWatcher {
    /// Creates a new DisconnectWatcher with empty callbacks.
    pub fn new() -> DisconnectWatcher {
        DisconnectWatcher { callbacks: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl DisconnectWatcher {
    /// Adds a client address to be monitored for disconnect events.
    pub fn add(&mut self, address: BusName<'static>, callback: Box<dyn Fn() + Send>) {
        if !self.callbacks.lock().unwrap().contains_key(&address) {
            self.callbacks.lock().unwrap().insert(address.clone(), vec![]);
        }

        (*self.callbacks.lock().unwrap().get_mut(&address).unwrap()).push(callback);
    }

    /// Sets up the D-Bus handler that monitors client disconnects.
    pub async fn setup_watch(&mut self, conn: Arc<SyncConnection>) {
        let mr = MatchRule::new_signal("org.freedesktop.DBus", "NameOwnerChanged");

        conn.add_match_no_cb(&mr.match_str()).await.unwrap();
        let callbacks_map = self.callbacks.clone();
        conn.start_receive(
            mr,
            Box::new(move |msg, _conn| {
                // The args are "address", "old address", "new address".
                // https://dbus.freedesktop.org/doc/dbus-specification.html#bus-messages-name-owner-changed
                let (addr, old, new) = msg.get3::<String, String, String>();

                if addr.is_none() || old.is_none() || new.is_none() {
                    return true;
                }

                if old.unwrap().eq("") || !new.unwrap().eq("") {
                    return true;
                }

                // If old address exists but new address is empty, that means that client is
                // disconnected. So call the registered callbacks to be notified of this client
                // disconnect.
                let addr = BusName::new(addr.unwrap()).unwrap().into_static();
                if !callbacks_map.lock().unwrap().contains_key(&addr) {
                    return true;
                }

                for callback in &callbacks_map.lock().unwrap()[&addr] {
                    callback();
                }

                callbacks_map.lock().unwrap().remove(&addr);

                true
            }),
        );
    }
}

/// Implements `DBusArg` for an enum.
///
/// A Rust enum is converted to D-Bus INT32 type.
#[macro_export]
macro_rules! impl_dbus_arg_enum {
    ($enum_type:ty) => {
        impl DBusArg for $enum_type {
            type DBusType = i32;
            fn from_dbus(
                data: i32,
                _conn: Option<Arc<SyncConnection>>,
                _remote: Option<dbus::strings::BusName<'static>>,
                _disconnect_watcher: Option<
                    Arc<std::sync::Mutex<dbus_projection::DisconnectWatcher>>,
                >,
            ) -> Result<$enum_type, Box<dyn std::error::Error>> {
                match <$enum_type>::from_i32(data) {
                    Some(x) => Ok(x),
                    None => Err(Box::new(DBusArgError::new(String::from(format!(
                        "error converting {} to {}",
                        data,
                        stringify!($enum_type)
                    ))))),
                }
            }

            fn to_dbus(data: $enum_type) -> Result<i32, Box<dyn std::error::Error>> {
                return Ok(data.to_i32().unwrap());
            }
        }
    };
}
