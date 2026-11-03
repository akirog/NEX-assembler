use crate::emulator::cli::TuiCli;
use crate::emulator::emulator_controller::EmuTuiData;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pretty_num::PrettyNumber;
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Paragraph, Row, Table};
use std::sync::mpsc::{Receiver, Sender};


const BACKGROUND_EVEN_ROW: Color = Color::Rgb(22, 22, 30);
const BACKGROUND_ODD_ROW: Color = Color::Rgb(30, 30, 42);
const BACKGROUND_INSTRUCTION_POINTER: Color = Color::Rgb(70, 55, 15);
const BACKGROUND_BASE_POINTER: Color = Color::Rgb(50, 50, 55);
const BACKGROUND_STACK_POINTER: Color = Color::Rgb(50, 50, 55);


const FOREGROUND_ADDRESS: Color = Color::Rgb(110, 170, 220);
const FOREGROUND_BYTE: Color = Color::Rgb(220, 220, 230);
const FOREGROUND_ZERO_BYTE: Color = Color::Rgb(90, 90, 105);
const FOREGROUND_TEXT: Color = Color::Rgb(130, 200, 130);
const FOREGROUND_INSTRUCTION_POINTER: Color = Color::Rgb(255, 210, 90);




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
            self.mem_size = (mem_area.height as usize * 4).saturating_sub(2);


            // Background
            let background_block = Block::default().style(Style::default().bg(BACKGROUND_ODD_ROW));
            frame.render_widget(background_block, whole_screen);


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
            self.render_registers(frame, reg_area);


            // Memory
            self.render_memory(frame, mem_area);


            // Set cursor
            frame.set_cursor_position(Position::new(console_area.x + 3 + self.cli.cursor_pos as u16, console_area.y + console_area.height - 2));


            // Console
            let console_content = self.cli.get_content(console_area);
            frame.render_widget(console_content, console_area)

        })?;
        Ok(None)
    }


    fn render_registers(&self, frame: &mut Frame, area: Rect) {
        let reg_block = Block::bordered().title("Registers");
        let reg_rows = self.emulator_data.registers.iter().enumerate().skip(self.scroll_offset as usize).map(|(i, register)| {
            let mut row = Row::new(vec![
                format!("r{:<3}:", i),
                format!("{:#010X}", register),
            ]);

            if i % 2 == 0 {
                row = row.style(Style::default().bg(BACKGROUND_EVEN_ROW));
            }

            row
        });

        let column_widths = [Constraint::Length(5), Constraint::Min(12)];

        let reg_content = Table::new(reg_rows, column_widths).header(Row::new(vec!["Reg", "Value"])).block(reg_block);
        frame.render_widget(reg_content, area);
    }


    fn render_memory(&self, frame: &mut Frame, area: Rect) {

        let mem_block = Block::bordered().title("Memory");
        let mem_rows = self.emulator_data.memory_slice.chunks(4).enumerate().take(area.height.saturating_sub(2) as usize).map(|(addr, value)| {
            let addr = addr*4 + self.mem_start as usize;

            let addr_text = format!("{:#010X} :", addr);
            let value_text = format!(" {:#04X} {:#04X} {:#04X} {:#04X}", value[0], value[1], value[2], value[3]);
            let decode_text = format!("decoded value: {} {} {} {}",
                                      if value[0] == 0 { ' ' } else { value[0] as char },
                                      if value[1] == 0 { ' ' } else { value[1] as char },
                                      if value[2] == 0 { ' ' } else { value[2] as char },
                                      if value[3] == 0 { ' ' } else { value[3] as char }
            );

            let mut row = Row::new(vec![
                Cell::from(addr_text).style(Style::default().fg(FOREGROUND_ADDRESS)),
                Cell::from(value_text).style(Style::default().fg(FOREGROUND_BYTE)),
                Cell::from(decode_text).style(Style::default().fg(FOREGROUND_TEXT)),
            ]);


            row = if addr % 8 == 4 {
                row.style(Style::default().bg(BACKGROUND_EVEN_ROW))
            } else {
                row.style(Style::default().bg(BACKGROUND_ODD_ROW))
            };

            row = if (addr as u32) == self.emulator_data.registers[30] {
                row.style(Style::default().bg(BACKGROUND_BASE_POINTER))
            } else if (addr as u32) == self.emulator_data.registers[29] {
                row.style(Style::default().bg(BACKGROUND_STACK_POINTER))
            } else {
                row
            };

            row = if (addr as u32) == self.emulator_data.instruction_pointer
            {
                row.style(Style::default().bg(BACKGROUND_INSTRUCTION_POINTER))
            } else {
                row
            };

            row
        });

        let column_widths = [Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)];
        let mem_content = Table::new(mem_rows, column_widths).block(mem_block);
        frame.render_widget(mem_content, area);
    }

    fn get_input(&mut self) -> Result<Option<TuiAction>, Box<dyn std::error::Error>> {
        if event::poll(std::time::Duration::from_millis(30))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => { return Ok(Some(TuiAction::Quit)); }
                        KeyCode::Enter => { return self.cli.handle_enter() },
                        KeyCode::Backspace => { self.cli.delete_char(); },
                        KeyCode::Up => { self.cli.select_prev(); }
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
    Step(u32),
    Run,
    Stop,
    /// Address, Size
    UpdateMemArea((u32, u32)),
}