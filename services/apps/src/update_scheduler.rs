use crate::generated::common::*;
use crate::shared_state::AppsSharedData;
use crate::tasks::CheckForUpdateTask;
use common::traits::Shared;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::thread;
use std::time::Duration;

#[cfg(not(test))]
use common::traits::Service;
#[cfg(not(test))]
use geckobridge::generated::common::NetworkState;
#[cfg(not(test))]
use geckobridge::service::GeckoBridgeService;
use std::sync::mpsc::{channel, Sender};
use time::OffsetDateTime;

#[cfg(target_os = "android")]
use geckobridge::generated::common::NetworkType;

#[cfg(target_os = "android")]
static CONFIG_DEFAULT: &str = "/system/b2g/defaults/app-update-schedule.json";
#[cfg(target_os = "android")]
static CONFIG_DATA: &str = "/data/local/service/api-daemon/app-update-schedule.json";

#[cfg(not(target_os = "android"))]
static CONFIG_DEFAULT: &str = "/tmp/default-app-update-schedule.json";
#[cfg(not(target_os = "android"))]
static CONFIG_DATA: &str = "/tmp/app-update-schedule.json";

pub enum SchedulerMessage {
    Config(UpdatePolicy),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateScheduler {
    enabled: bool,
    conn_type: ConnectionType, // Connection type allowed for checking updates
    delay: i64,                // In seconds,
    #[serde(default = "default_last_check")]
    last_check: i64,
}

static ONE_DAY: i64 = 24 * 60 * 60;

impl Default for UpdateScheduler {
    fn default() -> Self {
        UpdateScheduler {
            enabled: true,
            conn_type: ConnectionType::WiFiOnly,
            delay: ONE_DAY,
            last_check: 0,
        }
    }
}

impl UpdateScheduler {
    pub fn new() -> Self {
        let mut default = UpdateScheduler::default();
        default.load();

        default
    }

    fn should_trigger(&self) -> bool {
        let today = get_local_time_sec() / self.delay;
        let last_day = self.last_check / self.delay;

        today > last_day
    }

    pub fn configure(&mut self, config: UpdatePolicy) {
        debug!("configure ");
        let changed = self.enabled != config.enabled
            || self.conn_type != config.conn_type
            || self.delay != config.delay;
        self.enabled = config.enabled;
        self.conn_type = config.conn_type;
        self.delay = config.delay;
        if changed {
            self.save();
        }
    }

    pub fn get_config(&self) -> UpdatePolicy {
        UpdatePolicy {
            enabled: self.enabled,
            conn_type: self.conn_type,
            delay: self.delay,
        }
    }

    fn save(&self) {
        if let Ok(file) = File::create(CONFIG_DATA) {
            debug!("scheduler save to file, config is {:?}", &self);
            if let Err(err) = serde_json::to_writer(file, &self) {
                error!("Error writing file {}, {}", CONFIG_DATA, err);
            }
        } else {
            error!("Creating file {} failed!", CONFIG_DATA);
        }
    }

    fn load_config(&mut self, path: &str) -> bool {
        if let Ok(file) = File::open(path) {
            if let Ok(config) = serde_json::from_reader::<_, UpdateScheduler>(file) {
                debug!("load_config scheduler is {:?}", &config);
                self.enabled = config.enabled;
                self.conn_type = config.conn_type;
                self.delay = config.delay;
                self.last_check = config.last_check;

                true
            } else {
                error!("Failed to read {}", path);
                false
            }
        } else {
            false
        }
    }

    fn load(&mut self) {
        if !self.load_config(CONFIG_DATA) {
            self.load_config(CONFIG_DEFAULT);
        }
    }

    #[cfg(target_os = "android")]
    fn connection_type_allowed(&self) -> bool {
        let maybe_conn = GeckoBridgeService::shared_state()
            .lock()
            .networkmanager_get_network_info();
        let conn = match maybe_conn.get() {
            Ok(conn) => conn,
            Err(_) => {
                debug!("networkmanager_get_network_info failed");
                return false;
            }
        };

        // TODO need to confirm many mobile types and the real meaning of
        // NetworkStateConnected
        if self.conn_type == ConnectionType::WiFiOnly
            && conn.network_type != NetworkType::NetworkTypeWifi
        {
            false
        } else {
            true
        }
    }

    #[cfg(not(target_os = "android"))]
    fn connection_type_allowed(&self) -> bool {
        true
    }

    fn notify_checked(&mut self) {
        self.last_check = get_local_time_sec();
        self.save();
    }

