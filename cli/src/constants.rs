use regex::Regex;

use android_localization_utilities::DevExpt;

pub mod commands {
    pub const LOCALIZE: &str = "localize";
    pub const LOCALIZED: &str = "localized";
    pub const VALIDATE: &str = "validate";
    pub const CHECK_LOCALIZATION: &str = "check_localization";
}

pub mod args {
    pub const RES_DIR: &str = "res-dir";
    pub const LOCALIZE_OUTPUT_DIR: &str = "output-dir";
    pub const LOCALIZED_INPUT_FILE: &str = "input-file";
    pub const MAPPING: &str = "mapping";
}

lazy_static::lazy_static! {
    pub static ref TEXT_TO_TEXT_REGEX: Regex = Regex::new("^([a-zA-Z]+)=([a-zA-Z]+)$").expt("Invalid regex!");
}
