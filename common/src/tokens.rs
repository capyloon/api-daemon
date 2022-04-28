/// Management of the session peer identity.
/// Each peer presents a token when opening a session
/// and we check if this token and valid, and to which
/// permissions this gives the peer.
///
/// While token are per-session, they map to a stable
/// identifier that can be used for stateful operations
/// across sessions.
/// For web clients, the identifier is the caller origin.
use crate::traits::{OriginAttributes, Shared};
use std::collections::HashMap;

pub type SharedTokensManager = Shared<TokensManager>;

#[derive(Default)]
pub struct TokensManager {
    // Map token -> attributes.
    ids: HashMap<String, OriginAttributes>,
}

impl TokensManager {
    // Creates a new manager shareable among threads.
    pub fn new_shareable() -> SharedTokensManager {
        Shared::adopt(TokensManager::default())
    }

    // Register a new token for the given identity.
    // Returns whether the operation is successfull.
    pub fn register(&mut self, token: &str, attrs: OriginAttributes) -> bool {
        if self.ids.contains_key(token) {
            // We never should register twice the same token.
            return false;
        }

        self.ids.insert(token.into(), attrs).is_none()
    }

    // Returns the identity associated with the token.
    // This will remove the token from the set of valid tokens to
    // prevent session reuse.
    pub fn get_origin_attributes(&mut self, token: &str) -> Option<OriginAttributes> {
        self.ids.remove(token)
    }
}

#[test]
fn token_manager_test() {
    use std::collections::HashSet;

    let mut mgr = TokensManager::default();

    let mut permissions = HashSet::new();
    permissions.insert("permission-1".to_string());
    permissions.insert("permission-2".to_string());

    let attr = OriginAttributes::new("client-id-0", permissions);

    assert!(mgr.register("random-token", attr.clone()));
    assert!(!mgr.register("random-token", attr));

    let attr = mgr.get_origin_attributes("random-token").unwrap();
    assert_eq!(attr.identity(), "client-id-0".to_owned());
    assert!(attr.has_permission("permission-1"));
    assert!(attr.has_permission("permission-2"));
    assert!(!attr.has_permission("permission-3"));

    // Single use token can't be re-used.
    assert!(mgr.get_origin_attributes("random-token").is_none());
}
