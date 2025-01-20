use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    widgets::{Block, Paragraph, Widget},
    Frame,
};

use crate::commands::State;

impl Widget for &State {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Paragraph::new("amogus").centered().render(area, buf);
    }
}

impl State {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub(crate) fn handle_events(&mut self) -> Result<(), io::Error> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.on_key_pressed(key_event.code)
            }
            _ => (),
        };
        Ok(())
    }

    fn on_key_pressed(&mut self, code: KeyCode) {}
}
