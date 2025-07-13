// #[cfg(feature = "checkbox")]
// #[cfg_attr(docsrs, doc(cfg(feature = "checkbox")))]
// pub mod checkbox;

#[cfg(feature = "readline")]
#[cfg_attr(docsrs, doc(cfg(feature = "readline")))]
pub mod readline;

#[cfg(feature = "confirm")]
#[cfg_attr(docsrs, doc(cfg(feature = "confirm")))]
pub mod confirm;

#[cfg(feature = "password")]
#[cfg_attr(docsrs, doc(cfg(feature = "password")))]
pub mod password;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub mod json;

#[cfg(feature = "listbox")]
#[cfg_attr(docsrs, doc(cfg(feature = "listbox")))]
pub mod listbox;

#[cfg(feature = "query-selector")]
#[cfg_attr(docsrs, doc(cfg(feature = "query-selector")))]
pub mod query_selector;

#[cfg(feature = "tree")]
#[cfg_attr(docsrs, doc(cfg(feature = "tree")))]
pub mod tree;

#[cfg(feature = "form")]
#[cfg_attr(docsrs, doc(cfg(feature = "form")))]
pub mod form;

#[cfg(feature = "text")]
#[cfg_attr(docsrs, doc(cfg(feature = "text")))]
pub mod text;
