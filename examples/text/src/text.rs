use promkit::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
            MouseEvent, MouseEventKind,
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    widgets::text::{self, Text, TextHit},
    Prompt, Signal,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Text,
}

struct TextPrompt {
    renderer: Option<SharedRenderer<Index>>,
    text: text::State,
}

impl TextPrompt {
    fn new(value: impl AsRef<str>) -> Self {
        Self {
            renderer: None,
            text: text::State {
                text: Text::from(value),
                ..Default::default()
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
                self.text.text.backward();
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
                self.text.text.forward();
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                let renderer = self.renderer.clone();
                if let Some(position) = renderer
                    .as_ref()
                    .and_then(|renderer| {
                        renderer.hit_test(ScreenPosition {
                            row: *row,
                            column: *column,
                        })
                    })
                    .filter(|position| position.index == Index::Text)
                {
                    if let Some(TextHit::Select { index }) =
                        self.text.hit_at(position.content_position())
                    {
                        self.text.text.move_to(index);
                    }
                }
            }
            _ => {}
        }
        Ok(Signal::Continue)
    }

    async fn render(&self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Renderer not initialized"))?;
        renderer
            .update([(Index::Text, self.text.create_graphemes())])
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for TextPrompt {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes([(Index::Text, self.text.create_graphemes())], true)
                .await?,
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
    TextPrompt::new(std::fs::read_to_string("Cargo.toml")?)
        .run()
        .await
}
