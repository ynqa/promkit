use std::{fs::File, io, path::PathBuf};

use clap::Parser;
use promkit::{
    core::{
        crossterm::{
            event::{
                self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
                MouseEvent, MouseEventKind,
            },
            execute, terminal,
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    widgets::table::{CsvOptions, Document, State},
    Prompt, Signal,
};

/// Interactive CSV viewer powered by promkit.
#[derive(Debug, Parser)]
#[command(name = "csv", version)]
struct Args {
    /// Optional path to a CSV file. Reads from stdin when omitted or when "-" is specified.
    input: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Table,
}

struct CsvViewer {
    renderer: Option<SharedRenderer<Index>>,
    table: State,
}

impl CsvViewer {
    fn new(document: Document) -> Self {
        Self {
            renderer: None,
            table: State::new(document),
        }
    }

    fn handle_event(&mut self, event: &Event) -> anyhow::Result<Signal> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Ok(Signal::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => return Err(anyhow::anyhow!("ctrl+c")),
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.table.document.up();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.table.document.down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollLeft,
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollUp,
                modifiers: KeyModifiers::SHIFT,
                ..
            }) => {
                self.table.document.scroll_left();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollRight,
                modifiers: KeyModifiers::NONE,
                ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                modifiers: KeyModifiers::SHIFT,
                ..
            }) => {
                self.table.document.scroll_right();
            }
            _ => {}
        }
        Ok(Signal::Continue)
    }

    async fn render(&mut self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Renderer not initialized"))?;
        let (width, height) = terminal::size()?;
        renderer
            .update([(
                Index::Table,
                self.table.create_graphemes_in_viewport(width, height),
            )])
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for CsvViewer {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [(
                    Index::Table,
                    self.table.create_graphemes_in_viewport(width, height),
                )],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = self.handle_event(event)?;
        if signal == Signal::Continue {
            self.render().await?;
        }
        Ok(signal)
    }

    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

fn parse_document(args: &Args) -> anyhow::Result<Document> {
    let options = CsvOptions::default();
    match &args.input {
        None => {
            let stdin = io::stdin();
            Document::from_csv(stdin.lock(), options).map_err(anyhow::Error::from)
        }
        Some(path) if path == &PathBuf::from("-") => {
            let stdin = io::stdin();
            Document::from_csv(stdin.lock(), options).map_err(anyhow::Error::from)
        }
        Some(path) => Document::from_csv(File::open(path)?, options).map_err(anyhow::Error::from),
    }
}

/// Ensure the terminal is restored to its original state when dropped.
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        );
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let document = parse_document(&args)?;

    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;
    let _terminal_guard = TerminalGuard;

    CsvViewer::new(document).run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewer() -> CsvViewer {
        CsvViewer::new(
            Document::from_csv(
                "a,b,c\none,two,three\nfour,five,six\n".as_bytes(),
                CsvOptions::default(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn arrow_keys_move_on_both_axes() {
        let mut viewer = viewer();

        viewer
            .handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Down,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(viewer.table.document.position(), 1);

        viewer
            .handle_event(&Event::Key(KeyEvent::new(
                KeyCode::Right,
                KeyModifiers::NONE,
            )))
            .unwrap();
        assert_eq!(viewer.table.document.first_column(), 1);
    }

    #[test]
    fn shifted_mouse_wheel_scrolls_horizontally() {
        let mut viewer = viewer();
        viewer
            .handle_event(&Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::SHIFT,
            }))
            .unwrap();

        assert_eq!(viewer.table.document.first_column(), 1);
    }

    #[test]
    fn loads_the_large_csv_fixture_from_a_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../promkit-widgets/benches/table.csv");
        let document = parse_document(&Args { input: Some(path) }).unwrap();

        assert_eq!(document.row_count(), 47_852);
        assert_eq!(document.column_count(), 12);
    }
}
