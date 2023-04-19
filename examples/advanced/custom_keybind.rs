use std::io;

use promkit::{
    build::Builder,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    grapheme::Graphemes,
    keybind::KeyBind,
    readline::{self, State},
    EventHandleFn, Result,
};

fn main() -> Result<()> {
    let mut b = KeyBind::default();
    b.assign(vec![(
        Event::Key(KeyEvent {
            code: KeyCode::Char('r'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }),
        Box::new(|_, _, _: &mut io::Stdout, state: &mut State| {
            state.0.editor.replace(&Graphemes::from("REPLCED!!"));
            Ok(false)
        }) as Box<EventHandleFn<State>>,
    )]);
    let mut p = readline::Builder::default().handler(b).build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
