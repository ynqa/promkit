use termharness::{error::Result, scenario};

#[test]
fn height_policies_allocate_terminal_rows() -> Result<()> {
    scenario::run_document(include_str!(
        "scenarios/height_policies_allocate_terminal_rows.th"
    ))?;
    Ok(())
}
