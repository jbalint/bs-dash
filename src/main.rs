use std::fs::read_dir;
use std::io;
use std::io::stdin;

use itertools::Itertools;
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::Terminal;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::widgets::Text;
use tui::widgets::Widget;

mod jira;

#[macro_use]
extern crate serde_derive;

fn draw_files<T: Backend>(terminal: &mut Terminal<T>, area: Rect) -> io::Result<()> {
    terminal.draw(|mut f| {
        let block = Block::default()
            .style(Style::default().bg(Color::White))
            .title("Block")
            .borders(Borders::ALL);
        // this is kind of stupid, but we create a /sorted/ list of strings
        // before turning them into Text objects
        // (sorted() produces a Vec which must be saved into a variable,
        //  and we need a Vec of Text to create an iterator of Text references
        //  for Paragraph::new())
        let filename_strings =
            read_dir("/home/jbalint/dl")
                .unwrap()
                .take(10)
                .map(|entry| entry.unwrap().file_name())
                .map(|name| String::from(name.to_string_lossy()))
                .map(|mut str| {
                    str.push('\n');
                    str
                })
                .sorted();
        let files = filename_strings
            .iter()
            .map(|str| Text::raw(str))
            .collect::<Vec<Text>>();
        Paragraph::new(files.iter())
            .block(block)
            .wrap(true)
            .render(&mut f, area);
    })
}

fn main() -> Result<(), io::Error> {
    //let test = HashMap::new();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    //let mut selected_file = 0;

    println!("{}", clear::All);

    for c in stdin().keys() {
        let size = terminal.size()?;

        draw_files(&mut terminal, size)?;

        match c.unwrap() {
            Key::Char('q') => break,
            //Key::Up => { selected_file += 1 }
            _ => {}
        }
    }

    Ok(())
}
