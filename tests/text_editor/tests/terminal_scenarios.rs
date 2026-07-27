use termharness::{error::Result, scenario};

#[test]
fn continuation_prefix_and_cursor_redraw() -> Result<()> {
    run(include_str!(
        "scenarios/continuation_prefix_and_cursor_redraw.th"
    ))
}

#[test]
fn multiline_wrap_cursor_redraw() -> Result<()> {
    run(include_str!("scenarios/multiline_wrap_cursor_redraw.th"))
}

#[test]
fn multiline_viewport_scrolls_with_cursor() -> Result<()> {
    run(include_str!(
        "scenarios/multiline_viewport_scrolls_with_cursor.th"
    ))
}

fn run(document: &str) -> Result<()> {
    scenario::run_document(document)?;
    Ok(())
}
