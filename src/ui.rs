use std::{collections::HashMap, io};

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Direction, Layout},
    prelude::Color,
    style::Stylize,
    widgets::{
        canvas::{Canvas, Rectangle},
        Paragraph, Widget,
    },
    Frame,
};

use crate::state::{Bar, State};

const TILE_COLORS: [(i32, Color); 1] = [(0, Color::Rgb(0x99, 0xe5, 0x99))];

fn map_canvas(map: &[Vec<i32>], scale: f64) -> impl Widget + '_ {
    Canvas::default()
        .x_bounds([0., scale])
        .y_bounds([0., scale])
        .paint(move |ctx| {
            let color_map = HashMap::from(TILE_COLORS);
            let x = map.len();
            let y = map.get(0).map(Vec::len).unwrap_or(0);
            let sx = scale / x as f64;
            let sy = scale / y as f64;
            for i in 0..x {
                for j in 0..y {
                    ctx.draw(&Rectangle {
                        color: color_map[&map[i][j]],
                        x: i as f64 * sx,
                        y: j as f64 * sy,
                        height: sx,
                        width: sx,
                    })
                }
            }
        })
}

impl State {
    pub(crate) fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(1)])
            .split(area);

        match *self.map() {
            Some(map) => map_canvas(map, 10.).render(layout[0], buf),
            None => Paragraph::new("no map :(").render(layout[0], buf),
        }

        match &self.bar {
            Bar::Input(input) => {
                Paragraph::new(":".to_owned() + input.text().as_ref()).render(layout[1], buf);
                frame.set_cursor_position((input.cursor() as u16 + 1, layout[1].y));
            }
            Bar::Err(err) => {
                Paragraph::new(err.as_str())
                    .fg(Color::Red)
                    .render(layout[1], buf);
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
