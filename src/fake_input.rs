use std::{
    fs::File,
    io,
    os::unix::io::{AsRawFd, IntoRawFd},
};

use libc::{ioctl, TIOCSTI};

use crate::Result;

/// Simulates keyboard input by injecting bytes into the stdin stream.
///
/// This function switches the standard input (stdin)
/// to the terminal device (`/dev/tty`)
/// and then simulates keyboard input by injecting
/// the provided string `s` directly into the stdin buffer.
/// This is particularly useful for testing or for applications
/// that need to automate input.
///
/// # Errors
///
/// This function will return an error if it fails to
/// open `/dev/tty` or if any system call involved fails.
///
/// # Safety
///
/// This function uses unsafe operations,
/// such as `libc::dup2` to duplicate file descriptors
/// and `ioctl` with `TIOCSTI` to simulate terminal input,
/// which can have security and stability implications.
pub fn run(s: &str) -> Result<()> {
    let stdin_fd = io::stdin().as_raw_fd();
    let tty = File::open("/dev/tty")?;

    let tty_fd = tty.into_raw_fd();
    unsafe {
        libc::dup2(tty_fd, stdin_fd);
    }

    for byte in s.as_bytes() {
        unsafe {
            ioctl(stdin_fd, TIOCSTI, byte);
        }
    }
    Ok(())
}
