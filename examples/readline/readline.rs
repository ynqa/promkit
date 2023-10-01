use anyhow::Result;

use promkit::{editor::text::TextEditor, Prompt};

fn main() -> Result<()> {
    let texteditor1 = TextEditor::new();
    let texteditor2 = TextEditor::new();
    let mut p = Prompt::new(vec![Box::new(texteditor1), Box::new(texteditor2)]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
