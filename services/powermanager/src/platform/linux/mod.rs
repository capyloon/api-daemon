/// Linux specific implementation of the power manager trait.
use crate::PowerManagerSupport;
use b2ghald::client::SimpleClient;

pub struct LinuxPowerManager {
    hal: SimpleClient,
}

impl LinuxPowerManager {
    pub fn new() -> Option<Self> {
        SimpleClient::new().map(|hal| Self { hal })
    }
}

impl PowerManagerSupport for LinuxPowerManager {
    fn set_screen_brightness(&mut self, value: u8, screen_id: u8) -> bool {
        self.hal.set_screen_brightness(screen_id, value);
        true
    }

    fn set_screen_state(&mut self, state: bool, screen_id: u8) -> bool {
        if state {
            self.hal.enable_screen(screen_id);
        } else {
            self.hal.disable_screen(screen_id);
        }
        true
    }

    fn power_off(&mut self) {
        self.hal.poweroff();
    }

    fn reboot(&mut self) {
        self.hal.reboot();
    }
}
