use std::fmt::Display;

use crate::{
    checkbox,
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    snapshot::Snapshot,
    style::StyleBuilder,
    text, EventHandler, Prompt, PromptSignal, Renderer,
};

pub mod keymap;
pub mod render;

/// Represents a checkbox component for creating
/// and managing a list of selectable options.
pub struct Checkbox {
    /// Renderer for the title displayed above the checkbox list.
    title_renderer: text::Renderer,
    /// Renderer for the checkbox list itself.
    checkbox_renderer: checkbox::Renderer,
    keymap: KeymapManager<self::render::Renderer>,
    enable_mouse_scroll: bool,
}

impl Checkbox {
    /// Constructs a new `Checkbox` instance with a list of items
    /// to be displayed as selectable options.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items
    /// that implement the `Display` trait, to be used as options.
    pub fn new<T: Display, I: IntoIterator<Item = T>>(items: I) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            checkbox_renderer: checkbox::Renderer {
                checkbox: checkbox::Checkbox::from_iter(items),
                cursor: String::from("❯ "),
                active_mark: '•',
                inactive_mark: '◦',
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
            keymap: KeymapManager::new("default", self::keymap::default),
            enable_mouse_scroll: false,
        }
    }

    /// Enables mouse scroll functionality for the component.
    /// When enabled, users can scroll through the items of list using the mouse wheel.
    pub fn enable_mouse_scroll(mut self) -> Self {
        self.enable_mouse_scroll = true;
        self
    }

    /// Sets the title text displayed above the checkbox list.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Sets the cursor symbol used to indicate the current selection.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.checkbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the mark symbol used to indicate selected items.
    pub fn active_mark(mut self, mark: char) -> Self {
        self.checkbox_renderer.active_mark = mark;
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.checkbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the checkbox list.
    pub fn checkbox_lines(mut self, lines: usize) -> Self {
        self.checkbox_renderer.lines = Some(lines);
        self
    }

    pub fn register_keymap<K: AsRef<str>>(
        mut self,
        key: K,
        handler: EventHandler<self::render::Renderer>,
    ) -> Self {
        self.keymap = self.keymap.register(key, handler);
        self
    }

    /// Displays the checkbox prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is a list of selected options.
    pub fn prompt(self) -> Result<Prompt<Vec<String>>> {
        Prompt::try_new(
            Box::new(self::render::Renderer {
                title_snapshot: Snapshot::<text::Renderer>::new(self.title_renderer),
                checkbox_snapshot: Snapshot::<checkbox::Renderer>::new(self.checkbox_renderer),
                keymap: self.keymap,
            }),
            Box::new(
                |event: &Event, renderer: &mut Box<dyn Renderer + 'static>| {
                    let renderer = self::render::Renderer::cast_mut(renderer.as_mut())?;
                    match renderer.keymap.get() {
                        Some(f) => f(event, renderer),
                        None => Ok(PromptSignal::Quit),
                    }
                },
            ),
            |renderer: &(dyn Renderer + '_)| -> Result<Vec<String>> {
                Ok(self::render::Renderer::cast(renderer)?
                    .checkbox_snapshot
                    .after()
                    .checkbox
                    .get())
            },
            self.enable_mouse_scroll,
        )
    }
}