    fn check(&mut self, shared_data: Shared<AppsSharedData>) {
        debug!("checking");
        let shared = shared_data.lock();
        shared.registry.get_all().iter().for_each(|app| {
            debug!("checking apps");
            let update_url: String = app.update_url.to_owned();
            if !update_url.is_empty() {
                // Create tasks and check for update
                let apps_options = AppsOptions {
                    auto_install: Some(true),
                };
                let task = CheckForUpdateTask(shared_data.clone(), update_url, apps_options, None);
                shared.registry.queue_task(task);
            }
        });

        self.notify_checked();
        debug!("checking end");
    }
}

pub fn start(shared_data: Shared<AppsSharedData>) -> Sender<SchedulerMessage> {
    let mut scheduler = UpdateScheduler::new();
    info!("starting enabled {}", scheduler.enabled);
    let (sender, receiver) = channel();

    thread::Builder::new()
        .name("update scheduler".into())
        .spawn(move || {
            debug!("start delay starting now");
            loop {
                if let Ok(SchedulerMessage::Config(config)) = receiver.try_recv() {
                    debug!("Config received {:?}", config);
                    scheduler.configure(config);
                };

                if scheduler.enabled && scheduler.should_trigger() {
                    if network_connected() {
                        if scheduler.connection_type_allowed() {
                            scheduler.check(shared_data.clone());
                        }
                    } else {
                        // TODO Observe network connectivity
                    }
                }

                thread::sleep(Duration::from_secs(10));
            }
        })
        .expect("Failed to create app update scheduler thread");

    sender
}

#[cfg(test)]
pub fn network_connected() -> bool {
    true
}

#[cfg(not(test))]
pub fn network_connected() -> bool {
    let maybe_netinfo = GeckoBridgeService::shared_state()
        .lock()
        .networkmanager_get_network_info();
    match maybe_netinfo.get() {
        Ok(conn) => conn.network_state == NetworkState::NetworkStateConnected,
        Err(_) => {
            debug!("networkmanager_get_network_info failed");
            false
        }
    }
}

fn get_local_time_sec() -> i64 {
    let today_local = OffsetDateTime::try_now_local()
        .or_else::<(), _>(|_| Ok(OffsetDateTime::now_utc()))
        .unwrap();
    let offset_seconds = today_local.offset().as_seconds();
    let mut today_sec = today_local.unix_timestamp() + i64::from(offset_seconds);
    if today_sec < 0 {
        today_sec = 0;
    }

    today_sec
}

fn default_last_check() -> i64 {
    0
}

#[test]
fn test_in_order() {
    test_scheduler();
    test_scheduler_thread();

    fn test_scheduler() {
        use std::env;
        use std::fs;
        use std::path::Path;

        // Remove saved config
        let _ = fs::remove_file(CONFIG_DEFAULT);
        let _ = fs::remove_file(CONFIG_DATA);

        // Default
        let mut scheduler = UpdateScheduler::new();
        assert_eq!(scheduler.enabled, true);
        assert_eq!(scheduler.conn_type, ConnectionType::WiFiOnly);
        assert_eq!(scheduler.delay, ONE_DAY);
        assert_eq!(scheduler.should_trigger(), true);

        scheduler.notify_checked();

        assert_eq!(get_local_time_sec() - scheduler.last_check < 1, true);

        assert_eq!(scheduler.should_trigger(), false);
        // Remove saved config
        let _ = fs::remove_file(CONFIG_DATA);

        // Customization
        let _ = env_logger::try_init();
        let current = env::current_dir().unwrap();
        let default_src_dir = format!("{}/test-fixtures/test-scheduler", current.display());
        let default_config = Path::new(&default_src_dir).join("default-app-update-schedule.json");
        let _ = fs::copy(default_config, CONFIG_DEFAULT);

        let mut scheduler = UpdateScheduler::new();
        assert_eq!(scheduler.enabled, false);
        assert_eq!(scheduler.conn_type, ConnectionType::WiFiOnly);
        assert_eq!(scheduler.delay, 24);
        assert_eq!(scheduler.should_trigger(), true);

        scheduler.notify_checked();
        assert_eq!(get_local_time_sec() - scheduler.last_check < 1, true);

        assert_eq!(scheduler.should_trigger(), false);

        let config = UpdatePolicy {
            enabled: true,
            conn_type: ConnectionType::Any,
            delay: 1000,
        };
        scheduler.configure(config);

        scheduler.notify_checked();
        scheduler.save();

        // Load from User config
        let scheduler = UpdateScheduler::new();
        assert_eq!(scheduler.enabled, true);
        assert_eq!(scheduler.conn_type, ConnectionType::Any);
        assert_eq!(scheduler.delay, 1000);
        assert_eq!(scheduler.should_trigger(), false);
    }

    fn test_scheduler_thread() {
        use crate::shared_state::APPS_SHARED_SHARED_DATA;
        let _ = std::fs::remove_file(CONFIG_DEFAULT);
        let _ = std::fs::remove_file(CONFIG_DATA);

        {
            let shared = &*APPS_SHARED_SHARED_DATA;
            let sender = self::start(shared.clone());

            let _ = sender.send(SchedulerMessage::Config(UpdatePolicy {
                enabled: true,
                conn_type: ConnectionType::Any,
                delay: 25,
            }));

            // The scheduler loops every 10 sec
            thread::sleep(Duration::from_secs(11));

            // Check saved config
            let scheduler = UpdateScheduler::new();
            assert_eq!(scheduler.enabled, true);
            assert_eq!(scheduler.conn_type, ConnectionType::Any);
            assert_eq!(scheduler.delay, 25);
            assert_eq!(scheduler.should_trigger(), false);
        }
    }
}
