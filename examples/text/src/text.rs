use promkit::{
    core::{
        crossterm::event::{
            Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
        },
        render::{Renderer, SharedRenderer},
        Widget,
    },
    widgets::text::{self, Text},
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
                ..
            }) => return Ok(Signal::Quit),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                ..
            }) => return Err(anyhow::anyhow!("ctrl+c")),
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
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
                ..
            })
            | Event::Mouse(MouseEvent {
                kind: MouseEventKind::ScrollDown,
                modifiers: KeyModifiers::NONE,
                ..
            }) => {
                self.text.text.forward();
            }
            _ => {}
        }
        Ok(Signal::Continue)
    }

    async fn render(&self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("renderer not initialized"))?;
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
        let signal = self.handle_event(event)?;
        self.render().await?;
        Ok(signal)
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
