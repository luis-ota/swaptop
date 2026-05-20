mod app;
mod config;
mod swap_info;
mod theme;

use color_eyre::Result;

const HELP: &str = r#"swaptop - real-time swap usage monitor tui

usage:
  swaptop [flags]

flags:
  -h, --help       prints this help information
  -v, --version    prints version information

keyboard shortcuts:
  general:
    q / esc         quit
    ?               toggle help popup

  navigation:
    ↑/↓ or u/d     move selection
    PgUp/PgDn      page up/down
    Home/End        first/last process
    Enter / click   open/close info panel

  actions:
    a               toggle aggregate mode
    t               cycle theme
    h               show/hide swap devices
    k / m / g       unit: KB / MB / GB
    ← / →           adjust refresh timeout

  panels:
    Tab / Shift+Tab cycle focused panel
    l / r           resize focused panel divider

  mouse:
    click ⬌         drag to resize divider
    scroll          scroll active panel
    click process   select + open info panel
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
