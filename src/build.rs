use crate::{state::State, Prompt, Result};

/// A trait to build [Prompt](struct.Prompt.html).
pub trait Builder<D, S> {
    fn state(self) -> Result<Box<State<D, S>>>;
    fn build(self) -> Result<Prompt<State<D, S>>>;
}
