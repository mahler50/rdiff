pub mod cli;
mod config;
mod req;
mod utils;

pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use req::RequestProfile;
pub use utils::diff_text;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub query: Vec<(String, String)>,
    pub header: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
