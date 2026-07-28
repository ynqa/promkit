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
        text::{self, Text},
        yaml::{self, config::OverflowMode, Document, YamlHit},
    },
    Prompt, Signal, TerminalModes, TerminalSession,
};

/// Interactive YAML viewer powered by promkit.
#[derive(Debug, Parser)]
#[command(name = "yaml", version)]
struct Args {
    /// Optional path to a YAML file. Reads from stdin when omitted or when "-" is specified.
    input: Option<PathBuf>,
}

/// Parse a YAML document from a file or stdin based on the provided arguments.
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
    Yaml,
}

struct YamlViewer {
    renderer: Option<SharedRenderer<Index>>,
    title: text::State,
    yaml: yaml::State,
}

impl YamlViewer {
    fn new(document: Document) -> Self {
        Self {
            renderer: None,
            title: text::State {
                text: Text::from("YAML Viewer"),
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            yaml: yaml::State {
                document,
                config: yaml::Config {
                    map_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    sequence_style: ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    },
                    key_style: ContentStyle {
                        foreground_color: Some(Color::DarkBlue),
                        ..Default::default()
                    },
                    tag_style: ContentStyle {
                        foreground_color: Some(Color::DarkYellow),
                        ..Default::default()
                    },
                    string_style: ContentStyle {
                        foreground_color: Some(Color::DarkGreen),
                        ..Default::default()
                    },
                    number_style: ContentStyle::default(),
                    boolean_style: ContentStyle::default(),
                    null_style: ContentStyle {
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
                self.yaml.document.up();
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
                self.yaml.document.down();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            }) => self.yaml.document.toggle(),
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
            .filter(|position| position.index == Index::Yaml)
        else {
            return;
        };
        if let Some(YamlHit::Toggle { row_index }) =
            self.yaml.hit_at_viewport(position.content_position())
        {
            self.yaml.document.toggle_at(row_index);
        }
    }

    fn graphemes(&self) -> anyhow::Result<[(Index, promkit::core::CreatedGraphemes); 2]> {
        let (width, height) = terminal::size()?;
        Ok([
            (Index::Title, self.title.create_graphemes()),
            (
                Index::Yaml,
                self.yaml.create_graphemes_in_viewport(width, height),
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
impl Prompt for YamlViewer {
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

    YamlViewer::new(document).run().await
}
