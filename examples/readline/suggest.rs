use promkit::{build::Builder, readline, register::Register, suggest::Suggest, Result};

fn main() -> Result<()> {
    let mut s = Suggest::default();
    s.register_all(vec!["/help", "/run", "/exit"]);
    let mut p = readline::Builder::default().suggest(s).build()?;
    loop {
        let line = p.run()?;
        if line == "/exit" {
            println!("/exit command was executed");
            return Ok(());
        }
    }
}
