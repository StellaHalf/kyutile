use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    prelude::Color,
    style::Stylize,
    widgets::{Paragraph, Widget},
    Frame,
};

use crate::{
    bar::Input,
    state::{Bar, State},
};

const BF: &str = "\u{2588}\u{2588}";
const BS: &str = "\u{259e}\u{259e}";
const BC: &str = "\u{3010}\u{3011}";

impl State {
    fn render_map(&self, area: Rect, buf: &mut Buffer) {
        match &self.map {
            Some(map) => {
                for i in 0..map.len() {
                    for j in 0..map[0].len() {
                        Paragraph::new(if j == self.cursorx && i == self.cursory {
                            BC
                        } else if self.select.contains(&(i, j)) {
                            BS
                        } else {
                            BF
                        })
                        .fg(Color::from_u32(self.data.colors[&map[j][i]]))
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
        let ui_area = Rect::new(0, 0, area.width, area.height - 1);
        let ui_layout =
            Layout::horizontal(vec![Constraint::Fill(1), Constraint::Length(5)]).split(ui_area);
        let tile_layout =
            Layout::vertical(vec![Constraint::Fill(1), Constraint::Fill(1)]).split(ui_layout[1]);

        self.render_map(ui_layout[0], buf);

        let bar_area = Rect::new(0, area.height.max(1) - 1, area.width, 1);
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
        match &code {
            KeyCode::Char(':') => self.bar = Bar::Input(Input::empty()),
            _ => (),
        }
    }
}
