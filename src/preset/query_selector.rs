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
    text_editor::{self, Mode, Suggest},
    EventHandler, Prompt, Renderer,
};

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
                suggest: Default::default(),
                keymap: KeymapManager::new("default", text_editor::keymap::default_keymap),
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
                keymap: KeymapManager::new("default", listbox::keymap::default_keymap),
                cursor: String::from("❯ "),
                active_item_style: StyleBuilder::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: StyleBuilder::new().build(),
                lines: Default::default(),
            },
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

    /// Enables suggestion functionality in the text editor component with the specified suggestion configuration.
    pub fn enable_suggest(mut self, suggest: Suggest) -> Self {
        self.text_editor_renderer.suggest = suggest;
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
        handler: EventHandler<text_editor::Renderer>,
    ) -> Self {
        self.text_editor_renderer.keymap = self.text_editor_renderer.keymap.register(key, handler);
        self
    }

    /// Displays the query select prompt and waits for user input.
    /// Returns a `Result` containing the `Prompt` result,
    /// which is the selected option.
    pub fn prompt(self) -> Result<Prompt<String>> {
        let filter = self.filter;

        Prompt::try_new(
            vec![
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<text_editor::Renderer>::new(
                    self.text_editor_renderer,
                )),
                Box::new(Snapshot::<listbox::Renderer>::new(self.listbox_renderer)),
            ],
            move |_: &Event, renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<bool> {
                let text_snapshot = Snapshot::<text_editor::Renderer>::cast(renderers[1].as_ref())?;
                let list_snapshot = Snapshot::<listbox::Renderer>::cast(renderers[2].as_ref())?;

                if text_snapshot.compare_states(|before, after| {
                    before.texteditor.text() != after.texteditor.text()
                }) {
                    let query = text_snapshot
                        .borrow_after()
                        .texteditor
                        .text_without_cursor()
                        .to_string();

                    let list = filter(&query, list_snapshot.init().listbox.items());
                    list_snapshot.borrow_mut_after().listbox = Listbox::from_iter(list);
                }
                Ok(true)
            },
            |renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<String> {
                Ok(
                    Snapshot::<listbox::Renderer>::cast_and_borrow_after(renderers[2].as_ref())?
                        .listbox
                        .get(),
                )
            },
            self.enable_mouse_scroll,
        )
    }
}
