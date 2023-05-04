use std::io;

use promkit::{
    build::Builder,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    grapheme::Graphemes,
    keybind::KeyBind,
    readline::{self, State},
    Action, Result, UpstreamContext,
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
        Box::new(
            |_: &mut io::Stdout, _: &UpstreamContext, state: &mut State| {
                state.editor.replace(&Graphemes::from("REPLCED!!"));
                Ok(None)
            },
        ) as Box<Action<State>>,
    )]);
    let mut p = readline::Builder::default().keybind(b).build()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
