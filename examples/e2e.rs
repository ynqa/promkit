use promkit::{
    json::JsonNode,
    preset::{json::Json, listbox::Listbox, readline::Readline},
    Result,
};

fn main() -> Result {
    let mut p = Readline::default().title("Feel free to fill in").prompt()?;
    println!("result: {:?}", p.run()?);
    let mut p = Listbox::new(0..100)
        .title("What number do you like?")
        .listbox_lines(5)
        .prompt()?;
    println!("result: {:?}", p.run()?);
    let mut p = Json::new(JsonNode::try_from(
        r#"{
          "number": 1,
          "map": {
            "string1": "aaa",
            "string2": "bbb"
          },
          "list": [
            "abc",
            "def"
          ],
          "map_in_map": {
            "nested": {
              "leaf": "eof"
            }
          },
          "map_in_list": [
            {
              "map1": 1
            },
            {
              "map2": 2
            }
          ]
        }"#,
    )?)
    .title("JSON viewer")
    .json_lines(10)
    .prompt()?;
    println!("result: {:?}", p.run()?);
    Ok(())
}
