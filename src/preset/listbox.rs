use std::fmt::Display;

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    listbox,
    snapshot::Snapshot,
    style::StyleBuilder,
    text, EventHandler, Prompt, PromptSignal, Renderer,
};

pub mod keymap;
pub mod render;

/// A component for creating and managing a selectable list of options.
pub struct Listbox {
    keymap: KeymapManager<self::render::Renderer>,
    /// Renderer for the title displayed above the selectable list.
    title_renderer: text::Renderer,
    /// Renderer for the selectable list itself.
    listbox_renderer: listbox::Renderer,
    enable_mouse_scroll: bool,
}

impl Listbox {
    /// Constructs a new `Listbox` instance
    /// with a list of items to be displayed as selectable options.
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
            listbox_renderer: listbox::Renderer {
                listbox: listbox::Listbox::from_iter(items),
                cursor: String::from("❯ "),
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

    /// Sets the title text displayed above the selectable list.
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
        self.listbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines to be used for displaying the selectable list.
    pub fn listbox_lines(mut self, lines: usize) -> Self {
        self.listbox_renderer.lines = Some(lines);
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

    /// Displays the select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> Result<Prompt<String>> {
        Prompt::try_new(
            Box::new(self::render::Renderer {
                keymap: self.keymap,
                title_snapshot: Snapshot::<text::Renderer>::new(self.title_renderer),
                listbox_snapshot: Snapshot::<listbox::Renderer>::new(self.listbox_renderer),
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
            |renderer: &(dyn Renderer + '_)| -> Result<String> {
                Ok(self::render::Renderer::cast(renderer)?
                    .listbox_snapshot
                    .after()
                    .listbox
                    .get())
            },
            self.enable_mouse_scroll,
        )
    }
}
