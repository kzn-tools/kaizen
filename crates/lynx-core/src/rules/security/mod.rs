//! Security rules for vulnerability detection

pub mod command_injection;
pub mod sql_injection;

pub use command_injection::CommandInjection;
pub use sql_injection::SqlInjection;
