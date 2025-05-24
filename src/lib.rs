pub mod cli;
mod config;
mod utils;

pub use config::{
    DiffConfig, DiffProfile, LoadConfig, RequestConfig, RequestProfile, ResponseProfile,
    get_body_text, get_headers_text, get_status_text,
};
pub use utils::{diff_text, highlight_text};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub query: Vec<(String, String)>,
    pub header: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
