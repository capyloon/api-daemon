/// UCAN Capabilities specific to the content manager.
/// TODO: use the ucan-rs types?
use log::{error, debug};
use serde::Deserialize;
use ucan::ucan::Ucan;

// [{\"can\":\"*\",\"with\":\"my:*\"}]
#[derive(Clone, Default, Deserialize)]
struct Att {
    can: String,
    with: String,
}

impl Att {
    fn is_superuser(&self) -> bool {
        self.can == "*" && self.with == "my:*"
    }
}

// Capabilities set:
// Read:   { "with": "vfs:///pictures/", "can": "vfs/READ" }
// Write:  { "with": "vfs:///pictures/", "can": "vfs/WRITE" }
// Search: { "with": "vfs://",           "can": "vfs/SEARCH" }
// Visit:  { "with": "vfs://",           "can": "vfs/VISIT" }
#[derive(Clone, Debug)]
pub(crate) enum UcanCapability {
    Read(String),  // Read access to a path
    Write(String), // Write access to a path
    Search,        // Various search functions
    Visit,         // Visit a resource
}

#[derive(Clone)]
pub(crate) struct UcanCapabilities {
    capabilities: Vec<UcanCapability>,
    superuser: bool,
    identity: String,
}

impl UcanCapabilities {
    pub fn from_ucan(ucan: &Ucan, identity: &str) -> Self {
        let mut superuser = false;
        let mut capabilities = vec![];

        for value in ucan.attenuation() {
            let att = Att {
                with: value.with.clone(),
                can: value.can.clone(),
            };

            if att.is_superuser() {
                superuser = true;
            } else {
                let can_search = att.can == "vfs/SEARCH";
                let can_visit = att.can == "vfs/VISIT";
                let can_write = att.can == "vfs/WRITE";
                let can_read = can_write || att.can == "vfs/READ";
                let is_vfs = att.with.starts_with("vfs:///");
                if !(is_vfs && (can_read || can_write)) {
                    continue;
                }
                let path = &att.with[6..]; // Keep the first /
                if can_read {
                    capabilities.push(UcanCapability::Read(path.into()))
                }
                if can_write {
                    capabilities.push(UcanCapability::Write(path.into()))
                }
                if can_search {
                    capabilities.push(UcanCapability::Search)
                }
                if can_visit {
                    capabilities.push(UcanCapability::Visit)
                }
            }
        }

        Self {
            capabilities,
            superuser,
            identity: identity.into(),
        }
    }

    pub fn new(identity: &str) -> Self {
        Self {
            identity: identity.into(),
            superuser: false,
            capabilities: vec![],
        }
    }

    #[inline(always)]
    pub fn is_superuser(&self) -> bool {
        // debug!("UcanCapabilities is_superuser");
        self.superuser
    }

    pub fn can_read(&self, requested_path: &str) -> bool {
        debug!("UcanCapabilities can_read {}", requested_path);
        debug!("UcanCapabilities {:?}", self.capabilities);
        for cap in &self.capabilities {
            if let UcanCapability::Read(path) = cap {
                // Temporary: special case '/' to allow reading of the root.
                // TODO: provide a better direct access to a full path.
                if requested_path == "/" || requested_path.starts_with(path) {
                    return true;
                }
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/READ capability for {}",
            self.identity, self.superuser, requested_path
        );
        false
    }

    pub fn can_write(&self, requested_path: &str) -> bool {
        debug!("UcanCapabilities can_write {}", requested_path);
        debug!("UcanCapabilities {:?}", self.capabilities);
        for cap in &self.capabilities {
            if let UcanCapability::Write(path) = cap {
                if requested_path.starts_with(path) {
                    return true;
                }
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/WRITE capability for {}",
            self.identity, self.superuser, requested_path
        );
        false
    }

    pub fn can_search(&self) -> bool {
        debug!("UcanCapabilities can_search");
        debug!("UcanCapabilities {:?}", self.capabilities);
        if self.superuser {
            return true;
        }

        for cap in &self.capabilities {
            if let UcanCapability::Search = cap {
                return true;
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/SEARCH capability",
            self.identity, self.superuser,
        );
        false
    }

    pub fn can_visit(&self) -> bool {
        debug!("UcanCapabilities can_visit");
        debug!("UcanCapabilities {:?}", self.capabilities);
        if self.superuser {
            return true;
        }

        for cap in &self.capabilities {
            if let UcanCapability::Visit = cap {
                return true;
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/VISIT capability",
            self.identity, self.superuser,
        );
        false
    }
}
