use std::sync::LazyLock;

pub type Frame = Vec<&'static str>;

pub static DOTS: LazyLock<Frame> =
    LazyLock::new(|| vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

pub const HAMBURGER: LazyLock<Frame> = LazyLock::new(|| vec!["☱", "☲", "☴"]);

pub const ARC: LazyLock<Frame> = LazyLock::new(|| vec!["◜", "◠", "◝", "◞", "◡", "◟"]);

pub const CIRCLE: LazyLock<Frame> = LazyLock::new(|| vec!["◡", "⊙", "◠"]);

pub const SQUARE_CORNERS: LazyLock<Frame> = LazyLock::new(|| vec!["◰", "◳", "◲", "◱"]);

pub const CIRCLE_QUARTERS: LazyLock<Frame> = LazyLock::new(|| vec!["◴", "◷", "◶", "◵"]);

pub const CIRCLE_HALVES: LazyLock<Frame> = LazyLock::new(|| vec!["◐", "◓", "◑", "◒"]);

pub const TOGGLE: LazyLock<Frame> = LazyLock::new(|| vec!["⊶", "⊷"]);

pub const CLOCK: LazyLock<Frame> = LazyLock::new(|| {
    vec![
        "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚",
    ]
});

pub const EARTH: LazyLock<Frame> = LazyLock::new(|| vec!["🌍", "🌎", "🌏"]);

pub const MOON: LazyLock<Frame> =
    LazyLock::new(|| vec!["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"]);
