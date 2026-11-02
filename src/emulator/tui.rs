use std::cmp::{max, min};
use std::sync::mpsc::{Receiver, Sender};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pretty_num::PrettyNumber;
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding, Paragraph, Row, Table};
use crate::emulator::emulator_controller::EmuTuiData;

pub struct Tui {
    terminal: DefaultTerminal,
    emulator_data: EmuTuiData,
    scroll_offset: isize,
    console_lines: Vec<String>,
    console_input: String,
    console_cursor_pos: usize,
}

impl Tui {
    pub fn new() -> Tui {
        Tui {
            terminal: ratatui::init(),
            emulator_data: EmuTuiData::default(),
            scroll_offset: 0,
            console_lines: Vec::new(),
            console_input: String::new(),
            console_cursor_pos: 0,
        }
    }

    pub fn start(&mut self, info_receiver: Receiver<EmuTuiData>, action_sender: Sender<TuiAction>) {
        loop {
            while let Ok(info) = info_receiver.try_recv() {
                self.emulator_data = info
            }

            self.render_info().unwrap();

            let action = self.get_input().unwrap();
            if let Some(action) = action {
                action_sender.send(action.clone()).unwrap();
                match action {
                    TuiAction::Quit => break,
                    _ => {}
                }
            }
        }
    }

    fn render_info(
        &mut self,
    ) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        self.terminal.draw(|frame| {
            let whole_screen = frame.area();

            // Divide up screen
            let rows = Layout::vertical([Constraint::Min(4), Constraint::Percentage(100), Constraint::Min(10)]).split(whole_screen);
            let status_area = rows[0];
            let main_area = rows[1];
            let console_area = rows[2];

            let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(main_area);
            let reg_area = columns[0];
            let mem_area = columns[1];


            // Status
            let status_block = Block::bordered().title("NEX emulator");
            let status_text = format!(
                "Emulator state: {:?}, \
                Instruction pointer: {:#010x}, \
                Instruction: {} \
                Instructions per second last run: {}",
                self.emulator_data.state,
                self.emulator_data.instruction_pointer,
                self.emulator_data.instruction_pointer/4,
                self.emulator_data.instr_per_sec.pretty_format(),
            );

            let status_content = Paragraph::new(status_text).block(status_block);
            frame.render_widget(status_content, status_area);


            // Registers
            let reg_block = Block::bordered().title("Registers");
            let reg_rows = self.emulator_data.registers.iter().enumerate().skip(self.scroll_offset as usize).map(|(i, register)| {
                Row::new(vec![
                    format!("r{:<3}:", i),
                    format!("{:#010X}", register),
                ])
            });

            let column_widths = [Constraint::Length(5), Constraint::Min(10)];

            let reg_content = Table::new(reg_rows, column_widths).header(Row::new(vec!["Reg", "Value"])).block(reg_block);
            frame.render_widget(reg_content, reg_area);


            // Memory
            let mem_block = Block::bordered().title("Memory");
            let mem_rows = self.emulator_data.memory_slice.iter().take(200).enumerate().map(|(addr, value)| {
                Row::new(vec![
                    format!("{:#04X} :", addr),
                    format!(" {:#04X}", value),
                    format!("decoded value: {}", *value as char,
                    ),
                ])
            });

            let column_widths = [Constraint::Length(6), Constraint::Length(6), Constraint::Min(16)];
            let mem_content = Table::new(mem_rows, column_widths).block(mem_block);
            frame.render_widget(mem_content, mem_area);


            // Console
            let console_block = Block::bordered().title("Console");

            // Set cursor
            frame.set_cursor_position(Position::new(console_area.x + 3 + self.console_cursor_pos as u16, console_area.y + console_area.height - 2));

            let line_count = console_area.height - 3;
            let mut console_text = self.console_lines[self.console_lines.len().saturating_sub(line_count as usize)..self.console_lines.len()].join("\n");
            if console_text.len() > 0 {
                console_text.push('\n');
            }

            console_text.push_str(format!("> {}", self.console_input.as_str()).as_str());

            let top = console_area.height.saturating_sub(2 + console_text.lines().count() as u16);
            let console_content = Paragraph::new(console_text).block(console_block.padding(Padding::new(0, 0, top, 0)));
            frame.render_widget(console_content, console_area)

        })?;
        Ok(None)
    }

    fn get_input(&mut self) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        if event::poll(std::time::Duration::from_millis(30))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Enter => { return self.handle_enter() },
                        KeyCode::Backspace => { self.delete_char(); },
                        KeyCode::Left => { self.move_cursor_left(); },
                        KeyCode::Right => { self.move_cursor_right(); },
                        KeyCode::Char(char) => { self.enter_char(char); },
                        _ => {}
                    };
                }
            }
        }

        Ok(None)
    }

    fn print_line(&mut self, text: &str) {
        self.console_lines.push(text.to_string());
    }

    fn move_cursor_left(&mut self) {
        let pos = self.clamp_cursor(self.console_cursor_pos.saturating_sub(1));
        self.console_cursor_pos = pos;
    }

    fn move_cursor_right(&mut self) {
        let pos = self.clamp_cursor(self.console_cursor_pos.saturating_add(1));
        self.console_cursor_pos = pos;
    }

    fn clamp_cursor(&self, pos: usize) -> usize {
        min(max(0, pos), self.console_input.len())
    }

    fn handle_enter(&mut self) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        let input = self.console_input.clone();
        self.console_input = String::new();
        self.console_cursor_pos = 0;

        self.console_lines.push(format!("> {}", input));

        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 1 { return Ok(None); }

        let command = parts[0];
        match command {
            "quit" | "q" | "exit" => { return Ok(Some(TuiAction::Quit)) },
            "step" => { return Ok(Some(TuiAction::Step)) },
            "run" => { return Ok(Some(TuiAction::Run)) },
            "stop" => { return Ok(Some(TuiAction::Stop)) },
            "help" => { self.print_help() },
            _ => { self.print_line(format!("unknown command: {}", command).as_str()); }
        }

        Ok(None)
    }

    fn enter_char(&mut self, char: char) {
        let mut before = self.console_input[0..self.console_cursor_pos].to_string();
        let after = self.console_input.chars().skip(self.console_cursor_pos);

        before.push(char);

        after.into_iter().for_each(|c| before.push(c));

        self.console_cursor_pos += 1;
        self.console_input = before;
    }

    fn delete_char(&mut self) {
        self.console_cursor_pos = self.console_cursor_pos.saturating_sub(1);
        let mut before = self.console_input[0..self.console_cursor_pos].to_string();
        let after = self.console_input.chars().skip(self.console_cursor_pos+1).collect::<String>();

        before.push_str(&after);

        self.console_input = before;
    }

    fn print_help(&mut self) {
        self.print_line("Commands:");
        self.print_line("help: prints this help message");
        self.print_line("quit: quit the tui");
        self.print_line("step: execute the next instruction");
        self.print_line("run: start executing automatically");
        self.print_line("stop: stop executing automatically");
    }
}

#[derive(Clone)]
pub enum TuiAction {
    Quit,
    Step,
    Run,
    Stop,
}