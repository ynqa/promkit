pub mod middle_insert_wrap {
    use std::time::Duration;

    use crate::{
        Scenario,
        capture::{move_cursor_left, send_bytes},
    };

    pub const TERMINAL_ROWS: u16 = 10;
    pub const TERMINAL_COLS: u16 = 40;
    pub const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
    pub const INSERTED_TEXT: &str = " and open source maintainer";
    pub const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

    pub fn scenario() -> Scenario {
        Scenario::new("middle_insert_wrap")
            .step("spawn", Duration::from_millis(300), |_session| Ok(()))
            .step("type text", Duration::from_millis(100), |session| {
                send_bytes(session, INPUT_TEXT.as_bytes())
            })
            .step("move cursor left", Duration::from_millis(100), |session| {
                move_cursor_left(session, TIMES_TO_MOVE_CURSOR_LEFT)
            })
            .step("insert text", Duration::from_millis(100), |session| {
                send_bytes(session, INSERTED_TEXT.as_bytes())
            })
    }
}
