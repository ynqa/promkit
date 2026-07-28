//! Terminal lifecycle management for interactive applications.

use std::io;

use bitflags::bitflags;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, is_raw_mode_enabled, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

bitflags! {
    /// Terminal modes managed by a [`TerminalSession`].
    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    pub struct TerminalModes: u8 {
        /// Enables raw mode.
        const RAW_MODE = 1 << 0;
        /// Enters the alternate screen.
        const ALTERNATE_SCREEN = 1 << 1;
        /// Hides the cursor.
        const HIDDEN_CURSOR = 1 << 2;
        /// Enables mouse capture.
        const MOUSE_CAPTURE = 1 << 3;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operation {
    EnableRawMode,
    EnterAlternateScreen,
    HideCursor,
    EnableMouseCapture,
    DisableMouseCapture,
    ShowCursor,
    LeaveAlternateScreen,
    DisableRawMode,
}

type Backend = fn(Operation) -> io::Result<()>;

fn crossterm_backend(operation: Operation) -> io::Result<()> {
    match operation {
        Operation::EnableRawMode => enable_raw_mode(),
        Operation::EnterAlternateScreen => execute!(io::stdout(), EnterAlternateScreen),
        Operation::HideCursor => execute!(io::stdout(), cursor::Hide),
        Operation::EnableMouseCapture => execute!(io::stdout(), EnableMouseCapture),
        Operation::DisableMouseCapture => execute!(io::stdout(), DisableMouseCapture),
        Operation::ShowCursor => execute!(io::stdout(), cursor::Show),
        Operation::LeaveAlternateScreen => execute!(io::stdout(), LeaveAlternateScreen),
        Operation::DisableRawMode => disable_raw_mode(),
    }
}

/// Owns configured terminal modes for the duration of an interactive session.
///
/// [`TerminalSession::try_new`] applies the requested [`TerminalModes`].
/// [`TerminalSession::restore`] reverses them, and restoration is also attempted
/// automatically when the session is dropped, including during panic
/// unwinding.
///
/// If setup fails partway through, all modes whose setup was attempted are
/// restored before the setup error is returned. Raw mode is not disabled if it
/// was already enabled before the session started.
///
/// # Example
///
/// ```no_run
/// use promkit::{TerminalModes, TerminalSession};
///
/// # fn run() -> std::io::Result<()> {
/// let modes = TerminalModes::RAW_MODE
///     | TerminalModes::HIDDEN_CURSOR
///     | TerminalModes::MOUSE_CAPTURE;
/// let _session = TerminalSession::try_new(modes)?;
/// // Run the application's event loop.
/// # Ok(())
/// # }
/// ```
#[must_use = "dropping the session immediately restores the terminal"]
pub struct TerminalSession {
    backend: Backend,
    pending_restore: TerminalModes,
}

impl TerminalSession {
    /// Applies `modes` and starts a terminal session.
    ///
    /// Setup follows a fixed order: raw mode, alternate screen, hidden cursor,
    /// then mouse capture. If a step returns an error or unwinds, [`Drop`]
    /// attempts to reverse every step that had started.
    pub fn try_new(modes: TerminalModes) -> io::Result<Self> {
        Self::try_new_with(crossterm_backend, modes, is_raw_mode_enabled()?)
    }

    fn try_new_with(
        backend: Backend,
        modes: TerminalModes,
        raw_mode_was_enabled: bool,
    ) -> io::Result<Self> {
        let mut session = Self {
            backend,
            pending_restore: TerminalModes::empty(),
        };

        for (mode, operation) in [
            (TerminalModes::RAW_MODE, Operation::EnableRawMode),
            (
                TerminalModes::ALTERNATE_SCREEN,
                Operation::EnterAlternateScreen,
            ),
            (TerminalModes::HIDDEN_CURSOR, Operation::HideCursor),
            (TerminalModes::MOUSE_CAPTURE, Operation::EnableMouseCapture),
        ] {
            if !modes.contains(mode) || (mode == TerminalModes::RAW_MODE && raw_mode_was_enabled) {
                continue;
            }

            // Record the inverse before setup so Drop also covers a panic or a
            // partially applied terminal command.
            session.pending_restore.insert(mode);
            (session.backend)(operation)?;
        }

        Ok(session)
    }

    /// Restores the terminal modes owned by this session.
    ///
    /// Every pending restoration is attempted even if an earlier one fails.
    /// The first error is returned. Successfully restored modes are removed, so
    /// a later call or [`Drop`] retries only failed restorations.
    pub fn restore(&mut self) -> io::Result<()> {
        let mut first_error = None;

        for (mode, operation) in [
            (TerminalModes::MOUSE_CAPTURE, Operation::DisableMouseCapture),
            (TerminalModes::HIDDEN_CURSOR, Operation::ShowCursor),
            (
                TerminalModes::ALTERNATE_SCREEN,
                Operation::LeaveAlternateScreen,
            ),
            (TerminalModes::RAW_MODE, Operation::DisableRawMode),
        ] {
            if !self.pending_restore.contains(mode) {
                continue;
            }

            match (self.backend)(operation) {
                Ok(()) => self.pending_restore.remove(mode),
                Err(error) if first_error.is_none() => first_error = Some(error),
                Err(_) => {}
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore().ok();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::RefCell,
        collections::VecDeque,
        io::{self, ErrorKind},
    };

    use super::{Operation, TerminalModes, TerminalSession};

    #[derive(Default)]
    struct MockState {
        operations: Vec<Operation>,
        failures: VecDeque<Operation>,
    }

    thread_local! {
        static MOCK_STATE: RefCell<MockState> = RefCell::new(MockState::default());
    }

    fn mock_backend(operation: Operation) -> io::Result<()> {
        MOCK_STATE.with_borrow_mut(|state| {
            state.operations.push(operation);
            if state.failures.front() == Some(&operation) {
                state.failures.pop_front();
                Err(io::Error::other("terminal operation failed"))
            } else {
                Ok(())
            }
        })
    }

    fn panicking_backend(operation: Operation) -> io::Result<()> {
        if operation == Operation::HideCursor {
            MOCK_STATE.with_borrow_mut(|state| state.operations.push(operation));
            panic!("terminal operation panicked");
        }
        mock_backend(operation)
    }

    fn reset(failures: impl IntoIterator<Item = Operation>) {
        MOCK_STATE.with_borrow_mut(|state| {
            *state = MockState {
                failures: failures.into_iter().collect(),
                ..MockState::default()
            };
        });
    }

    fn operations() -> Vec<Operation> {
        MOCK_STATE.with_borrow(|state| state.operations.clone())
    }

    fn try_new(modes: TerminalModes) -> io::Result<TerminalSession> {
        TerminalSession::try_new_with(mock_backend, modes, false)
    }

    #[test]
    fn restores_fullscreen_modes_in_reverse_order() {
        reset([]);

        let modes = TerminalModes::RAW_MODE
            | TerminalModes::ALTERNATE_SCREEN
            | TerminalModes::HIDDEN_CURSOR
            | TerminalModes::MOUSE_CAPTURE;
        let mut session = try_new(modes).unwrap();
        session.restore().unwrap();
        session.restore().unwrap();

        assert_eq!(
            operations(),
            [
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::EnableMouseCapture,
                Operation::DisableMouseCapture,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn restores_terminal_when_dropped() {
        reset([]);

        {
            let modes = TerminalModes::RAW_MODE
                | TerminalModes::HIDDEN_CURSOR
                | TerminalModes::MOUSE_CAPTURE;
            let _session = try_new(modes).unwrap();
        }

        assert_eq!(
            operations(),
            [
                Operation::EnableRawMode,
                Operation::HideCursor,
                Operation::EnableMouseCapture,
                Operation::DisableMouseCapture,
                Operation::ShowCursor,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn rolls_back_attempted_setup_when_setup_fails() {
        reset([Operation::HideCursor]);

        let modes = TerminalModes::RAW_MODE
            | TerminalModes::ALTERNATE_SCREEN
            | TerminalModes::HIDDEN_CURSOR
            | TerminalModes::MOUSE_CAPTURE;
        let error = try_new(modes).err().expect("setup must fail");

        assert_eq!(error.kind(), ErrorKind::Other);
        assert_eq!(
            operations(),
            [
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn restores_attempted_setup_when_setup_panics() {
        reset([]);

        let modes = TerminalModes::RAW_MODE
            | TerminalModes::ALTERNATE_SCREEN
            | TerminalModes::HIDDEN_CURSOR
            | TerminalModes::MOUSE_CAPTURE;
        let panic = std::panic::catch_unwind(|| {
            TerminalSession::try_new_with(panicking_backend, modes, false).ok();
        });

        assert!(panic.is_err());
        assert_eq!(
            operations(),
            [
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
            ]
        );
    }

    #[test]
    fn continues_restoring_after_an_error_and_retries_failed_steps() {
        reset([Operation::DisableMouseCapture]);

        let modes = TerminalModes::RAW_MODE
            | TerminalModes::ALTERNATE_SCREEN
            | TerminalModes::HIDDEN_CURSOR
            | TerminalModes::MOUSE_CAPTURE;
        let mut session = try_new(modes).unwrap();
        assert_eq!(session.restore().unwrap_err().kind(), ErrorKind::Other);
        session.restore().unwrap();

        assert_eq!(
            operations(),
            [
                Operation::EnableRawMode,
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::EnableMouseCapture,
                Operation::DisableMouseCapture,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
                Operation::DisableRawMode,
                Operation::DisableMouseCapture,
            ]
        );
    }

    #[test]
    fn applies_only_requested_modes() {
        reset([]);

        let modes = TerminalModes::ALTERNATE_SCREEN | TerminalModes::MOUSE_CAPTURE;
        let mut session = try_new(modes).unwrap();
        session.restore().unwrap();

        assert_eq!(
            operations(),
            [
                Operation::EnterAlternateScreen,
                Operation::EnableMouseCapture,
                Operation::DisableMouseCapture,
                Operation::LeaveAlternateScreen,
            ]
        );
    }

    #[test]
    fn preserves_raw_mode_that_was_already_enabled() {
        reset([]);

        let modes = TerminalModes::RAW_MODE
            | TerminalModes::ALTERNATE_SCREEN
            | TerminalModes::HIDDEN_CURSOR
            | TerminalModes::MOUSE_CAPTURE;
        let mut session = TerminalSession::try_new_with(mock_backend, modes, true).unwrap();
        session.restore().unwrap();

        assert_eq!(
            operations(),
            [
                Operation::EnterAlternateScreen,
                Operation::HideCursor,
                Operation::EnableMouseCapture,
                Operation::DisableMouseCapture,
                Operation::ShowCursor,
                Operation::LeaveAlternateScreen,
            ]
        );
    }
}
