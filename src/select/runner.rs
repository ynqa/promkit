use std::io;

use crate::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers},
    internal::selector::Selector,
    select::State,
    termutil, Result, Runnable, Runner,
};

impl Runner<State> {
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
}

impl Runnable for Runner<State> {
    fn used_lines(&self) -> Result<u16> {
        self.state.used_lines()
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            if let Some(ret) = self.handle_resize((*x, *y), out)? {
                return Ok(Some(ret));
            }
        }

        if let Some(ret) = self.keybind.handle(ev, out, &mut self.state)? {
            return Ok(Some(ret));
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if let Some(ret) = self.handle_input(*ch, out)? {
                return Ok(Some(ret));
            }
        }

        Ok(None)
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
            self.state
                .title
                .as_ref()
                .map_or(Ok(0), |t| t.used_lines())?,
        )?;
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        self.state.render(out)?;
        self.state.prev = self.state.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _: &mut io::Stdout) -> Result<Option<String>> {
        self.state.next = self.state.editor.clone();
        Ok(None)
    }
}
