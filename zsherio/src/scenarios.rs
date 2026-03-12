pub mod mid_buffer_insert_wrap {
    use std::time::Duration;

    use crate::{
        Scenario,
        opts::{move_cursor_left, send_bytes},
    };

    pub const TERMINAL_ROWS: u16 = 10;
    pub const TERMINAL_COLS: u16 = 40;
    pub const INPUT_TEXT: &str = "ynqa is a software engineer who writes terminal tools every day";
    pub const INSERTED_TEXT: &str = " and open source maintainer";
    pub const TIMES_TO_MOVE_CURSOR_LEFT: usize = 36;

    pub fn scenario() -> Scenario {
        Scenario::new("mid_buffer_insert_wrap")
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

pub mod prompt_initial_render_at_mid_screen {
    use std::time::Duration;

    use crate::Scenario;

    pub const TERMINAL_ROWS: u16 = 10;
    pub const TERMINAL_COLS: u16 = 40;
    pub const START_CURSOR_ROW: u16 = TERMINAL_ROWS / 2;
    pub const START_CURSOR_COL: u16 = 0;

    pub fn scenario() -> Scenario {
        Scenario::new("prompt_initial_render_at_mid_screen").step(
            "spawn",
            Duration::from_millis(300),
            |_session| Ok(()),
        )
    }
}

pub mod resize_roundtrip_wrap_reflow {
    use std::time::Duration;

    use termharness::terminal::TerminalSize;

    use crate::{
        Scenario,
        opts::{move_cursor_left, send_bytes},
    };

    pub const TERMINAL_ROWS: u16 = 10;
    pub const TERMINAL_COLS: u16 = 40;
    pub const RESIZED_TERMINAL_COLS: u16 = 20;
    pub const TIMES_TO_MOVE_CURSOR_LEFT: usize = 30;

    pub fn scenario() -> Scenario {
        let mut scenario = Scenario::new("resize_roundtrip_wrap_reflow")
            .step("spawn", Duration::from_millis(300), |_session| Ok(()))
            .step("run echo", Duration::from_millis(100), |session| {
                send_bytes(session, b"\"ynqa is a software engineer\"\r")
            })
            .step("type text", Duration::from_millis(100), |session| {
                send_bytes(session, b"this is terminal test suite!")
            });

        // Move the cursor far enough left so resizes do not reflow the active
        // input across the visible boundary.
        scenario = scenario.step("move cursor left", Duration::from_millis(100), |session| {
            move_cursor_left(session, TIMES_TO_MOVE_CURSOR_LEFT)
        });
        for cols in (RESIZED_TERMINAL_COLS..TERMINAL_COLS).rev() {
            scenario = scenario.step(
                format!("resize -> {cols} cols"),
                Duration::from_millis(100),
                move |session| session.resize(TerminalSize::new(TERMINAL_ROWS, cols)),
            );
        }
        for cols in (RESIZED_TERMINAL_COLS + 1)..=TERMINAL_COLS {
            scenario = scenario.step(
                format!("resize -> {cols} cols"),
                Duration::from_millis(100),
                move |session| session.resize(TerminalSize::new(TERMINAL_ROWS, cols)),
            );
        }

        scenario
    }
}

pub mod tiny_viewport_overflow_wrap_scroll {
    use std::time::Duration;

    use crate::{Scenario, opts::send_bytes};

    pub const TERMINAL_ROWS: u16 = 4;
    pub const TERMINAL_COLS: u16 = 12;
    pub const INPUT_TEXT: &str =
        "this input should overflow a tiny terminal viewport and keep wrapping";

    pub fn scenario() -> Scenario {
        Scenario::new("tiny_viewport_overflow_wrap_scroll")
            .step("spawn", Duration::from_millis(300), |_session| Ok(()))
            .step("type long text", Duration::from_millis(100), |session| {
                send_bytes(session, INPUT_TEXT.as_bytes())
            })
    }
}
