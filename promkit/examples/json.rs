use std::{fs::File, io::Read};

use promkit::{json::JsonStream, jsonz, serde_json::Deserializer};

fn main() -> anyhow::Result<()> {
    let mut json = String::new();
    File::open("/Users/eufy/workspace/github.com/ynqa/jnv/test.json")?.read_to_string(&mut json)?;
    let serdejson = Deserializer::from_str(&json)
        .into_iter::<serde_json::Value>()
        .filter_map(serde_json::Result::ok);
    // let stream = JsonStream::new(serdejson, None);
    jsonz::create_rows(serdejson);

    // let mut p = Json::new(stream)
    //     .title("JSON viewer")
    //     .json_lines(5)
    //     .prompt()?;
    // println!("result: {:?}", p.run()?);
    Ok(())
}
