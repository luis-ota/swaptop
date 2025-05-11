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
    pub accent: Color,
    pub warning: Color,
    pub text: Color,
    pub border: Color,
    pub background: Color,
    pub highlight: Color,
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
            accent: Color::Rgb(255, 150, 50),
            warning: Color::Rgb(255, 80, 80),
            text: Color::Rgb(220, 220, 220),
            border: Color::Rgb(80, 80, 120),
            background: Color::Rgb(20, 20, 30),
            highlight: Color::Rgb(200, 200, 255),
            scrollbar: Color::Rgb(100, 100, 140),
        }
    }

    fn solarized_theme() -> Self {
        Self {
            primary: Color::Rgb(38, 139, 210),   // Blue
            secondary: Color::Rgb(42, 161, 152), // Cyan
            accent: Color::Rgb(203, 75, 22),     // Orange
            warning: Color::Rgb(211, 1, 2),      // Red
            text: Color::Rgb(238, 232, 213),     // Base1
            border: Color::Rgb(88, 110, 117),    // Base01
            background: Color::Rgb(0, 43, 54),    // Base03
            highlight: Color::Rgb(147, 161, 161),// Base0
            scrollbar: Color::Rgb(101, 123, 131),// Base00
        }
    }

    fn monokai_theme() -> Self {
        Self {
            primary: Color::Rgb(249, 38, 114),   // Pink
            secondary: Color::Rgb(102, 217, 239), // Cyan
            accent: Color::Rgb(253, 151, 31),    // Orange
            warning: Color::Rgb(255, 0, 0),      // Red
            text: Color::Rgb(248, 248, 242),     // White
            border: Color::Rgb(117, 113, 94),    // Gray
            background: Color::Rgb(39, 40, 34),  // Dark gray
            highlight: Color::Rgb(174, 129, 255),// Purple
            scrollbar: Color::Rgb(105, 105, 105),
        }
    }

    fn dracula_theme() -> Self {
        Self {
            primary: Color::Rgb(189, 147, 249),  // Purple
            secondary: Color::Rgb(139, 233, 253),// Cyan
            accent: Color::Rgb(255, 184, 108),   // Orange
            warning: Color::Rgb(255, 85, 85),    // Red
            text: Color::Rgb(248, 248, 242),     // White
            border: Color::Rgb(98, 114, 164),    // Blue-gray
            background: Color::Rgb(40, 42, 54),  // Dark purple
            highlight: Color::Rgb(80, 250, 123), // Green
            scrollbar: Color::Rgb(68, 71, 90),
        }
    }

    fn nord_theme() -> Self {
        Self {
            primary: Color::Rgb(129, 161, 193), // Frost1
            secondary: Color::Rgb(136, 192, 208),// Frost2
            accent: Color::Rgb(191, 97, 106),    // Aurora0
            warning: Color::Rgb(208, 135, 112),  // Aurora1
            text: Color::Rgb(236, 239, 244),     // Snow1
            border: Color::Rgb(76, 86, 106),     // PolarNight2
            background: Color::Rgb(46, 52, 64),  // PolarNight0
            highlight: Color::Rgb(143, 188, 187),// Frost3
            scrollbar: Color::Rgb(67, 76, 94),
        }
    }
    
}