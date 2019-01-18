use std::io;

use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Widget;

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    terminal.draw(|mut f| {
        Block::default()
            .title("Block")
            .borders(Borders::ALL)
            .render(&mut f, size);
    })
}
