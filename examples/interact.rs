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
    .evaluate(Box::new(|widgets: &Vec<Box<dyn Widget>>| -> Result<bool> {
        let texteditor_state = widgets[1]
            .as_any()
            .downcast_ref::<State<TextEditor>>()
            .unwrap();
        let itempucker_state = widgets[2]
            .as_any()
            .downcast_ref::<State<ItemPicker>>()
            .unwrap();

        if texteditor_state.before.textbuffer.content()
            != texteditor_state.after.borrow().textbuffer.content()
        {
            let query = texteditor_state.after.borrow().output();
            itempucker_state.after.borrow_mut().itembox.position = 0;
            match query.parse::<usize>() {
                Ok(query) => {
                    itempucker_state.after.borrow_mut().itembox.list = itempucker_state
                        .init
                        .itembox
                        .list
                        .iter()
                        .filter(|num| query <= num.parse::<usize>().unwrap_or_default())
                        .map(|num| num.to_string())
                        .collect::<Vec<String>>();
                }
                Err(_) => {
                    *itempucker_state.after.borrow_mut() = itempucker_state.init.clone();
                }
            }
        }
        Ok(!itempucker_state.output().is_empty())
    }))
    .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
