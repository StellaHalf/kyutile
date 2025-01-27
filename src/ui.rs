use std::{any::Any, cmp::min, collections::HashMap, io};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    prelude::Color,
    style::Stylize,
    widgets::{Paragraph, Widget},
    Frame,
};

use crate::state::{Bar, State};

const BLOCK: &str = "\u{2588}\u{2588}";

fn render_canvas(map: &[Vec<i32>], tile_colors: &HashMap<i32, u32>, area: Rect, buf: &mut Buffer) {
    for i in 0..map.len() {
        for j in 0..map[0].len() {
            Paragraph::new(BLOCK)
                .fg(Color::from_u32(tile_colors[&map[j][i]]))
                .render(
                    Rect::new(area.x + 2 * i as u16, area.y + j as u16, 2, 1),
                    buf,
                );
        }
    }
}

impl State {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let bar_height = if self.bar == Bar::Closed { 0 } else { 1 };
        let sidelen = area.width.min(area.height - bar_height).max(2) - 2;
        let map_rect = Rect::new(1, 1, sidelen, sidelen);

        match *self.map() {
            Some(map) => render_canvas(map, &(*self.tile_data()).colors, map_rect, buf),
            None => Paragraph::new("no map :(").render(map_rect, buf),
        }

        let bar_rect = Rect::new(0, area.height.max(1) - 1, area.width, 1);
        match &self.bar {
            Bar::Input(input) => {
                Paragraph::new(":".to_owned() + input.text().as_ref()).render(bar_rect, buf);
                frame.set_cursor_position((input.cursor() as u16 + 1, bar_rect.y));
            }
            Bar::Err(err) => {
                Paragraph::new(err.as_str())
                    .fg(Color::Red)
                    .render(bar_rect, buf);
            }
            _ => (),
        }
    }

    pub(crate) fn handle_events(&mut self) -> Result<(), io::Error> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.receive_key(key_event.code)
            }
            _ => (),
        };
        Ok(())
    }

    fn receive_key(&mut self, code: KeyCode) {
        match &mut self.bar {
            Bar::Input(input) => match &code {
                KeyCode::Right => input.move_right(),
                KeyCode::Left => input.move_left(),
                KeyCode::Char(c) => input.write(*c),
                KeyCode::Backspace => input.backspace(),
                KeyCode::Delete => input.delete(),
                KeyCode::Esc => self.clear_bar(),
                KeyCode::Enter => {
                    let text = input.text();
                    if let Err(err) = self.parse_command(&text) {
                        self.bar = Bar::Err(err)
                    } else {
                        self.bar = Bar::Closed
                    }
                }
                _ => (),
            },
            Bar::Closed => self.receive_key_closed(code),
            Bar::Err(_) => {
                self.clear_bar();
                self.receive_key_closed(code);
            }
        }
    }

    fn receive_key_closed(&mut self, code: KeyCode) {
        match &code {
            KeyCode::Char(':') => self.begin_input(),
            _ => (),
        }
    }
}
