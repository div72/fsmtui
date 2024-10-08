#![feature(let_chains)]

use core::f64;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Circle, Context, Line},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};

mod vector2d;
use vector2d::Vector2D;

fn main() -> std::io::Result<()> {
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

struct FSMState {
    x: f64,
    y: f64,
    name: String,
    next_states: Vec<Weak<RefCell<FSMState>>>,
}

impl FSMState {
    fn circle_radius(&self) -> f64 {
        ((self.name.len() * 2) as f64 + 5.0).max(10.0)
    }

    fn to_circle(&self, color: Color) -> Circle {
        Circle {
            x: self.x,
            y: self.y,
            radius: self.circle_radius(),
            color,
        }
    }

    fn draw(&self, circle_color: Color, canvas_ctx: &mut Context<'_>) {
        for next_state in &self.next_states {
            if let Some(state) = next_state.upgrade() {
                let state = state.borrow();

                let mut v1 = Vector2D {
                    x: self.x,
                    y: self.y,
                };

                let mut v2 = Vector2D {
                    x: state.x,
                    y: state.y,
                };

                let v_arrow = (v2 - v1).normalized() * 1.5;
                v1 = v1 + v_arrow * self.circle_radius();
                v2 = v2 - v_arrow * state.circle_radius();

                let (x1, y1) = v1.into();
                let (x2, y2) = v2.into();

                canvas_ctx.draw(&Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color: Color::White,
                });

                // The arrowhead part

                let (x3, y3) =
                    ((v1 - v2).normalized().rotate(f64::consts::FRAC_PI_4) * 10.0 + v2).into();
                let (x4, y4) =
                    ((v1 - v2).normalized().rotate(-f64::consts::FRAC_PI_4) * 10.0 + v2).into();

                canvas_ctx.draw(&Line {
                    x1: x2,
                    y1: y2,
                    x2: x3,
                    y2: y3,
                    color: Color::White,
                });

                canvas_ctx.draw(&Line {
                    x1: x2,
                    y1: y2,
                    x2: x4,
                    y2: y4,
                    color: Color::White,
                });
            }
        }
        canvas_ctx.draw(&self.to_circle(circle_color));
        // TODO: Pass name as &str?
        canvas_ctx.print(
            self.x - self.name.len() as f64 + 1.0,
            self.y - 5.0,
            self.name.clone(),
        );
    }
}

struct App {
    states: std::vec::Vec<Rc<RefCell<FSMState>>>,
    selected_state: Weak<RefCell<FSMState>>,
    secondary_selected_state: Weak<RefCell<FSMState>>,
    new_state_name: Option<String>,
    marker: Marker,
}

