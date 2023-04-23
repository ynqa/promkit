use crate::{
    grapheme::Grapheme,
    internal::buffer::Buffer,
    readline::{Mode, State},
    register::Register,
    termutil, Result, Runnable, Runner,
};

impl Runnable for Runner<State> {
    fn handle_resize(
        &mut self,
        _: (u16, u16),
        out: &mut std::io::Stdout,
    ) -> Result<Option<String>> {
        termutil::clear(out)?;
        self.state.render_static(out)?;
        // Overwrite the prev as default.
        self.state.prev = Buffer::default();
        Ok(None)
    }

    fn handle_input(&mut self, ch: char, _out: &mut std::io::Stdout) -> Result<Option<String>> {
        if let Some(limit) = self.state.buffer_limit()? {
            if limit <= self.state.editor.data.width() {
                return Ok(None);
            }
        }
        match self.state.edit_mode {
            Mode::Insert => self.state.editor.insert(Grapheme::from(ch)),
            Mode::Overwrite => self.state.editor.overwrite(Grapheme::from(ch)),
        }
        Ok(None)
    }

    fn act(
        &mut self,
        ev: &crossterm::event::Event,
        out: &mut std::io::Stdout,
    ) -> Result<Option<String>> {
        self.keybind.handle(ev, out, &mut self.state)
    }

    fn initialize(&mut self, out: &mut std::io::Stdout) -> Result<Option<String>> {
        self.state.render_static(out)?;
        Ok(None)
    }

    fn finalize(&mut self, out: &mut std::io::Stdout) -> Result<Option<String>> {
        termutil::move_right(out, self.state.editor.width_from_position() as u16)?;
        termutil::move_down(out, 1)?;
        termutil::move_head(out)?;
        if let Some(hstr) = &mut self.state.hstr {
            hstr.register(self.state.editor.data.clone());
        }
        self.state.editor = Buffer::default();
        self.state.prev = Buffer::default();
        self.state.next = Buffer::default();
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut std::io::Stdout) -> Result<Option<String>> {
        self.state.can_render()?;
        self.state.render(out)?;
        self.state.prev = self.state.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _: &mut std::io::Stdout) -> Result<Option<String>> {
        self.state.next = self.state.editor.clone();
        Ok(None)
    }
}
