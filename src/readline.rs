use std::fmt;
use std::io;

use crate::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyModifiers},
        style, terminal,
    },
    grapheme::{Grapheme, Graphemes},
    internal::buffer::Buffer,
    internal::selector::history::History,
    keybind::KeyBind,
    register::Register,
    suggest::Suggest,
    termutil, text, Controller, Result,
};

pub mod action;
mod builder;
mod keybind;

pub use self::builder::Builder;

/// Readline specific state.
#[derive(Debug)]
pub struct State {
    pub editor: Buffer,
    pub prev: Buffer,
    pub next: Buffer,
    /// A label as prompt (e.g. ">>").
    pub label: Graphemes,
    pub label_color: style::Color,
    /// A char to mask the input chars (e.g. "*"),
    /// for example when you type the passwords.
    pub mask: Option<Grapheme>,
    pub edit_mode: Mode,
    /// How many lines to receive the user input string.
    pub num_lines: Option<u16>,
    pub hstr: Option<History>,
    pub suggest: Option<Suggest>,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.editor.data)
    }
}

impl State {
    pub fn buffer_lines(&self) -> Result<u16> {
        let unused_rows = terminal::size()?.1;
        Ok(*vec![unused_rows, self.num_lines.unwrap_or(unused_rows)]
            .iter()
            .min()
            .unwrap_or(&unused_rows))
    }

    pub fn buffer_limit(&self) -> Result<u16> {
        // -1 is for the space for cursor.
        Ok(terminal::size()?.0 * self.buffer_lines()? - self.label.width() as u16 - 1)
    }
}

/// Edit mode.
#[derive(Debug, Clone)]
pub enum Mode {
    /// Insert a char at the current position.
    Insert,
    /// Overwrite a char at the current position.
    Overwrite,
}

pub struct EventHandler {
    pub keybind: KeyBind<State>,
}

impl EventHandler {
    pub fn handle_event(
        &self,
        ev: &Event,
        out: &mut io::Stdout,
        readline: &mut State,
    ) -> Result<Option<String>> {
        if let Some(ret) = self.keybind.handle(ev, out, readline)? {
            return Ok(Some(ret));
        }

        if let Event::Key(KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
            ..
        }) = ev
        {
            if readline.buffer_limit()? <= readline.editor.data.width() as u16 {
                return Ok(None);
            }
            match readline.edit_mode {
                Mode::Insert => readline.editor.insert(Grapheme::from(*ch)),
                Mode::Overwrite => readline.editor.overwrite(Grapheme::from(*ch)),
            }
        }

        Ok(None)
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn render_static<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        // Render the label.
        crossterm::execute!(
            out,
            style::SetForegroundColor(state.label_color),
            style::Print(state.label.to_owned()),
            style::SetForegroundColor(style::Color::Reset),
        )
    }

    pub fn render<W: io::Write>(&self, out: &mut W, state: &State) -> Result<()> {
        let (mut prev, mut next) = (state.prev.clone(), state.next.clone());
        if prev.data == next.data {
            return Ok(());
        }

        // Masking.
        prev.data = match &state.mask {
            None => prev.data.clone(),
            Some(mask) => prev
                .data
                .iter()
                .map(|_| mask.clone())
                .collect::<Graphemes>(),
        };
        next.data = match &state.mask {
            None => next.data.clone(),
            Some(mask) => next
                .data
                .iter()
                .map(|_| mask.clone())
                .collect::<Graphemes>(),
        };

        // Go backward/forward to the position of lcp.
        let lcp = prev.data.longest_common_prefix(&next.data);
        if lcp.width() > prev.width_to_position() {
            termutil::move_right(out, (lcp.width() - prev.width_to_position()) as u16)?;
        } else {
            termutil::move_left(out, (prev.width_to_position() - lcp.width()) as u16)?;
        }

        // Render the suffix of next buffer.
        let mut input = next
            .data
            .iter()
            .enumerate()
            .filter(|&(i, _)| i >= lcp.len())
            .fold(Graphemes::default(), |mut g, (_, ch)| {
                g.push(ch.clone());
                g
            });

        // FromCursorDown remove the last char
        // if the cursor is at the end of line.
        // Therefore, put the space the last of input string.
        input.push(Grapheme::from(' '));
        crossterm::execute!(
            out,
            style::Print(input),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        // Go backward to the next position from the end of graphemes.
        // +1 is for input.push(Grapheme::from(' ')) step above.
        termutil::move_left(out, next.width_from_position() as u16 + 1)
    }
}

pub struct Store {
    pub state: State,
    pub handler: EventHandler,
    pub renderer: Renderer,
    pub title_store: Option<text::Store>,
}

impl Controller for Store {
    fn can_render(&self) -> Result<()> {
        let title_lines = self
            .title_store
            .as_ref()
            .map_or(Ok(0), |s| s.state.text_lines())?;
        if terminal::size()?.1 < title_lines {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Terminal does not leave the space to render.",
            ));
        }
        Ok(())
    }

    fn handle_event(&mut self, ev: &Event, out: &mut io::Stdout) -> Result<Option<String>> {
        if let Event::Resize(_, _) = ev {
            // Overwrite the prev as default.
            self.state.prev = Buffer::default();
            Ok(None)
        } else {
            self.handler.handle_event(ev, out, &mut self.state)
        }
    }

    fn render_static(&self, out: &mut io::Stdout) -> Result<()> {
        // Render title
        if let Some(ref title_store) = self.title_store {
            title_store.renderer.render(out, &title_store.state)?;
            termutil::move_down(out, 1)?;
        }
        // Render label.
        self.renderer.render_static(out, &self.state)
    }

    fn finalize(&mut self, out: &mut io::Stdout) -> Result<()> {
        termutil::move_down(out, 1)?;
        if let Some(hstr) = &mut self.state.hstr {
            hstr.register(self.state.editor.data.clone());
        }
        self.state.editor = Buffer::default();
        self.state.prev = Buffer::default();
        self.state.next = Buffer::default();
        Ok(())
    }

    fn render(&mut self, out: &mut io::Stdout) -> Result<()> {
        self.state.next = self.state.editor.clone();
        self.renderer.render(out, &self.state)?;
        self.state.prev = self.state.editor.clone();
        Ok(())
    }
}
