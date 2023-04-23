use std::io;

use crate::{
    internal::selector::Selector, select::State, termutil, text, Result, Runnable, Runner,
};

impl Runnable for Runner<State> {
    fn handle_resize(&mut self, _: (u16, u16), out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::clear(out)?;
        self.state.editor.to_head();
        self.state.cursor.to_head();
        self.state.render_static(out)?;
        // Overwrite the prev as default.
        self.state.prev = Selector::default();
        Ok(None)
    }

    fn handle_input(&mut self, _: char, _: &mut io::Stdout) -> Result<Option<String>> {
        Ok(None)
    }

    fn act(
        &mut self,
        ev: &crossterm::event::Event,
        out: &mut io::Stdout,
    ) -> Result<Option<String>> {
        self.keybind.handle(ev, out, &mut self.state)
    }

    fn initialize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::hide_cursor(out)?;
        self.state.render_static(out)?;
        Ok(None)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::show_cursor(out)?;
        termutil::move_down(
            out,
            termutil::num_lines(
                &self
                    .state
                    .title
                    .as_ref()
                    .unwrap_or(&text::State::default())
                    .text,
            )?,
        )?;
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        self.state.can_render()?;
        self.state.render(out)?;
        self.state.prev = self.state.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>> {
        self.state.next = self.state.editor.clone();
        Ok(None)
    }
}
