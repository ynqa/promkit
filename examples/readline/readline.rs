use anyhow::Result;

use promkit::{
    editor::{TextBuilder, TextEditorBuilder},
    Prompt,
};

fn main() -> Result<()> {
    let mut p = Prompt::new(vec![
        TextBuilder::new("hello world").build()?,
        TextEditorBuilder::new().build()?,
        TextEditorBuilder::new().build()?,
    ]);
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
