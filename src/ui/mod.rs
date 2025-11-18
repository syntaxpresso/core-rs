#[cfg(feature = "ui")]
pub mod form_trait;
#[cfg(feature = "ui")]
pub mod forms;
#[cfg(feature = "ui")]
pub mod runner;
#[cfg(feature = "ui")]
pub mod widgets;

#[cfg(feature = "ui")]
pub use form_trait::FormBehavior;
#[cfg(feature = "ui")]
pub use runner::run_ui_command;
