mod app;
mod config;
mod swap_info;
mod theme;

use color_eyre::Result;

const HELP: &str = r#"swaptop - Real-time swap usage monitor TUI

USAGE:
    swaptop [FLAGS]

FLAGS:
    -h, --help       Prints this help information
    -v, --version    Prints version information

KEYBOARD SHORTCUTS:
    General:
        q / Esc          Quit
        ?                Toggle help popup

    Navigation:
        ↑/↓ or u/d       Move selection
        PgUp/PgDn         Page up/down
        Home/End          First/last process
        Enter / click     Open/close info panel

    Actions:
        a                Toggle aggregate mode
        t                Cycle theme
        h                Show/hide swap devices
        k / m / g        Unit: KB / MB / GB
        ← / →            Adjust refresh timeout

    Panels:
        Tab / Shift+Tab  Cycle focused panel
        l / r            Resize focused panel divider

    Mouse:
        Click ⬌          Drag to resize divider
        Scroll            Scroll active panel
        Click process     Select + open info panel
"#;

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print!("{}", HELP);
        return Ok(());
    }
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("swaptop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = app::App::new().run(terminal);
    ratatui::restore();
    result
}
