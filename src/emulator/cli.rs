use std::cmp::{max, min};
use ratatui::prelude::{Rect, Widget};
use ratatui::widgets::{Block, Padding, Paragraph};
use crate::emulator::tui::TuiAction;

pub struct TuiCli {
    console_lines: Vec<String>,
    console_input: String,
    pub cursor_pos: usize,
}

impl TuiCli {
    pub fn new() -> Self {
        Self {
            console_lines: Vec::new(),
            console_input: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn get_content(&self, console_area: Rect) -> impl Widget {
        let console_block = Block::bordered().title("Console");

        let line_count = console_area.height.saturating_sub(3);
        let mut console_text = self.console_lines[self.console_lines.len().saturating_sub(line_count as usize)..self.console_lines.len()].join("\n");
        if console_text.len() > 0 {
            console_text.push('\n');
        }

        console_text.push_str(format!("> {}", self.console_input.as_str()).as_str());

        let top = console_area.height.saturating_sub(2 + console_text.lines().count() as u16);
        let console_content = Paragraph::new(console_text).block(console_block.padding(Padding::new(0, 0, top, 0)));
        console_content
    }

    pub fn print_line(&mut self, text: &str) {
        self.console_lines.push(text.to_string());
    }

    pub fn select_prev(&mut self) {
        self.console_input = self.console_lines.last().unwrap_or(&"> ".to_string()).to_string().strip_prefix("> ").unwrap().to_string();
    }

    pub fn move_cursor_left(&mut self) {
        let pos = self.clamp_cursor(self.cursor_pos.saturating_sub(1));
        self.cursor_pos = pos;
    }

    pub fn move_cursor_right(&mut self) {
        let pos = self.clamp_cursor(self.cursor_pos.saturating_add(1));
        self.cursor_pos = pos;
    }

    pub fn clamp_cursor(&self, pos: usize) -> usize {
        min(max(0, pos), self.console_input.len())
    }

    pub fn handle_enter(&mut self) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        let input = self.console_input.clone();
        self.console_input = String::new();
        self.cursor_pos = 0;

        self.console_lines.push(format!("> {}", input));

        let parts = input.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 1 { return Ok(None); }

        let command = parts[0];
        match command {
            "quit" | "q" | "exit" => { return Ok(Some(TuiAction::Quit)) },
            "step" => { return Ok(self.parse_step_instruction(parts)) },
            "run" => { return Ok(Some(TuiAction::Run)) },
            "mem" => { return Ok(self.parse_mem_instruction(parts)) }
            "stop" => { return Ok(Some(TuiAction::Stop)) },
            "help" => { self.print_help() },
            _ => { self.print_line(format!("unknown command: {}", command).as_str()); }
        }

        Ok(None)
    }

    fn parse_step_instruction(&mut self, parts: Vec<&str>) -> Option<TuiAction> {
        if parts.len() == 1 { return Some(TuiAction::Step(1)) };

        let mut count = parts[1].to_string();
        let mut radix = 10;
        if count.starts_with("0x") || count.starts_with("0X") {
            radix = 16;
            count = count.trim_start_matches("0x").to_string();
        };

        let result = u32::from_str_radix(&count, radix);
        if result.is_err() {
            self.print_line(format!("Invalid step count: {}", count).as_str());
            return None;
        }
        let count = result.unwrap();

        Some(TuiAction::Step(count))

    }

    fn parse_mem_instruction(&mut self, parts: Vec<&str>) -> Option<TuiAction> {
        if parts.len() < 2 { self.print_line("Missing mem address after mem instruction"); return None;}

        let mut address = parts[1].to_string();

        let mut radix = 10;
        if address.starts_with("0x") || address.starts_with("0X") {
            radix = 16;
            address = address.trim_start_matches("0x").to_string();
        };

        let result = u32::from_str_radix(&address, radix);
        if result.is_err() {
            self.print_line(format!("Invalid mem address: {}", address).as_str());
            return None;
        }
        let address = result.unwrap();

        Some(TuiAction::UpdateMemArea((address, 0)))
    }

    pub fn enter_char(&mut self, char: char) {
        let mut before = self.console_input[0..self.cursor_pos].to_string();
        let after = self.console_input.chars().skip(self.cursor_pos);

        before.push(char);

        after.into_iter().for_each(|c| before.push(c));

        self.cursor_pos += 1;
        self.console_input = before;
    }

    pub fn delete_char(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
        let mut before = self.console_input[0..self.cursor_pos].to_string();
        let after = self.console_input.chars().skip(self.cursor_pos +1).collect::<String>();

        before.push_str(&after);

        self.console_input = before;
    }

    pub fn print_help(&mut self) {
        self.print_line("Commands:");
        self.print_line("help: prints this help message");
        self.print_line("quit: quit the tui");
        self.print_line("step [count]: execute the next instruction if no count given, otherwise executes {count} instructions");
        self.print_line("run: start executing automatically");
        self.print_line("stop: stop executing automatically");
        self.print_line("mem [addr]: sets memory view address");
    }
}