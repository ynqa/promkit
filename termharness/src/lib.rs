pub mod error;
pub mod screen_diff;
use screen_diff::format_screen_diff;
pub mod screen;

pub fn assert_screen_eq(expected: &[String], actual: &[String]) {
    if actual == expected {
        return;
    }

    panic!("{}", format_screen_diff(expected, actual));
}