impl App {
    fn new() -> Self {
        Self {
            states: vec![],
            selected_state: Weak::new(),
            secondary_selected_state: Weak::new(),
            new_state_name: None,
            marker: Marker::Braille,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Press {
                        if let Some(ref mut new_state_name) = self.new_state_name {
                            match key.code {
                                KeyCode::Char(ch) => new_state_name.push(ch),
                                KeyCode::Backspace => {
                                    if !new_state_name.is_empty() {
                                        new_state_name.pop();
                                    }
                                }
                                KeyCode::Enter => {
                                    let state = Rc::new(RefCell::new(FSMState {
                                        x: 200.0,
                                        y: 200.0,
                                        name: self.new_state_name.take().unwrap(),
                                        next_states: vec![],
                                    }));

                                    self.selected_state = Rc::downgrade(&state);
                                    self.states.push(state);
                                }
                                KeyCode::Esc => self.new_state_name = None,
                                _ => (),
                            }

                            continue;
                        }

                        match key.code {
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Char('s') => {
                                self.secondary_selected_state = self.selected_state;
                                self.selected_state = Weak::new();
                            }
                            KeyCode::Char('c') => {
                                if let Some(selected_state) = self.selected_state.upgrade()
                                    && let Some(secondary_state) =
                                        self.secondary_selected_state.upgrade()
                                {
                                    let old_secondary_next_count =
                                        secondary_state.borrow().next_states.len();
                                    secondary_state.borrow_mut().next_states.retain(|s| {
                                        if let Some(s2) = s.upgrade() {
                                            !Rc::ptr_eq(&s2, &selected_state)
                                        } else {
                                            true
                                        }
                                    });

                                    if old_secondary_next_count
                                        == secondary_state.borrow().next_states.len()
                                    {
                                        secondary_state
                                            .borrow_mut()
                                            .next_states
                                            .push(Rc::downgrade(&selected_state));
                                    }

                                    self.selected_state = Weak::new();
                                    self.secondary_selected_state = Weak::new();
                                }
                            }
                            KeyCode::Char('d') => {
                                if let Some(state) = self.selected_state.upgrade() {
                                    let index = self
                                        .states
                                        .iter()
                                        .position(|s| Rc::ptr_eq(s, &state))
                                        .unwrap();

                                    self.states.swap_remove(index);
                                }
                            }
                            KeyCode::Char('n') => self.new_state_name = Some(String::new()),
                            KeyCode::Char('m') => {
                                let markers = [
                                    Marker::Dot,
                                    Marker::Block,
                                    Marker::Bar,
                                    Marker::Braille,
                                    Marker::HalfBlock,
                                ];

                                self.marker = *markers
                                    .iter()
                                    .skip_while(|m| **m != self.marker)
                                    .nth(1)
                                    .unwrap_or(&markers[0]);
                            }
                            KeyCode::Tab => {
                                if let Some(state) = self.selected_state.upgrade() {
                                    let new_selected = self
                                        .states
                                        .iter()
                                        .skip_while(|s| !Rc::ptr_eq(s, &state))
                                        .nth(1)
                                        .unwrap_or_else(|| self.states.first().unwrap());

                                    self.selected_state = Rc::downgrade(new_selected);
                                } else if !self.states.is_empty() {
                                    self.selected_state =
                                        Rc::downgrade(self.states.first().unwrap());
                                }
                            }
                            KeyCode::Esc => {
                                self.selected_state = Weak::new();
                                self.secondary_selected_state = Weak::new();
                            }
                            KeyCode::Left => {
                                if let Some(selected) = self.selected_state.upgrade() {
                                    selected.borrow_mut().x -= 5.0;
                                }
                            }
                            KeyCode::Right => {
                                if let Some(selected) = self.selected_state.upgrade() {
                                    selected.borrow_mut().x += 5.0;
                                }
                            }
                            KeyCode::Up => {
                                if let Some(selected) = self.selected_state.upgrade() {
                                    selected.borrow_mut().y += 5.0;
                                }
                            }
                            KeyCode::Down => {
                                if let Some(selected) = self.selected_state.upgrade() {
                                    selected.borrow_mut().y -= 5.0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([Constraint::Percentage(70), Constraint::Percentage(30)]);
        let [canvas, menu] = vertical.areas(frame.area());

        frame.render_widget(self.canvas(), canvas);
        frame.render_widget(
            Paragraph::new(if self.new_state_name.is_none() {
                "Press q to exit.
Press tab to switch between states.
Press Esc to unselect.
Use the arrow keys to move states.
Press s to select a state for connection.
Press c to toggle connection between previously selected state to the current.
Press n to create a new state.
Press d to delete the selected state.
Press m to change canvas style."
            } else {
                "Creating new state.
Type state name. Press enter to create.
Press Esc to abort."
            })
            .block(Block::bordered().title("Menu")),
            menu,
        );
    }

    fn canvas(&self) -> impl Widget + '_ {
        Canvas::default()
            .marker(self.marker)
            .paint(|ctx| {
                for state in &self.states {
                    state.borrow().draw(
                        if let Some(selected) = self.selected_state.upgrade()
                            && Rc::ptr_eq(state, &selected)
                        {
                            Color::Yellow
                        } else if let Some(secondary_selected) =
                            self.secondary_selected_state.upgrade()
                            && Rc::ptr_eq(state, &secondary_selected)
                        {
                            Color::Cyan
                        } else {
                            Color::White
                        },
                        ctx,
                    );
                }

                if let Some(new_state_name) = &self.new_state_name {
                    ctx.print(0.0, 0.0, new_state_name.clone());
                }
            })
            .x_bounds([0.0, 500.0])
            .y_bounds([0.0, 500.0])
    }
}
