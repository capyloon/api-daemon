use common::traits::Shared;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub root_path: String,
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

    // Sets a mapping of http://from.localhost to http://to.localhost
    // For instance, http://branding.localhost -> http://b2gos-branding.localhost
    // This is different from a redirection since this lets the "from" origin to
    // be used in CSPs.
    pub fn set_host_mapping(&mut self, from: &str, to: &str) {
        let mappings = &mut self.private.lock().mappings;
        let _ = mappings.remove(from);
        let _ = mappings.insert(from.into(), to.into());
    }

    // name is the app specific part of the url.
    // Eg. for http://contacts.localhost/index.html name is "contacts".
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

    pub fn app_disabled(&mut self, name: &str) {
        self.remove_from_cache(name);
    }
}
