use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub root: Style,
    pub app_title: Style,
    pub tabs: Style,
    pub tabs_selected: Style,
    pub borders: Style,
    pub key_binding: KeyBinding,
    pub content: Style,
    // pub error: Style,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().bg(DARK_BLUE),
    app_title: Style::new()
        .fg(WHITE)
        .bg(DARK_BLUE)
        .add_modifier(Modifier::BOLD),
    tabs: Style::new().fg(MID_GRAY).bg(DARK_BLUE),
    tabs_selected: Style::new()
        .fg(WHITE)
        .bg(DARK_BLUE)
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::REVERSED),
    borders: Style::new().fg(LIGHT_GRAY).bg(DARK_BLUE),
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(DARK_GRAY),
        description: Style::new().fg(DARK_GRAY).bg(BLACK),
    },
    content: Style::new().fg(LIGHT_GRAY).bg(DARK_BLUE),
    //error: Style::new().fg(RED).bg(DARK_BLUE),
};

const DARK_BLUE: Color = Color::Rgb(16, 24, 48);
// const LIGHT_BLUE: Color = Color::Rgb(64, 96, 192);
// const LIGHT_YELLOW: Color = Color::Rgb(192, 192, 96);
// const LIGHT_GREEN: Color = Color::Rgb(64, 192, 96);
// const LIGHT_RED: Color = Color::Rgb(192, 96, 96);
// const RED: Color = Color::Rgb(215, 0, 0);
const BLACK: Color = Color::Rgb(8, 8, 8); // not really black, often #080808
const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
const MID_GRAY: Color = Color::Rgb(128, 128, 128);
const LIGHT_GRAY: Color = Color::Rgb(188, 188, 188);
const WHITE: Color = Color::Rgb(238, 238, 238); // not really white, often #eeeeee
