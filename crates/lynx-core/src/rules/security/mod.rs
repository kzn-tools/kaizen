//! Security rules for vulnerability detection

pub mod command_injection;
pub mod sql_injection;
pub mod xss;

pub use command_injection::CommandInjection;
pub use sql_injection::SqlInjection;
pub use xss::Xss;
