/// Tor support:
/// - creates a Tor socks proxy with Arti.
/// - observes the tor.enabled setting.
use arti::driver;
use arti_client::status::BootstrapStatus;
use arti_config::ArtiConfig;
use common::traits::SharedServiceState;
use common::JsonValue;
use futures::future::{AbortHandle, Abortable};
use log::{error, info, warn};
use settings_service::db::{DbObserver, ObserverType};
use settings_service::generated::common::SettingInfo;
use tor_rtcompat::BlockOn;

static TOR_ENABLED_SETTING: &str = "tor.enabled";
static TOR_STATUS_SETTING: &str = "tor.status";

#[derive(Clone)]
struct TorSettingObserver {
    abort_handle: Option<AbortHandle>,
}

impl DbObserver for TorSettingObserver {
    fn callback(&mut self, _name: &str, value: &JsonValue) {
        if let serde_json::Value::Bool(new_value) = &*(*value) {
            info!(
                "Tor status changed to: {}",
                if *new_value { "enabled" } else { "disabled" }
            );

            // Disabling Tor while it's running.
            if !*new_value && self.abort_handle.is_some() {
                self.abort_handle.as_ref().unwrap().abort();
                self.abort_handle = None;
            }

            // Enabling Tor.
            if *new_value && self.abort_handle.is_none() {
                self.abort_handle = Some(Self::start());
            }
        }
    }
}

fn update_status_setting(ready: bool, progress: f32) {
    let json = serde_json::json!(
        {
            "ready": ready,
            "progress": progress
        }
    );
    let settings = settings_service::service::SettingsService::shared_state();
    {
        let db = &mut settings.lock().db;
        if let Err(err) = db.set(&[SettingInfo {
            name: TOR_STATUS_SETTING.to_owned(),
            value: json.into(),
        }]) {
            error!("Failed to set Tor status: {}", err);
        }
    }
}
fn notify_tor_status(status: &BootstrapStatus) {
    info!(
        "Tor status: ready={} frac={} blocked={:?}",
        status.ready_for_traffic(),
        status.as_frac(),
        status.blocked()
    );

    update_status_setting(status.ready_for_traffic(), status.as_frac());
}

impl TorSettingObserver {
    fn start() -> AbortHandle {
        let (abort_handle, abort_registration) = AbortHandle::new_pair();
        let future = Abortable::new(futures::future::pending::<()>(), abort_registration);
        let _ = std::thread::Builder::new()
            .name("tor proxy".into())
            .spawn(move || {
                let config = ArtiConfig::default();
                let client_config = config.tor_client_config().unwrap();
                let socks_port = config.proxy().socks_port().unwrap_or(9150);

                info!("Tor starting socks proxy on port {}", socks_port);

                let runtime = driver::ChosenRuntime::create().unwrap();
                let rt_clone = runtime.clone();
                let _ = rt_clone.block_on(driver::run_abortable(
                    runtime,
                    config,
                    client_config,
                    future,
                    notify_tor_status,
                ));
                info!("Tor stopping proxy");
            });
        abort_handle
    }
}

pub fn start() {
    // Set the initial value of the status, which is never ready at startup.
    update_status_setting(false, 0.0);

    let settings = settings_service::service::SettingsService::shared_state();
    {
        let db = &mut settings.lock().db;

        let mut observer = TorSettingObserver { abort_handle: None };

        // Get the initial enabled value.
        if let Ok(value) = db.get(TOR_ENABLED_SETTING) {
            let json = &*value;
            if let serde_json::Value::Bool(setting_value) = json {
                info!("Tor initially enabled: {}", *setting_value);
                let v: JsonValue = JsonValue::from(json.clone());

                observer.callback(TOR_ENABLED_SETTING, &v);
            } else {
                // Not a boolean, ignoring.
                warn!("No initial value for setting '{}'", TOR_ENABLED_SETTING);
            }
        } else {
            // No such setting, ignoring.
            warn!("No initial value for setting '{}'", TOR_ENABLED_SETTING);
        }

        // Setup a setting listener.
        let _id = db.add_observer(
            &TOR_ENABLED_SETTING,
            ObserverType::FuncPtr(Box::new(observer)),
        );
    }
}
