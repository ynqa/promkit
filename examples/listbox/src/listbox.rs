use promkit::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton,
                MouseEvent, MouseEventKind,
            },
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    widgets::{
        listbox::{self, Listbox, ListboxHit},
        text::{self, Text},
    },
    Prompt, Signal, TerminalModes, TerminalSession,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Index {
    Title,
    List,
}

struct ListboxPrompt {
    renderer: Option<SharedRenderer<Index>>,
    title: text::State,
    list: listbox::State,
}

impl ListboxPrompt {
    fn new(items: impl IntoIterator<Item = impl std::fmt::Display>) -> Self {
        Self {
            renderer: None,
            title: text::State {
                text: Text::from("What number do you like?"),
                config: text::Config {
                    style: Some(ContentStyle {
                        attributes: Attributes::from(Attribute::Bold),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
            list: listbox::State {
                listbox: Listbox::from(items),
                config: listbox::Config {
                    cursor: "❯ ".into(),
                    active_item_style: Some(ContentStyle {
                        foreground_color: Some(Color::DarkCyan),
                        ..Default::default()
                    }),
                    inactive_item_style: Some(ContentStyle::default()),
                    lines: None,
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
                self.list.listbox.backward();
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
                self.list.listbox.forward();
            }
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => self.select_at(*column, *row),
            _ => {}
        }
        Ok(Signal::Continue)
    }

    fn select_at(&mut self, column: u16, row: u16) {
        let Some(position) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
            .filter(|position| position.index == Index::List)
        else {
            return;
        };
        if let Some(ListboxHit::Select { index }) = self.list.hit_at(position.content_position()) {
            self.list.listbox.move_to(index);
        }
    }

    async fn render(&self) -> anyhow::Result<()> {
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Renderer not initialized"))?;
        renderer
            .update([
                (Index::Title, self.title.create_graphemes()),
                (Index::List, self.list.create_graphemes()),
            ])
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for ListboxPrompt {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                [
                    (Index::Title, self.title.create_graphemes()),
                    (Index::List, self.list.create_graphemes()),
                ],
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = self.handle_event(event);
        self.render().await?;
        signal
    }

    type Return = String;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self.list.listbox.get().to_string())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = {
        let modes =
            TerminalModes::RAW_MODE | TerminalModes::HIDDEN_CURSOR | TerminalModes::MOUSE_CAPTURE;
        let _terminal_session = TerminalSession::try_new(modes)?;
        ListboxPrompt::new(0..100).run().await?
    };
    println!("result: {:?}", ret);
    Ok(())
}
