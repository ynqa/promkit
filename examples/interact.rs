use std::iter::FromIterator;

use anyhow::Result;

use promkit::{
    crossterm::style::Color,
    item_box::ItemBox,
    style::ContentStyleBuilder,
    widgets::{
        ItemPicker, ItemPickerBuilder, State, TextBuilder, TextEditor, TextEditorBuilder, Widget,
    },
    PromptBuilder,
};

fn main() -> Result<()> {
    let mut p = PromptBuilder::new(vec![
        TextBuilder::new("Type Here")
            .style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Green)
                    .build(),
            )
            .build()?,
        TextEditorBuilder::default()
            .style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::DarkYellow)
                    .build(),
            )
            .cursor_style(
                ContentStyleBuilder::new()
                    .background_color(Color::DarkBlue)
                    .build(),
            )
            .label_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::DarkGreen)
                    .build(),
            )
            .disable_history()
            .build()?,
        ItemPickerBuilder::new(ItemBox::from_iter(0..100))
            .cursor_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Magenta)
                    .build(),
            )
            .lines(10)
            .build()?,
    ])
    .posthandle(Box::new(|widgets: &Vec<Box<dyn Widget>>| -> Result<()> {
        if let Some(state) = widgets[1].as_any().downcast_ref::<State<TextEditor>>() {
            if state.before.textbuffer.content() != state.after.borrow().textbuffer.content() {
                let query = state.after.borrow().output();
                if let Some(state) = widgets[2].as_any().downcast_ref::<State<ItemPicker>>() {
                    state.after.borrow_mut().itembox.position = 0;
                    match query.parse::<usize>() {
                        Ok(query) => {
                            state.after.borrow_mut().itembox.list = state
                                .init
                                .itembox
                                .list
                                .iter()
                                .filter(|num| query <= num.parse::<usize>().unwrap_or_default())
                                .map(|num| num.to_string())
                                .collect::<Vec<String>>();
                        }
                        Err(_) => {
                            *state.after.borrow_mut() = state.init.clone();
                        }
                    }
                }
            }
        }
        Ok(())
    }))
    .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
