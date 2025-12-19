//! Security rules for vulnerability detection

pub mod command_injection;
pub mod eval_injection;
pub mod hardcoded_secrets;
pub mod insecure_random;
pub mod sql_injection;
pub mod weak_hashing;
pub mod xss;

pub use command_injection::CommandInjection;
pub use eval_injection::EvalInjection;
pub use hardcoded_secrets::HardcodedSecrets;
pub use insecure_random::InsecureRandom;
pub use sql_injection::SqlInjection;
pub use weak_hashing::WeakHashing;
pub use xss::Xss;
