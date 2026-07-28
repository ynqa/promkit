#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "runtime")]
pub use anyhow;
#[cfg(feature = "runtime")]
pub use async_trait;
#[cfg(feature = "core")]
pub use promkit_core as core;
#[cfg(feature = "widgets")]
pub use promkit_widgets as widgets;

#[cfg(feature = "validate")]
pub mod validate;

#[cfg(feature = "runtime")]
mod runtime;
#[cfg(feature = "runtime")]
pub use runtime::{Prompt, Signal, EVENT_STREAM};

#[cfg(feature = "terminal-session")]
mod terminal_session;
#[cfg(feature = "terminal-session")]
pub use terminal_session::{TerminalModes, TerminalSession};
