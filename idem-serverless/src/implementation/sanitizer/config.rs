use serde::{Deserialize, Serialize};

// TODO - change tiny-clean to allow serialization of mode enums
// TODO - more encoder types (html, css, cdata, etc.)
#[derive(Deserialize, Serialize, Clone)]
pub enum SanitizerMode {
    JavaScript(u64, bool),
    Uri(u64),
    Xml(u64)
}

impl Default for SanitizerMode {
    fn default() -> Self {
        SanitizerMode::JavaScript(4, true)
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub enum SanitizerSettings {

    #[default]
    Disabled,
    Enabled {
        mode: SanitizerMode,
        ignore_list: Option<Vec<String>>,
        encode_list: Option<Vec<String>>
    }
}
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct SanitizerHandlerConfig {
    pub enabled: bool,
    pub body_sanitizer: SanitizerSettings,
    pub header_sanitizer: SanitizerSettings
}