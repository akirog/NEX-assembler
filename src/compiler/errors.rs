use std::fmt::{Debug, Display};

pub struct SpanPosition {
    line: usize,
    column: usize,
}

impl SpanPosition {
    pub fn new(line: usize, column: usize) -> SpanPosition {
        SpanPosition { line, column }
    }
}

pub struct Span<'a> {
    /// Area to be highlighted with error carrots
    start: SpanPosition,
    end: SpanPosition,

    /// Source
    source_text: &'a str,
    file_name: &'a str,
}

impl<'a> Span<'a> {
    pub fn new(start: SpanPosition, end: SpanPosition, source_text: &'a str, file_name: &'a str) -> Span<'a> {
        Span {
            start,
            end,
            source_text,
            file_name
        }
    }
}


pub struct CompilerError<'a> {
    span: Span<'a>,

    message: String,
    hint: Option<String>,
}

impl CompilerError<'_> {
    pub fn new(span: Span, message: String, hint: String) -> CompilerError {
        let hint = match hint.as_str() {
            "" => None,
            _ => Some(hint),
        };

        CompilerError {
            span,
            message,
            hint
        }
    }
}


impl Display for CompilerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();

        message.push_str(format!("Error on line {} in {}\n", self.span.start.line+1, self.span.file_name).as_str());

        for text in self.span.source_text.lines().skip(self.span.start.line).take(self.span.end.line - self.span.start.line + 1) {
            // Lines to be shown
            message += text;
            message += "\n";

            for _ in 0..self.span.start.column { message.push('~') }
            for _ in self.span.start.column..self.span.end.column+1  { message.push('^') }
            message.push('\n');
        }


        // Message
        message.push_str(&self.message);
        message.push('\n');

        // Hint
        if let Some(hint) = &self.hint {
            message.push_str(hint);
            message.push('\n');
        }

        write!(f, "{}", &message)
    }
}

impl Debug for CompilerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{}", self.to_string());
        Display::fmt(&self, f)
    }
}