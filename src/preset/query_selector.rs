use std::{fmt::Display, iter::FromIterator};

use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    keymap::KeymapManager,
    listbox::{self, Listbox},
    snapshot::Snapshot,
    style::StyleBuilder,
    text,
    text_editor::{self, Mode},
    EventHandler, Prompt, PromptSignal, Renderer,
};

pub mod keymap;
pub mod render;

/// Used to process and filter a list of options
/// based on the input text in the `QuerySelector` component.
type Filter = dyn Fn(&str, &Vec<String>) -> Vec<String>;

/// Represents a query selection component that combines a text editor
/// for input and a list box
/// for displaying filtered options based on the input.
pub struct QuerySelector {
    /// Renderer for the title displayed above the query selection.
    title_renderer: text::Renderer,
    /// Renderer for the text editor component.
    text_editor_renderer: text_editor::Renderer,
    /// Renderer for the list box component.
    listbox_renderer: listbox::Renderer,
    keymap: KeymapManager<self::render::Renderer>,

    /// A filter function to apply to the list box items
    /// based on the text editor input.
    filter: Box<Filter>,
    enable_mouse_scroll: bool,
}

impl QuerySelector {
    /// Constructs a new `QuerySelector` instance
    /// with a list of items and a filter function.
    ///
    /// # Arguments
    ///
    /// * `items` - An iterator over items that implement the `Display` trait,
    /// to be used as options in the list box.
    /// * `filter` - A function that takes the current input
    /// from the text editor and the list of items,
    /// returning a filtered list of items to display.
    pub fn new<T, I, F>(items: I, filter: F) -> Self
    where
        T: Display,
        I: IntoIterator<Item = T>,
        F: Fn(&str, &Vec<String>) -> Vec<String> + 'static,
    {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            text_editor_renderer: text_editor::Renderer {
                texteditor: Default::default(),
                history: None,
                prefix: String::from("❯❯ "),
                mask: None,
                prefix_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                active_char_style: StyleBuilder::new().bgc(Color::DarkCyan).build(),
                inactive_char_style: StyleBuilder::new().build(),
                edit_mode: Default::default(),
                lines: Default::default(),
            },
            listbox_renderer: listbox::Renderer {
                listbox: Listbox::from_iter(items),
                cursor: String::from("❯ "),
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
            keymap: KeymapManager::new("default", self::keymap::default),
            filter: Box::new(filter),
            enable_mouse_scroll: false,
        }
    }

    /// Enables mouse scroll functionality for the component.
    /// When enabled, users can scroll through the items of list using the mouse wheel.
    pub fn enable_mouse_scroll(mut self) -> Self {
        self.enable_mouse_scroll = true;
        self
    }

    /// Sets the title text displayed above the query selection.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Sets the prefix string displayed before the input text in the text editor component.
    pub fn prefix<T: AsRef<str>>(mut self, prefix: T) -> Self {
        self.text_editor_renderer.prefix = prefix.as_ref().to_string();
        self
    }

    /// Sets the style for the prefix string in the text editor component.
    pub fn prefix_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.prefix_style = style;
        self
    }

    /// Sets the style for the active character (the character at the cursor position) in the text editor component.
    pub fn active_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.active_char_style = style;
        self
    }

    /// Sets the style for inactive characters (characters not at the cursor position) in the text editor component.
    pub fn inactive_char_style(mut self, style: ContentStyle) -> Self {
        self.text_editor_renderer.inactive_char_style = style;
        self
    }

    /// Sets the editing mode for the text editor component.
    pub fn edit_mode(mut self, mode: Mode) -> Self {
        self.text_editor_renderer.edit_mode = mode;
        self
    }

    /// Sets the number of lines available for the text editor component.
    pub fn text_editor_lines(mut self, lines: usize) -> Self {
        self.text_editor_renderer.lines = Some(lines);
        self
    }

    /// Sets the cursor symbol used in the list box component.
    pub fn cursor<T: AsRef<str>>(mut self, cursor: T) -> Self {
        self.listbox_renderer.cursor = cursor.as_ref().to_string();
        self
    }

    /// Sets the style for active (currently selected) items in the list box component.
    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.active_item_style = style;
        self
    }

    /// Sets the style for inactive (not currently selected) items in the list box component.
    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.listbox_renderer.inactive_item_style = style;
        self
    }

    /// Sets the number of lines available for the list box component.
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

    /// Displays the query select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            Box::new(self::render::Renderer {
                title_snapshot: Snapshot::<text::Renderer>::new(self.title_renderer),
                text_editor_snapshot: Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                ),
                listbox_snapshot: Snapshot::<listbox::Renderer>::new(self.listbox_renderer),
                keymap: self.keymap,
            }),
            Box::new(
                move |event: &Event,
                      renderer: &mut Box<dyn Renderer + 'static>|
                      -> Result<PromptSignal> {
                    let renderer = self::render::Renderer::cast_mut(renderer.as_mut())?;
                    let signal = match renderer.keymap.get() {
                        Some(f) => f(event, renderer),
                        None => Ok(PromptSignal::Quit),
                    };

                    if renderer.text_editor_snapshot.after().texteditor.text()
                        != renderer
                            .text_editor_snapshot
                            .borrow_before()
                            .texteditor
                            .text()
                    {
                        let query = renderer
                            .text_editor_snapshot
                            .after()
                            .texteditor
                            .text_without_cursor()
                            .to_string();

                        let list = filter(&query, renderer.listbox_snapshot.init().listbox.items());
                        renderer.listbox_snapshot.after_mut().listbox = Listbox::from_iter(list);
                    }
                    signal
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
