use common::traits::Shared;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub root_path: String,
    pub cert_path: String,
    pub key_path: String,
    pub csp: String,
}

#[derive(Default)]
pub struct VhostApi {
    private: Shared<crate::vhost_handler::AppData>,
}

impl VhostApi {
    pub fn new(private: Shared<crate::vhost_handler::AppData>) -> Self {
        Self { private }
    }

    fn remove_from_cache(&mut self, name: &str) {
        let zips = &mut self.private.lock().zips;
        let name = name.to_string();

        if zips.contains_key(&name) {
            let _ = zips.remove(&name);
        }
    }

    // name is the app specific part of the url.
    // Eg. for https://contacts.local/index.html name is "contacts".
    // For now we just remove the entry from the cache, but some
    // improvements are possible, like preloading the zip in the cache
    // at installation and update.
    pub fn app_installed(&mut self, name: &str) {
        self.remove_from_cache(name);
    }

    pub fn app_updated(&mut self, name: &str) {
        self.remove_from_cache(name);
    }

    pub fn app_uninstalled(&mut self, name: &str) {
        self.remove_from_cache(name);
    }
}
