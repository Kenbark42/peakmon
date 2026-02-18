use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    Resize,
}

pub fn poll_event(timeout: Duration) -> color_eyre::Result<AppEvent> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(AppEvent::Key(key)),
            Event::Mouse(mouse) => Ok(AppEvent::Mouse(mouse)),
            Event::Resize(_, _) => Ok(AppEvent::Resize),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}
