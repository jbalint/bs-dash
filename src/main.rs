use std::io;
use std::iter;
use std::thread;
use std::time::Duration;
use std::borrow::Borrow;

use termion::clear;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::canvas::Line;
use tui::widgets::Paragraph;
use tui::widgets::Text;
use tui::widgets::Widget;
use std::fs::read_dir;
use std::collections::HashMap;
use std::iter::Map;
use std::iter::Take;
use std::fs::ReadDir;
use std::fs::DirEntry;
use std::io::Error;
use std::ffi::OsString;

fn main() -> Result<(), io::Error> {
    //let test = HashMap::new();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size()?;
    println!("{}", clear::All);
    terminal.draw(|mut f| {
        let mut block = Block::default()
            .title("Block")
            .borders(Borders::ALL);
        block.render(&mut f, size);

        //let files: Map<Map<Take<ReadDir>, fn(Result<DirEntry, Error>) -> OsString>, fn(OsString) -> Text> =
        let files =
            read_dir("/home/jbalint/dl")
                .unwrap()
                .take(10)
                .map(|entry| entry.unwrap().file_name())
                .map(|name| String::from(name.to_string_lossy()))
                .map(|mut str| { str.push('\n'); str })
                .map(|str| Text::raw(str))
                .collect::<Vec<Text>>();
        Paragraph::new(files.iter())
            .block(block)
            .render(&mut f, size);
//        Paragraph::new(iter::once(&Text::raw(String::from("asdf"))))
//            .block(block)
//            .render(&mut f, size);
    })?;
    //loop{}
    thread::sleep(Duration::from_millis(5000));
    Ok(())
}
