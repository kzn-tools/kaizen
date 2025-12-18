//! Quality rules for code style and best practices

pub mod eqeqeq;
pub mod no_console;
pub mod no_eval;
pub mod no_unused_vars;
pub mod no_var;
pub mod prefer_using;

pub use eqeqeq::Eqeqeq;
pub use no_console::NoConsole;
pub use no_eval::NoEval;
pub use no_unused_vars::NoUnusedVars;
pub use no_var::NoVar;
pub use prefer_using::PreferUsing;
