use termharness::{error::Result, scenario};

#[test]
fn ordered_content_allocates_height_in_order() -> Result<()> {
    scenario::run_document(include_str!("scenarios/ordered_content.th"))?;
    Ok(())
}

#[test]
fn fair_fill_shares_and_preserves_height() -> Result<()> {
    scenario::run_document(include_str!("scenarios/fair_fill.th"))?;
    Ok(())
}

#[test]
fn fair_content_caps_each_share_and_packs_short_content() -> Result<()> {
    scenario::run_document(include_str!("scenarios/fair_content.th"))?;
    Ok(())
}
