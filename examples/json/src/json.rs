use std::{
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
};

use clap::Parser;
use promkit::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
                MouseEvent, MouseEventKind,
            },
            style::{Attribute, Attributes, Color, ContentStyle},
            terminal,
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    widgets::{
        json::{self, config::OverflowMode, Document, JsonHit},
        text::{self, Text},
    },
    Prompt, Signal, TerminalModes, TerminalSession,
};

/// Interactive JSON viewer powered by promkit.
#[derive(Debug, Parser)]
#[command(name = "json", version)]
struct Args {
    /// Optional path to a JSON file. Reads from stdin when omitted or when "-" is specified.
    input: Option<PathBuf>,
}

/// Parse a JSON document from a file or stdin based on the provided arguments.
fn parse_document(args: &Args) -> anyhow::Result<Document> {
    match &args.input {
        None => Document::from_reader(io::stdin().lock()).map_err(anyhow::Error::from),
        Some(path) if path == &PathBuf::from("-") => {
            Document::from_reader(io::stdin().lock()).map_err(anyhow::Error::from)
        }
        Some(path) => {
            let file = File::open(path)?;
            Document::from_reader(BufReader::new(file)).map_err(anyhow::Error::from)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Title,
    Json,
}

struct JsonViewer {
    renderer: Option<SharedRenderer<Index>>,
    title: text::State,
    json: json::State,
}

impl JsonViewer {
    fn new(document: Document) -> Self {
        Self {
            renderer: None,
            title: text::State {
                text: Text::from("JSON Viewer"),
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            json: json::State {
                document,
                config: json::Config {
                    curly_brackets_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    square_brackets_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    key_style: ContentStyle {
                        foreground_color: Some(Color::DarkBlue),
                        ..Default::default()
                    },
                    string_value_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    number_value_style: ContentStyle::default(),
                    boolean_value_style: ContentStyle::default(),
                    null_value_style: ContentStyle {
                        foreground_color: Some(Color::DarkGrey),
                        ..Default::default()
                    },
                    active_item_attribute: Attribute::Undercurled,
                    inactive_item_attribute: Attribute::Dim,
                    indent: 2,
                    overflow_mode: OverflowMode::Wrap,
                    lines: None,
                    show_line_numbers: true,
                },
            },
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
                self.json.document.up();
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
                self.json.document.down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.json.document.toggle(),
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => self.toggle_at(*column, *row),
            _ => {}
        }
        Ok(Signal::Continue)
    }

    fn toggle_at(&mut self, column: u16, row: u16) {
        let Some(position) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
            .filter(|position| position.index == Index::Json)
        else {
            return;
        };
        if let Some(JsonHit::Toggle { row_index }) =
            self.json.hit_at_viewport(position.content_position())
        {
            self.json.document.toggle_at(row_index);
        }
    }

    fn graphemes(&self) -> anyhow::Result<[(Index, promkit::core::CreatedGraphemes); 2]> {
        let (width, height) = terminal::size()?;
        Ok([
            (Index::Title, self.title.create_graphemes()),
            (
                Index::Json,
                self.json.create_graphemes_in_viewport(width, height),
            ),
        ])
    }

    async fn render(&mut self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Renderer not initialized"))?;
        renderer.update(self.graphemes()?).render().await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for JsonViewer {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(self.graphemes()?, true).await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = self.handle_event(event);
        self.render().await?;
        signal
    }

    type Return = ();

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let document = parse_document(&args)?;

    let modes = TerminalModes::RAW_MODE
        | TerminalModes::ALTERNATE_SCREEN
        | TerminalModes::HIDDEN_CURSOR
        | TerminalModes::MOUSE_CAPTURE;
    let _terminal_session = TerminalSession::try_new(modes)?;

    JsonViewer::new(document).run().await
}
