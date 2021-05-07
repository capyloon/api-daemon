/// Configuration structure for codegen.

pub struct Config {
    pub js_common_path: String, // The path that leads to the "common" crate.
}

impl Default for Config {
    fn default() -> Self {
        Self {
            js_common_path: "../../../../common".into(),
        }
    }
}
