#![feature(async_closure)]
#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![warn(clippy::style)]

pub mod configuration;
pub mod content;
pub mod db;
pub mod discovery;
pub mod domain;
pub mod metrics;
pub mod prometheus;
pub mod telemetry;
pub mod utils;
