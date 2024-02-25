use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    json::{self, JsonNode, JsonPathSegment},
    render::{Renderable, State},
    style::StyleBuilder,
    text, Prompt,
};

/// Represents a JSON preset for rendering JSON data and titles with customizable styles.
pub struct Json {
    title_renderer: text::Renderer,
    json_renderer: json::Renderer,
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
                json: json::JsonTree::new(root),
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
                lines: Default::default(),
                indent: 2,
            },
        }
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
        self.json_renderer.lines = Some(lines);
        self
    }

    /// Sets the indentation level for rendering the JSON data.
    pub fn indent(mut self, indent: usize) -> Self {
        self.json_renderer.indent = indent;
        self
    }

    /// Creates a prompt based on the current configuration of the `Json` instance.
    pub fn prompt(self) -> Result<Prompt<Vec<JsonPathSegment>>> {
        Prompt::try_new(
            vec![
                Box::new(State::<text::Renderer>::new(self.title_renderer)),
                Box::new(State::<json::Renderer>::new(self.json_renderer)),
            ],
            |_, _| Ok(true),
            |renderables: &Vec<Box<dyn Renderable + 'static>>| -> Result<Vec<JsonPathSegment>> {
                Ok(renderables[1]
                    .as_any()
                    .downcast_ref::<State<json::Renderer>>()
                    .unwrap()
                    .after
                    .borrow()
                    .json
                    .get())
            },
        )
    }
}
