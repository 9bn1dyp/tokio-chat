use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};

#[derive(Debug, Default)]
pub struct App {
    messages: Vec<String>,
    exit: bool,
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.exit();
        }
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
        // todo()! add red/green to sginify if we're connected
        let bottom_title = Line::from(" Connection: ".bold());
        Paragraph::new(Line::from(vec![" Chat".into()]))
            .left_aligned()
            .block(Block::bordered().title(bottom_title))
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
