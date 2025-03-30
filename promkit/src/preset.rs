/// Provides a checkbox interface for multiple options selection.
#[cfg(feature = "checkbox")]
pub mod checkbox;

/// Offers functionality for reading input from the user.
#[cfg(feature = "readline")]
pub mod readline;

/// Contains a simple yes/no confirmation prompt.
#[cfg(feature = "confirm")]
pub mod confirm;

/// Provides a password input interface with masking and validation.
#[cfg(feature = "password")]
pub mod password;

/// Enables parsing and interaction with JSON data.
#[cfg(feature = "json")]
pub mod json;

/// Implements a list box for single or multiple selections from a list.
#[cfg(feature = "listbox")]
pub mod listbox;

/// Facilitates querying and selecting from a set of options in a structured format.
#[cfg(feature = "query_selector")]
pub mod query_selector;

/// Supports creating and interacting with a tree structure for hierarchical data.
#[cfg(feature = "tree")]
pub mod tree;

/// Provides multiple readline input options.
#[cfg(feature = "form")]
pub mod form;

/// Provides a static text display.
#[cfg(feature = "text")]
pub mod text;
