use client::Message;
use client::User;
use crossbeam::channel::{Receiver, Sender, unbounded};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct App {
    input: Input,
    input_mode: InputMode,
    username: String,
    messages: Vec<String>,
    exit: bool,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    #[default]
    Normal,
    Editing,
    Username,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            self.ui_rec();

            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // see event poll
    fn handle_events(&mut self) -> anyhow::Result<()> {
        let event = event::read()?;
        if let Event::Key(key) = event {
            match self.input_mode {
                // Normal mode keyboard handling
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => self.start_editing(),
                    KeyCode::Char('/') => self.start_username(),
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

                InputMode::Username => match key.code {
                    KeyCode::Enter => self.push_username(),
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

    fn start_username(&mut self) {
        self.input_mode = InputMode::Username
    }

    fn stop_editing(&mut self) {
        self.input_mode = InputMode::Normal
    }

    fn push_message(&mut self) {
        // check if msg null
        if self.input.value().is_empty() {
            return;
        }
        // format to Message type here
        let msg = Message {
            username: User(self.username.clone()),
            message: self.input.value_and_reset(),
        };
        // send input text via server sender
        self.sender.send(msg).unwrap();
        self.input_mode = InputMode::Normal;
    }

    fn push_username(&mut self) {
        // check if username null
        if self.input.value().is_empty() {
            return;
        }
        self.username = self.input.value_and_reset();
        self.input_mode = InputMode::Normal;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn ui_rec(&mut self) {
        // receive Message type from receiver<Message>
        while let Ok(value) = self.receiver.try_recv() {
            // format Message and push into messages Vec
            self.messages
                .push(format!(" {} - {}", value.username, value.message));
        }
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
            InputMode::Username => Color::Blue.into(),
        };
        let title = match self.input_mode {
            InputMode::Normal => " Press <i> to insert | </> set username | <q> to quit ",
            InputMode::Editing => " Press <ENTER> to submit | <ESC> to escape ",
            InputMode::Username => " Press <ENTER> to submit | <ESC> to escape ",
        };
        Paragraph::new(self.input.value())
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title(title))
            .render(outer_layout[1], buf);
    }
}

// receive from ui send to server
fn server_rec(mut stream: TcpStream, receiver: Receiver<Message>) {
    while let Ok(value) = receiver.recv() {
        // append \n and format to json
        stream
            .write_all(
                format!(
                    "{{\"username\":\"{}\",\"message\":\"{}\"}}\n",
                    value.username, value.message
                )
                .as_bytes(),
            )
            .unwrap();
    }
}

// receive from server send to ui
fn server_sen(mut stream: TcpStream, sender: Sender<Message>) -> anyhow::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        // convert bytes to json string
        let msg = String::from_utf8_lossy(&buf[..n]).to_string();
        // try convert to message struct
        let message_struct: Message = serde_json::from_str(&msg)?;
        // send to ui receiver
        sender.send(message_struct).unwrap();
    }
    Ok(())
}

fn ui(sender: Sender<Message>, receiver: Receiver<Message>) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App {
        input: "".into(),
        input_mode: InputMode::Normal,
        messages: vec![],
        username: String::from("Guest"),
        exit: false,
        sender,
        receiver,
    }
    .run(&mut terminal);
    ratatui::restore();
    app_result
}

fn main() -> Result<(), Box<dyn Error>> {
    let (s1, r1) = unbounded::<Message>();
    let (s2, r2) = unbounded::<Message>();
    let stream = TcpStream::connect("127.0.0.1:8080")?;
    let stream2 = stream.try_clone()?;

    // UI sends to Server, Server sends to Receiver
    let ui_handle = thread::spawn(|| ui(s1, r2));
    let server_rec_handle = thread::spawn(move || server_rec(stream, r1));
    let server_sen_handle = thread::spawn(move || server_sen(stream2, s2));

    ui_handle.join().unwrap().unwrap();
    server_rec_handle.join().unwrap();
    server_sen_handle.join().unwrap().unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {}
