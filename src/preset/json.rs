use crate::{
    crossterm::style::{Attribute, Attributes, Color, ContentStyle},
    error::Result,
    json::{self, JsonNode, JsonPathSegment},
    render::{Renderable, State},
    style::Style,
    text, Prompt,
};

pub struct Json {
    title_renderer: text::Renderer,
    json_renderer: json::Renderer,
}

impl Json {
    pub fn new(root: JsonNode) -> Self {
        Self {
            title_renderer: text::Renderer {
                text: Default::default(),
                style: Style::new()
                    .attrs(Attributes::from(Attribute::Bold))
                    .build(),
            },
            json_renderer: json::Renderer {
                json: json::Json::new(root),
                active_item_style: Style::new().fgc(Color::DarkCyan).build(),
                inactive_item_style: Style::new().build(),
                lines: Default::default(),
            },
        }
    }

    pub fn title<T: AsRef<str>>(mut self, text: T) -> Self {
        self.title_renderer.text = text.as_ref().to_string();
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_renderer.style = style;
        self
    }

    pub fn active_item_style(mut self, style: ContentStyle) -> Self {
        self.json_renderer.active_item_style = style;
        self
    }

    pub fn inactive_item_style(mut self, style: ContentStyle) -> Self {
        self.json_renderer.inactive_item_style = style;
        self
    }

    pub fn json_lines(mut self, lines: usize) -> Self {
        self.json_renderer.lines = Some(lines);
        self
    }

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
