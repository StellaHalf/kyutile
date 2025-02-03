use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::Rect,
    prelude::Color,
    style::Stylize,
    widgets::{Paragraph, Widget},
    Frame,
};

use crate::{
    bar::Input,
    state::{Bar, State},
};

impl State {
    fn render_map(&self, area: Rect, buf: &mut Buffer) {
        match &self.map {
            Some(map) => {
                for i in 0..map.len() {
                    for j in 0..map[0].len() {
                        let cursor = j == self.cursorx && i == self.cursory;
                        let select = self.select.contains(&(j, i));
                        Paragraph::new(if cursor {
                            "◀▶"
                        } else if select {
                            "╱╱"
                        } else {
                            "  "
                        })
                        .bg(Color::from_u32(self.data.colors[&map[j][i]]))
                        .fg(if select {
                            Color::Rgb(0, 0, 255)
                        } else {
                            Color::Rgb(255, 0, 0)
                        })
                        .render(
                            Rect::new(area.x + 2 * i as u16, area.y + j as u16, 2, 1),
                            buf,
                        );
                    }
                }
            }
            None => Paragraph::new(
                "No map loaded. Use :o <path> to load a map, or :c <options> to create one.",
            )
            .render(area, buf),
        }
    }

    pub(crate) fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let map_area = Rect::new(0, 0, area.width, area.height - 2);
        self.render_map(map_area, buf);

        let bar_area = Rect::new(0, area.height.max(1) - 1, area.width, 1);
        let info_area = Rect::new(0, area.height.max(2) - 2, area.width, 1);
        Paragraph::new(self.info_bar()).render(info_area, buf);
        match &self.bar {
            Bar::Input(input) => {
                Paragraph::new(":".to_owned() + input.text().as_ref()).render(bar_area, buf);
                frame.set_cursor_position((input.cursor() as u16 + 1, bar_area.y));
            }
            Bar::Err(err) => {
                Paragraph::new(err.as_str())
                    .fg(Color::Red)
                    .render(bar_area, buf);
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
                KeyCode::Esc => self.bar = Bar::Closed,
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
                self.bar = Bar::Closed;
                self.receive_key_closed(code);
            }
        }
    }

    fn receive_key_closed(&mut self, code: KeyCode) {
        let _ = match &code {
            KeyCode::Char(':') => Ok(self.bar = Bar::Input(Input::empty())),
            KeyCode::Char('h') | KeyCode::Left => self.r#move(&["left"]),
            KeyCode::Char('j') | KeyCode::Down => self.r#move(&["down"]),
            KeyCode::Char('k') | KeyCode::Up => self.r#move(&["up"]),
            KeyCode::Char('l') | KeyCode::Right => self.r#move(&["right"]),
            KeyCode::Char('H') => self.edge(&["left"]),
            KeyCode::Char('J') => self.edge(&["down"]),
            KeyCode::Char('K') => self.edge(&["up"]),
            KeyCode::Char('L') => self.edge(&["right"]),
            KeyCode::Char('d') => self.dot(&[]),
            KeyCode::Char('a') => self.brush(&["add"]),
            KeyCode::Char('s') => self.brush(&["subtract"]),
            KeyCode::Char('i') => self.mode(&["draw"]),
            KeyCode::Esc => {
                let _ = self.mode(&["normal"]);
                self.argument = 0;
                Ok(())
            }
            KeyCode::Char('o') => self.bucket(&[]),
            KeyCode::Char('p') => self.pick(&[]),
            KeyCode::Char(c) => Ok(match c.to_digit(10) {
                Some(i) => self.append_argument(i as u8),
                None => {}
            }),
            _ => Ok(()),
        };
    }
}
