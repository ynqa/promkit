use std::io;

use crate::{
    internal::selector::Selector, keybind::KeyBind, select::State, termutil, text, Result, Runnable,
};

pub struct Select {
    pub keybind: KeyBind<State>,
    pub state: State,
}

impl Runnable for Select {
    fn handle_resize(&mut self, _: (u16, u16), _: &mut io::Stdout) -> Result<Option<String>> {
        Ok(None)
    }

    fn handle_input(&mut self, _: char, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::clear(out)?;
        self.state.editor.to_head();
        self.state.cursor.to_head();
        self.state.render_static(out)?;
        // Overwrite the prev as default.
        self.state.prev = Selector::default();
        Ok(None)
    }

    fn act(
        &mut self,
        ev: &crossterm::event::Event,
        out: &mut io::Stdout,
    ) -> Result<Option<String>> {
        self.keybind.handle(ev, out, &mut self.state)
    }

    fn initialize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::hide_cursor(out)?;
        self.state.render_static(out)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
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
        )
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<()> {
        self.state.can_render()?;
        self.state.render(out)?;
        self.state.prev = self.state.editor.clone();
        Ok(())
    }

    fn post_run(&mut self, _: &mut io::Stdout) -> Result<()> {
        self.state.next = self.state.editor.clone();
        Ok(())
    }
}
