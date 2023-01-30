
#![feature(async_closure)]
#![feature(flt2dec)]

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// #![warn(clippy::restriction)]
#![warn(clippy::style)]


pub mod configuration;
pub mod content;
pub mod db;
pub mod discovery;
pub mod domain;
pub mod telemetry;
pub mod metrics;
pub mod utils;
