use crate::{
    crossterm::{
        event::Event,
        style::{Attribute, Attributes, Color, ContentStyle},
    },
    error::Result,
    json::{self, JsonNode, JsonPath, JsonStream},
    snapshot::Snapshot,
    style::StyleBuilder,
    switch::ActiveKeySwitcher,
    text, EventHandler, Prompt, Renderer,
};

pub mod keymap;
pub mod render;

/// Represents a JSON preset for rendering JSON data and titles with customizable styles.
pub struct Json {
    keymap: ActiveKeySwitcher<keymap::Keymap>,
    title_state: text::State,
    json_state: json::State,
}

impl Json {
    pub fn new(stream: JsonStream) -> Self {
        Self {
            title_state: text::State {
                text: Default::default(),
                style: StyleBuilder::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            json_state: json::State {
                stream,
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
            keymap: ActiveKeySwitcher::new("default", self::keymap::default),
        }
    }

    /// Sets the title text for the JSON preset.
    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_state.text = text.as_ref().to_string();
        self
    }

    /// Sets the style for the title text.
    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_state.style = style;
        self
    }

    /// Sets the number of lines to be used for rendering the JSON data.
    pub fn json_lines(mut self, lines: usize) -> Self {
        self.json_state.theme.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the JSON data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.json_state.theme.indent = indent;
        self
    }

    /// Sets the attribute for active (currently selected) items.
    pub fn active_item_attribute(mut self, attr: Attribute) -> Self {
        self.json_state.theme.active_item_attribute = attr;
        self
    }

    /// Sets the attribute for inactive (not currently selected) items.
    pub fn inactive_item_attribute(mut self, attr: Attribute) -> Self {
        self.json_state.theme.inactive_item_attribute = attr;
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

    /// Creates a prompt based on the current configuration of the `Json` instance.
    pub fn prompt(self) -> Result<Prompt<(JsonNode, Option<JsonPath>)>> {
        Prompt::try_new(
            Box::new(self::render::Renderer {
                keymap: self.keymap,
                title_snapshot: Snapshot::<text::State>::new(self.title_state),
                json_snapshot: Snapshot::<json::State>::new(self.json_state),
            }),
            Box::new(
                |event: &Event, renderer: &mut Box<dyn Renderer + 'static>| {
                    let renderer = self::render::Renderer::cast_mut(renderer.as_mut())?;
                    renderer.keymap.get()(event, renderer)
                },
            ),
            |renderer: &(dyn Renderer + '_)| -> Result<(JsonNode, Option<JsonPath>)> {
                Ok(self::render::Renderer::cast(renderer)?
                    .json_snapshot
                    .after()
                    .stream
                    .current_root_and_path_from_root())
            },
        )
    }
}
