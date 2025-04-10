pub mod cli;
mod config;

pub use config::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraArgs {
    pub query: Vec<(String, String)>,
    pub header: Vec<(String, String)>,
    pub body: Vec<(String, String)>,
}
