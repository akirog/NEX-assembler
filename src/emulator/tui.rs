use std::cmp::{max, min};
use std::io::Stdout;
use std::sync::mpsc::{Receiver, Sender};
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pretty_num::PrettyNumber;
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding, Paragraph, Row, Table};
use crate::Cli;
use crate::emulator::cli::TuiCli;
use crate::emulator::emulator_controller::EmuTuiData;

pub struct Tui {
    emulator_data: EmuTuiData,
    cli: TuiCli,
    scroll_offset: isize,
    mem_start: u32,
    mem_size: usize,
}

impl Tui {
    pub fn new() -> Tui {
        Tui {
            cli: TuiCli::new(),
            emulator_data: EmuTuiData::default(),
            scroll_offset: 0,
            mem_start: 0,
            mem_size: 10,
        }
    }

    pub fn start(&mut self, info_receiver: Receiver<EmuTuiData>, action_sender: Sender<TuiAction>) {
        let mut terminal = ratatui::init();

        loop {
            while let Ok(info) = info_receiver.try_recv() {
                self.emulator_data = info
            }

            self.render_info(&mut terminal).unwrap();

            let action = self.get_input().unwrap();
            if let Some(action) = action {
                let action = match action {
                    TuiAction::UpdateMemArea(area) => {
                        self.mem_start = area.0;
                        TuiAction::UpdateMemArea((area.0, self.mem_size as u32))
                    },
                    _ => action,
                };
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
        terminal: &mut DefaultTerminal,
    ) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        terminal.draw(|frame| {
            let whole_screen = frame.area();

            // Divide up screen
            let rows = Layout::vertical([Constraint::Min(4), Constraint::Percentage(100), Constraint::Min(10)]).split(whole_screen);
            let status_area = rows[0];
            let main_area = rows[1];
            let console_area = rows[2];

            let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(main_area);
            let reg_area = columns[0];
            let mem_area = columns[1];
            self.mem_size = (mem_area.height as usize).saturating_sub(2);


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
            let mem_content = self.get_mem_content(mem_area);
            frame.render_widget(mem_content, mem_area);


            // Set cursor
            frame.set_cursor_position(Position::new(console_area.x + 3 + self.cli.cursor_pos as u16, console_area.y + console_area.height - 2));


            // Console
            let console_content = self.cli.get_content(console_area);
            frame.render_widget(console_content, console_area)

        })?;
        Ok(None)
    }

    fn get_mem_content(&self, area: Rect) -> impl Widget {

        let mem_block = Block::bordered().title("Memory");
        let mem_rows = self.emulator_data.memory_slice.iter().enumerate().take((area.height.saturating_sub(2)) as usize).map(|(addr, value)| {
            Row::new(vec![
                format!("{:#010X} :", addr + self.mem_start as usize),
                format!(" {:#04X}", value),
                format!("decoded value: {}", *value as char,
                ),
            ])
        });

        let column_widths = [Constraint::Length(12), Constraint::Length(6), Constraint::Min(16)];
        let mem_content = Table::new(mem_rows, column_widths).block(mem_block);
        mem_content
    }

    fn get_input(&mut self) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        if event::poll(std::time::Duration::from_millis(30))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => { return Ok(Some(TuiAction::Quit)); }
                        KeyCode::Enter => { return self.cli.handle_enter() },
                        KeyCode::Backspace => { self.cli.delete_char(); },
                        KeyCode::Left => { self.cli.move_cursor_left(); },
                        KeyCode::Right => { self.cli.move_cursor_right(); },
                        KeyCode::Char(char) => { self.cli.enter_char(char); },
                        _ => {}
                    };
                }
            }
        }

        Ok(None)
    }
}

#[derive(Clone)]
pub enum TuiAction {
    Quit,
    Step,
    Run,
    Stop,
    /// Address, Size
    UpdateMemArea((u32, u32)),
}