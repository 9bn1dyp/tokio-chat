use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::io;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, Default)]
pub struct App {
    input: Input,
    input_mode: InputMode,
    messages: Vec<String>,
    exit: bool,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // see event poll
    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        if let Event::Key(key) = event {
            match self.input_mode {
                // Normal mode keyboard handling
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => self.start_editing(),
                    KeyCode::Char('q') => self.exit(),
                    _ => {}
                },

                // Editing mode keyboard handling
                InputMode::Editing => match key.code {
                    KeyCode::Enter => self.push_message(),
                    KeyCode::Esc => self.stop_editing(),
                    _ => {
                        self.input.handle_event(&event);
                    }
                },
            }
        }
        Ok(())
    }

    fn start_editing(&mut self) {
        self.input_mode = InputMode::Editing
    }

    fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal
    }

    fn push_message(&mut self) {
        // prepend " "
        let msg = format!(" {}", self.input.value_and_reset());
        self.messages.push(msg)
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 90%: Chat log, 10%: Input
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(area);

        // Top Box for chat log
        let height = outer_layout[0].height.saturating_sub(2); // sub 2 for border
        // Take as many messages from self.message that fits in the current area
        // Reverse then take as many fits, reverse again and collect
        let visible_mes = self
            .messages
            .iter()
            .rev()
            .take(height as usize)
            .rev()
            .map(|m| Line::raw(m.clone()))
            .collect::<Vec<_>>();

        Paragraph::new(visible_mes)
            .block(Block::bordered().title(Line::from(" Chat room ".bold())))
            .render(outer_layout[0], buf);

        // Bottom box for input
        let width = area.width.max(3) - 3;
        let scroll = self.input.visual_scroll(width as usize);
        let style = match self.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Color::Yellow.into(),
        };
        Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title(" Input "))
            .render(outer_layout[1], buf);
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[cfg(test)]
mod tests {}
