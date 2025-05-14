use ratatui::style::Color;

#[derive(Debug, Clone, Copy, Default)]
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
    pub background: Color,
    pub scrollbar: Color,
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
            background: Color::Rgb(20, 20, 30),
            scrollbar: Color::Rgb(100, 100, 140),
        }
    }

    fn solarized_theme() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),   // Blue
            secondary: Color::Rgb(42, 161, 152), // Cyan
            text: Color::Rgb(238, 232, 213),     // Base1
            border: Color::Rgb(88, 110, 117),    // Base01
            background: Color::Rgb(0, 43, 54),    // Base03
            scrollbar: Color::Rgb(101, 123, 131),// Base00
        }
    }

    fn monokai_theme() -> Self {
        Self {
            primary: Color::Rgb(249, 38, 114),   // Pink
            secondary: Color::Rgb(102, 217, 239), // Cyan
            text: Color::Rgb(248, 248, 242),     // White
            border: Color::Rgb(117, 113, 94),    // Gray
            background: Color::Rgb(39, 40, 34),  // Dark gray
            scrollbar: Color::Rgb(105, 105, 105),
        }
    }

    fn dracula_theme() -> Self {
        Self {
            primary: Color::Rgb(189, 147, 249),  // Purple
            secondary: Color::Rgb(139, 233, 253),// Cyan
            text: Color::Rgb(248, 248, 242),     // White
            border: Color::Rgb(98, 114, 164),    // Blue-gray
            background: Color::Rgb(40, 42, 54),  // Dark purple
            scrollbar: Color::Rgb(68, 71, 90),
        }
    }

    fn nord_theme() -> Self {
        Self {
            primary: Color::Rgb(129, 161, 193), // Frost1
            secondary: Color::Rgb(136, 192, 208),// Frost2
            text: Color::Rgb(236, 239, 244),     // Snow1
            border: Color::Rgb(76, 86, 106),     // PolarNight2
            background: Color::Rgb(46, 52, 64),  // PolarNight0
            scrollbar: Color::Rgb(67, 76, 94),
        }
    }
    
}