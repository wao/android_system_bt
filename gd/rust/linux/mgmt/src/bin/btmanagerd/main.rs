mod bluetooth_manager;
mod bluetooth_manager_dbus;
mod config_util;
mod dbus_arg;
mod state_machine;

use crate::bluetooth_manager::BluetoothManager;
use dbus::channel::MatchingReceiver;
use dbus::message::MatchRule;
use dbus::nonblock::SyncConnection;
use dbus_crossroads::Crossroads;
use dbus_projection::DisconnectWatcher;
use dbus_tokio::connection;
use log::{Level, Metadata, Record};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true || metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

#[derive(Clone)]
struct ManagerContext {
    proxy: state_machine::StateMachineProxy,
    floss_enabled: Arc<AtomicBool>,
    dbus_connection: Arc<SyncConnection>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log::set_logger(&LOGGER)
        .map(|()| {
            log::set_max_level(
                config_util::get_log_level().unwrap_or(Level::Info).to_level_filter(),
            )
        })
        .unwrap();

    // Initialize config util
    config_util::fix_config_file_format();

    // Connect to the D-Bus system bus (this is blocking, unfortunately).
    let (resource, conn) = connection::new_system_sync()?;

    let context = state_machine::start_new_state_machine_context();
    let proxy = context.get_proxy();
    let manager_context = ManagerContext {
        proxy: proxy,
        floss_enabled: Arc::new(AtomicBool::new(config_util::is_floss_enabled())),
        dbus_connection: conn.clone(),
    };

    // The resource is a task that should be spawned onto a tokio compatible
    // reactor ASAP. If the resource ever finishes, you lost connection to D-Bus.
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    // Let's request a name on the bus, so that clients can find us.
    conn.request_name("org.chromium.bluetooth.Manager", false, true, false).await?;

    // Create a new crossroads instance.
    // The instance is configured so that introspection and properties interfaces
    // are added by default on object path additions.
    let mut cr = Crossroads::new();

    // Enable async support for the crossroads instance.
    cr.set_async_support(Some((
        conn.clone(),
        Box::new(|x| {
            tokio::spawn(x);
        }),
    )));

    // Object manager is necessary for clients (to inform them when Bluetooth is
    // available). Create it at root (/) so subsequent additions generate
    // InterfaceAdded and InterfaceRemoved signals.
    cr.set_object_manager_support(Some(conn.clone()));
    cr.insert("/", &[cr.object_manager()], {});

    let bluetooth_manager = Arc::new(Mutex::new(Box::new(BluetoothManager::new(manager_context))));

    // Set up the disconnect watcher to monitor client disconnects.
    let disconnect_watcher = Arc::new(Mutex::new(DisconnectWatcher::new()));
    disconnect_watcher.lock().unwrap().setup_watch(conn.clone()).await;

    // Let's add the "/org/chromium/bluetooth/Manager" path, which implements
    // the org.chromium.bluetooth.Manager interface, to the crossroads instance.
    bluetooth_manager_dbus::export_bluetooth_manager_dbus_obj(
        "/org/chromium/bluetooth/Manager",
        conn.clone(),
        &mut cr,
        bluetooth_manager.clone(),
        disconnect_watcher.clone(),
    );

    // We add the Crossroads instance to the connection so that incoming method calls will be handled.
    conn.start_receive(
        MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );

    tokio::spawn(async move {
        state_machine::mainloop(context, bluetooth_manager).await;
    });

    std::future::pending::<()>().await;

    // Run forever.
    unreachable!()
}
