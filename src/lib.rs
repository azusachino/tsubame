/**
 * Tsubame [A song from YOASOBI]
 */

pub const CURRENT_VERSION: &str = "0.0.1";

pub mod models;

mod controllers;
mod services;
mod support;

pub use support::GlobalState;
