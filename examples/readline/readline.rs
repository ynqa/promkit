use anyhow::Result;

use promkit::{editor::text::TextEditor, Prompt};

fn main() -> Result<()> {
    let mut p = Prompt::new(vec![
        Box::new(TextEditor::new()),
        Box::new(TextEditor::new()),
        Box::new(TextEditor::new()),
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
