use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    Resize,
}

pub fn poll_event(timeout: Duration) -> color_eyre::Result<AppEvent> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => Ok(AppEvent::Key(key)),
            Event::Resize(_, _) => Ok(AppEvent::Resize),
            _ => Ok(AppEvent::Tick),
        }
    } else {
        Ok(AppEvent::Tick)
    }
}
