//! Quality rules for code style and best practices

pub mod eqeqeq;
pub mod floating_promises;
pub mod no_console;
pub mod no_eval;
pub mod no_unused_vars;
pub mod no_var;
pub mod prefer_optional_chaining;
pub mod prefer_using;

pub use eqeqeq::Eqeqeq;
pub use floating_promises::FloatingPromises;
pub use no_console::NoConsole;
pub use no_eval::NoEval;
pub use no_unused_vars::NoUnusedVars;
pub use no_var::NoVar;
pub use prefer_optional_chaining::PreferOptionalChaining;
pub use prefer_using::PreferUsing;
