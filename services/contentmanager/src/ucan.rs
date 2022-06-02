/// UCAN Capabilities specific to the content manager.
/// TODO: use the ucan-rs types?
use log::{error, info};
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
#[derive(Clone)]
pub(crate) enum UcanCapability {
    Read(String),  // Read access to a path:
    Write(String), // Write access to a path
    Search,        // Various search functions
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
            let att: Att = serde_json::from_value(value.clone()).unwrap_or_else(|_| Att::default());
            if att.is_superuser() {
                superuser = true;
            } else {
                let can_search = att.can == "vfs/SEARCH";
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
        // info!("UcanCapabilities is_superuser");
        self.superuser
    }

    pub fn can_read(&self, full_path: &str) -> bool {
        info!("UcanCapabilities can_read {}", full_path);
        for cap in &self.capabilities {
            if let UcanCapability::Read(path) = cap {
                if full_path.starts_with(path) {
                    return true;
                }
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/READ permission for {}",
            self.identity, self.superuser, full_path
        );
        false
    }

    pub fn can_write(&self, full_path: &str) -> bool {
        info!("UcanCapabilities can_write {}", full_path);
        for cap in &self.capabilities {
            if let UcanCapability::Write(path) = cap {
                if full_path.starts_with(path) {
                    return true;
                }
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/WRITE permission for {}",
            self.identity, self.superuser, full_path
        );
        false
    }

    pub fn can_search(&self) -> bool {
        info!("UcanCapabilities can_search");
        if self.superuser {
            return true;
        }

        for cap in &self.capabilities {
            if let UcanCapability::Search = cap {
                return true;
            }
        }
        error!(
            "{} (superuser={}) is missing vfs/SEARCH permission",
            self.identity, self.superuser,
        );
        false
    }
}
