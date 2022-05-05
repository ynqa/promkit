use std::io;

use crate::Result;

#[derive(Default)]
pub struct State<D, S>(pub Inherited<D>, pub S);

#[derive(Default)]
pub struct Inherited<D> {
    pub input_stream: Vec<(Box<D>, Box<D>)>,
    pub editor: Box<D>,
}

/// A trait to render the items into the output stream.
pub trait Render<W: io::Write> {
    fn pre_render(&self, out: &mut W) -> Result<()>;
    fn render(&mut self, out: &mut W) -> Result<()>;
}
