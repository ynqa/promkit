#![cfg_attr(docsrs, feature(doc_cfg))]

pub use promkit_core as core;

pub mod cursor;

#[cfg(feature = "checkbox")]
#[cfg_attr(docsrs, doc(cfg(feature = "checkbox")))]
pub mod checkbox;

#[cfg(feature = "jsonstream")]
#[cfg_attr(docsrs, doc(cfg(feature = "jsonstream")))]
pub mod jsonstream;
#[cfg(feature = "jsonstream")]
#[cfg_attr(docsrs, doc(cfg(feature = "jsonstream")))]
pub use serde_json;

#[cfg(feature = "listbox")]
#[cfg_attr(docsrs, doc(cfg(feature = "listbox")))]
pub mod listbox;

#[cfg(feature = "text")]
#[cfg_attr(docsrs, doc(cfg(feature = "text")))]
pub mod text;

#[cfg(feature = "texteditor")]
#[cfg_attr(docsrs, doc(cfg(feature = "texteditor")))]
pub mod text_editor;

#[cfg(feature = "tree")]
#[cfg_attr(docsrs, doc(cfg(feature = "tree")))]
pub mod tree;
