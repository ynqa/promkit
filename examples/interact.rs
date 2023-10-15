use std::iter::FromIterator;

use anyhow::Result;

use promkit::{
    crossterm::style::Color,
    item_box::ItemBox,
    style::ContentStyleBuilder,
    widgets::{
        ItemPicker, ItemPickerBuilder, State, Text, TextBuilder, TextEditor, TextEditorBuilder,
        Widget,
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
            .build_state()?,
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
            .build_state()?,
        ItemPickerBuilder::new(ItemBox::from_iter(0..100))
            .cursor_style(
                ContentStyleBuilder::new()
                    .foreground_color(Color::Magenta)
                    .build(),
            )
            .lines(10)
            .build_state()?,
        TextBuilder::empty().build_state()?,
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
        let hinttext_state = widgets[3].as_any().downcast_ref::<State<Text>>().unwrap();

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
                    if !query.is_empty() {
                        itempucker_state.after.borrow_mut().itembox = ItemBox::default();
                    } else {
                        *itempucker_state.after.borrow_mut() = itempucker_state.init.clone();
                    }
                }
            }
        }
        let finalizable = !itempucker_state.output().is_empty();
        if !finalizable {
            *hinttext_state.after.borrow_mut() = TextBuilder::new("Put number under 99")
                .style(
                    ContentStyleBuilder::new()
                        .foreground_color(Color::Red)
                        .build(),
                )
                .build()?;
        } else {
            *hinttext_state.after.borrow_mut() = hinttext_state.init.clone();
        }
        Ok(finalizable)
    }))
    .build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
