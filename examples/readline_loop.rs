use promkit::{error::Result, preset::readline::Readline};

fn main() -> Result {
    let mut p = Readline::default().title("Feel free to fill in").prompt()?;
    loop {
        match p.run() {
            Ok(ret) => println!("result: {:?}", ret),
            Err(e) => return Err(e),
        }
    }
}
