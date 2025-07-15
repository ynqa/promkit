pub type Frame = &'static [&'static str];

pub const DOTS: Frame = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub const HAMBURGER: Frame = &["☱", "☲", "☴"];

pub const ARC: Frame = &["◜", "◠", "◝", "◞", "◡", "◟"];

pub const CIRCLE: Frame = &["◡", "⊙", "◠"];

pub const SQUARE_CORNERS: Frame = &["◰", "◳", "◲", "◱"];

pub const CIRCLE_QUARTERS: Frame = &["◴", "◷", "◶", "◵"];

pub const CIRCLE_HALVES: Frame = &["◐", "◓", "◑", "◒"];

pub const TOGGLE: Frame = &["⊶", "⊷"];

pub const CLOCK: Frame = &[
    "🕛", "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚",
];

pub const EARTH: Frame = &["🌍", "🌎", "🌏"];

pub const MOON: Frame = &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"];
