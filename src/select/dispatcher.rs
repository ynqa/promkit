use std::io;

use crate::{
    crossterm::{
        cursor,
        event::{Event, KeyCode, KeyEvent, KeyModifiers},
    },
    internal::selector::Selector,
    keybind::KeyBind,
    select::State,
    termutil, text, Result, Runnable,
};

pub struct Dispatcher {
    pub keybind: KeyBind<State>,
    /// Title displayed on the initial line.
    pub title: Option<text::State>,
    pub select: State,
}

impl Dispatcher {
    fn handle_resize(&mut self, _: (u16, u16), out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::clear(out)?;
        self.select.editor.to_head();
        self.select.screen_position = 0;
        // Render the title.
        if let Some(ref mut title) = self.title {
            title.render(out)?;
        }
        // Overwrite the prev as default.
        self.select.prev = Selector::default();
        Ok(None)
    }

    fn handle_input(&mut self, _: char, _: &mut io::Stdout) -> Result<Option<String>> {
        Ok(None)
    }
}

impl Runnable for Dispatcher {
    fn used_lines(&self) -> Result<u16> {
        let title_lines = self.title.as_ref().map_or(Ok(0), |t| t.text_lines())?;
        let selector_lines = self.select.selector_lines()?;
        Ok(title_lines + selector_lines)
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(x, y) = ev {
            if let Some(ret) = self.handle_resize((*x, *y), out)? {
                return Ok(Some(ret));
            }
        }

        if let Some(ret) = self.keybind.handle(ev, out, &mut self.select)? {
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
        // Render the title.
        if let Some(ref mut title) = self.title {
            title.render(out)?;
        }
        Ok(None)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        termutil::show_cursor(out)?;
        Ok(None)
    }

    fn pre_run(&mut self, out: &mut io::Stdout) -> Result<Option<String>> {
        crossterm::execute!(out, cursor::SavePosition)?;
        // Sync number of title lines with select state.
        self.select.title_lines = self.title.as_ref().map_or(Ok(0), |t| t.text_lines())?;
        self.select.render(out)?;
        // Return to the initial position before rendering.
        crossterm::execute!(out, cursor::RestorePosition)?;
        self.select.prev = self.select.editor.clone();
        Ok(None)
    }

    fn post_run(&mut self, _out: &mut io::Stdout) -> Result<Option<String>> {
        self.select.next = self.select.editor.clone();
        Ok(None)
    }
}
