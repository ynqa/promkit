use promkit::{error::Result, preset::Readline};

fn main() -> Result {
    let mut p = Readline::default()
        .title("Feel free to fill in")
        .validator(
            |text| text.len() > 10,
            |text| format!("Length must be over 10 but got {}", text.len()),
        )
        .enable_history()
        .prompt()?;
    loop {
        let line = p.run()?;
        println!("result: {:?}", line);
    }
}
