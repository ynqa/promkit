pub mod error;
pub mod formatting;
use formatting::format_screen_diff;
pub mod screen;

pub fn assert_screen_eq(expected: &[String], actual: &[String]) {
    if actual == expected {
        return;
    }

    panic!("{}", format_screen_diff(expected, actual));
}
