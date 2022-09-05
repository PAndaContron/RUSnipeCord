use serde::Deserialize;
use serde_trim::string_trim;

/// Configuration data for the bot, loaded from config.json
#[derive(Deserialize, Debug)]
pub struct Config {
    /// The Discord webhook URL
    pub webhook: String,
    /// Optional string to include at the beginning of a message,
    /// intended to be used to mention a user/role.
    /// For user mentions, do `"<@user id>"`;
    /// for role mentions, do `"<@&role id>"`.
    pub mention: Option<String>,
    /// Timeout for repeating a reminder that a course you're watching is open, in seconds; defaults to 60.
    /// If a course closes and reopens, the timeout will be reset automatically.
    #[serde(default = "default_repeat_timeout")]
    pub repeat_timeout: u32,
    /// The year of the courses we're sniping
    pub year: String,
    /// The term/semester of the courses we're sniping;
    /// `"0"` for winter, `"1"` for spring, `"7"` for summer, `"9"` for fall
    pub term: String,
    /// The campus of the courses we're sniping;
    /// `"NB"` for New Brunswick, `"NK"` for Newark, `"CM"` for Camden
    pub campus: String,
    /// The level of the courses we're sniping;
    /// `"U"` for undergrad and `"G"` for graduate
    pub level: String,
    /// A list of registration indexes; each index is a five-digit number you can find on SOC/CSP/Webreg
    pub indexes: Vec<String>,
}

fn default_repeat_timeout() -> u32 { 60 }

/// Data we need to extract from course info
#[derive(Deserialize, Debug)]
pub struct Course {
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    pub sections: Vec<Section>,
}

/// Data we need to extract from section info
#[derive(Deserialize, Debug)]
pub struct Section {
    pub number: String,
    pub index: String,
}
