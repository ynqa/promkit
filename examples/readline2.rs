use std::iter::FromIterator;

use anyhow::Result;

use promkit::{
    crossterm::{
        event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
        style::Color,
    },
    item_box::ItemBox,
    style::ContentStyleBuilder,
    widgets::{ItemPicker, ItemPickerBuilder, TextBuilder, TextEditor, TextEditorBuilder, Widget},
    Prompt,
};

fn main() -> Result<()> {
    let mut p = Prompt::new_with_posthandle(
        vec![
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
        ],
        Box::new(
            |event: &Event, widgets: &mut Vec<Box<dyn Widget>>| -> Result<()> {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(_),
                        modifiers: KeyModifiers::NONE,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    })
                    | Event::Key(KeyEvent {
                        code: KeyCode::Char(_),
                        modifiers: KeyModifiers::SHIFT,
                        kind: KeyEventKind::Press,
                        state: KeyEventState::NONE,
                    }) => {
                        if let Some(texteditor) = widgets[1].as_any().downcast_ref::<TextEditor>() {
                            let query = texteditor.output();
                            if let Ok(query) = query.parse::<usize>() {
                                if let Some(mut picker) =
                                    widgets[2].as_any_mut().downcast_mut::<ItemPicker>()
                                {
                                    picker.itembox.position = 0;
                                    picker.itembox.list = picker
                                        .itembox
                                        .list
                                        .iter()
                                        .filter(|num| {
                                            query <= num.parse::<usize>().unwrap_or_default()
                                        })
                                        .map(|num| num.to_string())
                                        .collect::<Vec<String>>()
                                }
                            }
                        }
                    }
                    _ => (),
                }
                Ok(())
            },
        ),
    );
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
