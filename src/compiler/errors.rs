use std::fmt::{Debug, Display};


#[derive(Copy, Clone, Debug)]
pub struct SpanPosition {
    line: usize,
    column: usize,
}


impl SpanPosition {
    pub fn new(line: usize, column: usize) -> SpanPosition {
        SpanPosition { line, column }
    }
}

#[derive(Copy, Clone, Debug)]
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

    /// Extends this span by other example:
    /// span with 'hello' extended by span with 'world' on a line with text "hello world" returns span with 'hello world'
    pub fn extend(&self, other: Span<'a>) -> Span<'a> {
        Span::new(
            self.start,
            other.end.clone(),
            self.source_text,
            self.file_name
        )
    }
}


pub struct CompilerError<'a> {
    span: Span<'a>,

    message: String,
}

impl CompilerError<'_> {
    pub fn new<'a>(span: Span<'a>, message: &str) -> CompilerError<'a> {
        CompilerError {
            span,
            message: message.to_string(),
        }
    }
}


impl Display for CompilerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();

        message.push_str(format!("Column start: {}\n", self.span.start.column).as_str());

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

        write!(f, "{}", &message)
    }
}

impl Debug for CompilerError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("{}", self.to_string());
        Display::fmt(&self, f)
    }
}