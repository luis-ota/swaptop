use ratatui::style::Color;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum ThemeType {
    #[default]
    Default,
    Solarized,
    Monokai,
    Dracula,
    Nord,
}
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub border: Color,
    pub focus_border: Color,
    pub background: Color,
    pub scrollbar: Color,
    pub highlight: Color,
}

impl Theme {
    pub fn from(theme_type: ThemeType) -> Self {
        match theme_type {
            ThemeType::Default => Self::default_theme(),
            ThemeType::Solarized => Self::solarized_theme(),
            ThemeType::Monokai => Self::monokai_theme(),
            ThemeType::Dracula => Self::dracula_theme(),
            ThemeType::Nord => Self::nord_theme(),
        }
    }

    fn default_theme() -> Self {
        Self {
            primary: Color::Rgb(100, 200, 255),
            secondary: Color::Rgb(150, 150, 255),
            text: Color::Rgb(220, 220, 220),
            border: Color::Rgb(80, 80, 120),
            focus_border: Color::Rgb(100, 200, 255),
            background: Color::Rgb(20, 20, 30),
            scrollbar: Color::Rgb(100, 100, 140),
            highlight: Color::Rgb(60, 80, 120),
        }
    }

    fn solarized_theme() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),
            secondary: Color::Rgb(42, 161, 152),
            text: Color::Rgb(238, 232, 213),
            border: Color::Rgb(88, 110, 117),
            focus_border: Color::Rgb(133, 153, 0),
            background: Color::Rgb(0, 43, 54),
            scrollbar: Color::Rgb(101, 123, 131),
            highlight: Color::Rgb(7, 54, 66),
        }
    }

    fn monokai_theme() -> Self {
        Self {
            primary: Color::Rgb(249, 38, 114),
            secondary: Color::Rgb(102, 217, 239),
            text: Color::Rgb(248, 248, 242),
            border: Color::Rgb(117, 113, 94),
            focus_border: Color::Rgb(249, 38, 114),
            background: Color::Rgb(39, 40, 34),
            scrollbar: Color::Rgb(105, 105, 105),
            highlight: Color::Rgb(73, 72, 52),
        }
    }

    fn dracula_theme() -> Self {
        Self {
            primary: Color::Rgb(189, 147, 249),
            secondary: Color::Rgb(139, 233, 253),
            text: Color::Rgb(248, 248, 242),
            border: Color::Rgb(98, 114, 164),
            focus_border: Color::Rgb(255, 121, 198),
            background: Color::Rgb(40, 42, 54),
            scrollbar: Color::Rgb(68, 71, 90),
            highlight: Color::Rgb(68, 71, 90),
        }
    }

    fn nord_theme() -> Self {
        Self {
            primary: Color::Rgb(129, 161, 193),
            secondary: Color::Rgb(136, 192, 208),
            text: Color::Rgb(236, 239, 244),
            border: Color::Rgb(76, 86, 106),
            focus_border: Color::Rgb(136, 192, 208),
            background: Color::Rgb(46, 52, 64),
            scrollbar: Color::Rgb(67, 76, 94),
            highlight: Color::Rgb(59, 66, 82),
        }
    }
}
