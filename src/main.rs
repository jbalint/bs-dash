#[macro_use]
extern crate serde_derive;

use std::fs::read_dir;
use std::io;
use std::io::stdin;

use itertools::Itertools;
use termion::clear;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::Frame;
use tui::layout::Constraint;
use tui::layout::Layout;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::Terminal;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::widgets::Row;
use tui::widgets::Table;
use tui::widgets::Text;
use tui::widgets::Widget;

use crate::jira::Issue;

mod event;
mod jira;

#[derive(Debug)]
enum CompoundError {
    Reqwest(reqwest::Error),
    Io(io::Error),
}

impl From<reqwest::Error> for CompoundError {
    fn from(e: reqwest::Error) -> Self {
        CompoundError::Reqwest(e)
    }
}

impl From<io::Error> for CompoundError {
    fn from(e: io::Error) -> Self {
        CompoundError::Io(e)
    }
}

fn draw_files<B: Backend>(f: &mut Frame<B>, area: Rect) -> io::Result<()> {
    let block = Block::default()
        .style(Style::default().bg(Color::White))
        .title(" Block ")
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
        .render(f, area);
    Ok(())
}

fn draw_overdue_issues<B: Backend>(f: &mut Frame<B>, area: Rect, issues: Vec<Issue>) -> io::Result<()> {
    let selected_style = Style::default().fg(Color::Yellow).modifier(Modifier::Bold);
    let normal_style = Style::default().fg(Color::White);
    let header = ["Key", "Due", "Created", "Summary"];
    let rows =
        issues.iter()
            .map(|issue| {
                vec![issue.key.clone(),
                     format!("{}", issue.fields.duedate.unwrap()),
                     format!("{}", issue.fields.created.naive_local().date()),
                     issue.fields.summary.clone()]
            })
            .enumerate()
            .map(|(i, item)| {
                if i == 3 {
                    Row::StyledData(item.into_iter(), selected_style)
                } else {
                    Row::StyledData(item.into_iter(), normal_style)
                }
            });
    Table::new(header.into_iter(), rows)
        .block(Block::default().borders(Borders::ALL).title(" Overdue Issues "))
        .widths(&[11, 11, 11, 90])
        .render(f, area);
    Ok(())
}

fn main() -> Result<(), CompoundError> {
    let stdout = io::stdout().into_raw_mode()?;
//    let stdout = MouseTerminal::from(stdout);
//    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    //let mut selected_file = 0;

    println!("{}", clear::All);

    for c in stdin().keys() {
        let size = terminal.size()?;
        // TODO : resizing

        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(size);

        let issues = jira::get_overdue_issues()?;

        // TODO : propagating error from the closure
        terminal.draw(|mut f| {
            // this all has to be done in one call to terminal::draw()
            draw_files(&mut f, chunks[0]).unwrap();
            draw_overdue_issues(&mut f, chunks[1], issues).unwrap();
        })?;

        match c.unwrap() {
            Key::Char('q') => break,
            //Key::Up => { selected_file += 1 }
            _ => {}
        }
    }

    Ok(())
}
