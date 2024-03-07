use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    json::{self, JsonNode, JsonPathSegment},
    keymap::KeymapManager,
    snapshot::Snapshot,
    style::StyleBuilder,
    text, Prompt, Renderer,
};

/// Represents a JSON preset for rendering JSON data and titles with customizable styles.
pub struct Json {
    title_renderer: text::Renderer,
    json_renderer: json::Renderer,
    enable_mouse_scroll: bool,
}

impl Json {
    /// Creates a new `Json` instance with a specified root JSON node.
    ///
    /// # Arguments
    ///
    /// * `root` - A `JsonNode` that represents the root of the JSON data to be rendered.
    pub fn new(root: JsonNode) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            json_renderer: json::Renderer {
                json: json::Json::new(root),
                keymap: KeymapManager::new("default", json::keymap::default_keymap),
                theme: json::Theme {
                    curly_brackets_style: StyleBuilder::new()
                        .attrs(Attributes::from(Attribute::Bold))
                        .build(),
                    square_brackets_style: StyleBuilder::new()
                        .attrs(Attributes::from(Attribute::Bold))
                        .build(),
                    key_style: StyleBuilder::new().fgc(Color::DarkBlue).build(),
                    string_value_style: StyleBuilder::new().fgc(Color::DarkGreen).build(),
                    number_value_style: StyleBuilder::new().build(),
                    boolean_value_style: StyleBuilder::new().build(),
                    null_value_style: StyleBuilder::new().fgc(Color::DarkGrey).build(),
                    active_item_attribute: Attribute::Undercurled,
                    inactive_item_attribute: Attribute::Dim,
                    lines: Default::default(),
                    indent: 2,
                },
            },
            enable_mouse_scroll: false,
        }
    }

    /// Enables mouse scroll functionality for the component.
    /// When enabled, users can scroll through the items of list using the mouse wheel.
    pub fn enable_mouse_scroll(mut self) -> Self {
        self.enable_mouse_scroll = true;
        self
    }

    /// Sets the title text for the JSON preset.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    /// Sets the number of lines to be used for rendering the JSON data.
    pub fn json_lines(mut self, lines: usize) -> Self {
        self.json_renderer.theme.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the JSON data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.json_renderer.theme.indent = indent;
        self
    }

    /// Sets the attribute for active (currently selected) items.
    pub fn active_item_attribute(mut self, attr: Attribute) -> Self {
        self.json_renderer.theme.active_item_attribute = attr;
        self
    }

    /// Sets the attribute for inactive (not currently selected) items.
    pub fn inactive_item_attribute(mut self, attr: Attribute) -> Self {
        self.json_renderer.theme.inactive_item_attribute = attr;
        self
    }

    /// Creates a prompt based on the current configuration of the `Json` instance.
    pub fn prompt(self) -> Result<Prompt<Vec<JsonPathSegment>>> {
        Prompt::try_new(
            vec![
                Box::new(Snapshot::<text::Renderer>::new(self.title_renderer)),
                Box::new(Snapshot::<json::Renderer>::new(self.json_renderer)),
            ],
            |_, _| Ok(true),
            |renderers: &Vec<Box<dyn Renderer + 'static>>| -> Result<Vec<JsonPathSegment>> {
                Ok(
                    Snapshot::<json::Renderer>::cast_and_borrow_after(renderers[1].as_ref())?
                        .json
                        .get(),
                )
            },
            self.enable_mouse_scroll,
        )
    }
}
