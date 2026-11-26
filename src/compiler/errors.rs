use std::fmt::Debug;


#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span {
            start,
            end,
        }
    }

    /// Extends this span by other example:
    /// span with 'hello' extended by span with 'world' on a line with text "hello world" returns span with 'hello world'
    pub fn extend(&self, other: Span) -> Span {
        Span::new(
            self.start,
            other.end,
        )
    }
}


pub struct CompilerError {
    span: Span,
    message: String,
}

impl CompilerError {
    pub fn new(span: Span, message: &str) -> CompilerError {
        CompilerError {
            span,
            message: message.to_string(),
        }
    }
}



pub struct ErrorPrinter {
    pub program_text: String,
    pub file_entries: Vec<FileEntry>
}

#[derive(Debug)]
pub struct FileEntry {
    pub character_pos: usize,
    pub starting_line: usize,
    pub file_path: String,
}

impl ErrorPrinter {
    pub fn new() -> ErrorPrinter {
        ErrorPrinter {
            program_text: String::new(),
            file_entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, text: &str, file_path: String, character_pos: usize, starting_line: usize) {
        self.program_text.push_str(text);
        self.file_entries.push(FileEntry {
            character_pos,
            starting_line,
            file_path,
        });
    }

    pub fn print_error(&self, err: CompilerError) -> String {
        let mut message = String::new();

        let mut file_entry = None;
        let mut earliest_pos = 0;
        for entry in &self.file_entries {
            if entry.character_pos <= err.span.start && entry.character_pos >= earliest_pos {
                file_entry = Some(entry);
                earliest_pos = entry.character_pos;
            }
        }

        let file_entry = file_entry.expect(
            &format!("Invalid error position: {}\nEntries checked: {:?}", err.span.start, self.file_entries)
        );

        let start_line = self.program_text[file_entry.character_pos..err.span.start]
            .lines().count() + file_entry.starting_line;
        let end_line = self.program_text[file_entry.character_pos..=err.span.end]
            .lines().count() + file_entry.starting_line;

        let start_column = self.program_text[file_entry.character_pos..err.span.start]
            .lines().last().unwrap_or_default().len();
        let end_column = self.program_text[file_entry.character_pos..=err.span.end]
            .lines().last().unwrap_or_default().len();

        message += &format!("Error in file: {}:{}:{}\n", &file_entry.file_path, start_line, start_column + 1);

        let code_lines = self.program_text[file_entry.character_pos..]
            .lines().enumerate().filter(|(i, _)| {
            i + 1 + file_entry.starting_line >= start_line && i + 1 + file_entry.starting_line <= end_line
        }).map(|(_, line)| line).collect::<Vec<&str>>();

        for (i, line) in code_lines.iter().enumerate() {
            message += &format!("{}\n", line);

            let start = if i == 0 { start_column } else { 0 };
            let end = if i == code_lines.len() - 1 { end_column } else { line.len() };

            for _ in 0..start {
                message += "~";
            }

            for _ in start..end {
                message += "^";
            }

            message += "\n";
        }

        message += &err.message;

        message
    }

    pub fn get_text(&self) -> &str {
        &self.program_text
    }
}