use promkit::{
    core::{
        crossterm::{
            event::{
                Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
                MouseEventKind,
            },
            style::{Attribute, Attributes, Color, ContentStyle},
        },
        render::{Renderer, SharedRenderer},
        ScreenPosition, Widget,
    },
    widgets::text_editor::{self, TextEditorHit},
    Prompt, Signal,
};

#[derive(Clone, Copy)]
struct Styles {
    prefix: ContentStyle,
    active: ContentStyle,
    inactive: ContentStyle,
}

struct Form {
    renderer: Option<SharedRenderer<usize>>,
    fields: Vec<text_editor::State>,
    styles: Vec<Styles>,
    active: Option<usize>,
}

impl Form {
    fn new(fields: impl IntoIterator<Item = text_editor::State>) -> Self {
        let fields: Vec<_> = fields.into_iter().collect();
        let styles = fields
            .iter()
            .map(|field| Styles {
                prefix: field.config.prefix_style,
                active: field.config.active_char_style,
                inactive: field.config.inactive_char_style,
            })
            .collect();
        let active = (!fields.is_empty()).then_some(0);
        Self {
            renderer: None,
            fields,
            styles,
            active,
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
            Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column,
                row,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.focus_at(*column, *row);
                return Ok(Signal::Continue);
            }
            _ => {}
        }

        let Some(active) = self.active else {
            return Ok(Signal::Continue);
        };

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => self.active = Some(active.saturating_sub(1)),
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.active = Some((active + 1).min(self.fields.len().saturating_sub(1)));
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                Self::edit(&mut self.fields[active], key);
            }
            _ => {}
        }
        Ok(Signal::Continue)
    }

    fn edit(field: &mut text_editor::State, key: &KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Left, KeyModifiers::NONE) => {
                field.texteditor.backward();
            }
            (KeyCode::Right, KeyModifiers::NONE) => {
                field.texteditor.forward();
            }
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => field.texteditor.move_to_head(),
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => field.texteditor.move_to_tail(),
            (KeyCode::Backspace, KeyModifiers::NONE) => field.texteditor.erase(),
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => field.texteditor.erase_all(),
            (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                match field.config.edit_mode {
                    text_editor::Mode::Insert => field.texteditor.insert(ch),
                    text_editor::Mode::Overwrite => field.texteditor.overwrite(ch),
                }
            }
            _ => {}
        }
    }

    fn focus_at(&mut self, column: u16, row: u16) {
        let Some(position) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.hit_test(ScreenPosition { row, column }))
        else {
            return;
        };
        let field_index = position.index;
        if let Some(TextEditorHit::Cursor { index }) = self
            .fields
            .get(field_index)
            .and_then(|field| field.hit_at(position.content_position()))
        {
            self.active = Some(field_index);
            self.fields[field_index].texteditor.move_to(index);
        }
    }

    fn update_focus_styles(&mut self) {
        for (index, field) in self.fields.iter_mut().enumerate() {
            let styles = self.styles[index];
            if Some(index) == self.active {
                field.config.prefix_style = styles.prefix;
                field.config.active_char_style = styles.active;
                field.config.inactive_char_style = styles.inactive;
            } else {
                field.config.prefix_style = ContentStyle {
                    attributes: Attributes::from(Attribute::Dim),
                    ..styles.prefix
                };
                field.config.active_char_style = ContentStyle {
                    attributes: Attributes::from(Attribute::Dim),
                    ..Default::default()
                };
                field.config.inactive_char_style = ContentStyle {
                    attributes: Attributes::from(Attribute::Dim),
                    ..styles.inactive
                };
            }
        }
    }

    async fn render(&mut self) -> anyhow::Result<()> {
        self.update_focus_styles();
        let renderer = self
            .renderer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("renderer not initialized"))?;
        renderer
            .update(
                self.fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| (index, field.create_graphemes())),
            )
            .render()
            .await
    }
}

#[promkit::async_trait::async_trait]
impl Prompt for Form {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        self.update_focus_styles();
        self.renderer = Some(SharedRenderer::new(
            Renderer::try_new_with_graphemes(
                self.fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| (index, field.create_graphemes())),
                true,
            )
            .await?,
        ));
        Ok(())
    }

    async fn evaluate(&mut self, event: &Event) -> anyhow::Result<Signal> {
        let signal = self.handle_event(event)?;
        self.render().await?;
        Ok(signal)
    }

    type Return = Vec<String>;

    fn finalize(&mut self) -> anyhow::Result<Self::Return> {
        Ok(self
            .fields
            .iter()
            .map(|field| field.texteditor.text_without_cursor().to_string())
            .collect())
    }
}

fn field(prefix_color: Color) -> text_editor::State {
    text_editor::State {
        config: text_editor::Config {
            prefix: "❯❯ ".into(),
            prefix_style: ContentStyle {
                foreground_color: Some(prefix_color),
                ..Default::default()
            },
            active_char_style: ContentStyle {
                background_color: Some(Color::DarkCyan),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ret = Form::new([
        field(Color::DarkRed),
        field(Color::DarkGreen),
        field(Color::DarkBlue),
    ])
    .run()
    .await?;
    println!("result: {:?}", ret);
    Ok(())
}
