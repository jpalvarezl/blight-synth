pub mod cli;
pub mod models;
pub mod project;
mod timing;

type Result<T> = anyhow::Result<T, anyhow::Error>;
