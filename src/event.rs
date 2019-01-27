use termion::event::Key;

pub enum Event {
    Key(Key),
}