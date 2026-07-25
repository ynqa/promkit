use termharness::{error::Result, scenario};

#[test]
fn mid_buffer_insert_wrap() -> Result<()> {
    run(include_str!("scenarios/mid_buffer_insert_wrap.th"))
}

#[test]
fn prompt_initial_render_at_mid_screen() -> Result<()> {
    run(include_str!(
        "scenarios/prompt_initial_render_at_mid_screen.th"
    ))
}

#[test]
fn resize_roundtrip_wrap_reflow() -> Result<()> {
    run(include_str!("scenarios/resize_roundtrip_wrap_reflow.th"))
}

#[test]
fn tiny_viewport_overflow_wrap_scroll() -> Result<()> {
    run(include_str!(
        "scenarios/tiny_viewport_overflow_wrap_scroll.th"
    ))
}

fn run(document: &str) -> Result<()> {
    scenario::run_document(document)?;
    Ok(())
}
