#![cfg_attr(docsrs, feature(doc_cfg))]

pub use promkit_core as core;

#[cfg(feature = "checkbox")]
#[cfg_attr(docsrs, doc(cfg(feature = "checkbox")))]
pub mod checkbox;

#[cfg(any(feature = "json", feature = "yaml", feature = "tree"))]
pub mod structured;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use serde_json;
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use structured::json;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub use serde_yaml;
#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub use structured::yaml;

#[cfg(feature = "listbox")]
#[cfg_attr(docsrs, doc(cfg(feature = "listbox")))]
pub mod listbox;

#[cfg(feature = "prefixsearch")]
#[cfg_attr(docsrs, doc(cfg(feature = "prefixsearch")))]
pub mod prefix_search;

#[cfg(feature = "table")]
#[cfg_attr(docsrs, doc(cfg(feature = "table")))]
pub mod table;

#[cfg(feature = "text")]
#[cfg_attr(docsrs, doc(cfg(feature = "text")))]
pub mod text;

#[cfg(feature = "status")]
#[cfg_attr(docsrs, doc(cfg(feature = "status")))]
pub mod status;

#[cfg(feature = "texteditor")]
#[cfg_attr(docsrs, doc(cfg(feature = "texteditor")))]
pub mod text_editor;

#[cfg(feature = "spinner")]
#[cfg_attr(docsrs, doc(cfg(feature = "spinner")))]
pub mod spinner;
