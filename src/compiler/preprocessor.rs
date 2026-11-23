use crate::compiler::errors::CompilerError;

pub struct PreProcessor {}

impl<'chunks> PreProcessor {

    pub fn process(&mut self, file_path: &std::path::Path) -> Result<Vec<CodeChunk>, CompilerError<'chunks>> {
        self.process_file(file_path)
    }

    fn process_file(&mut self, file_path: &std::path::Path) -> Result<Vec<CodeChunk>, CompilerError<'chunks>> {
        let text = std::fs::read_to_string(&file_path).expect("Failed to read file");

        let mut chunks = Vec::new();
        chunks.push(CodeChunk {
            starting_line: 0,
            text: String::new(),
        });

        let mut curr_line = 0;
        let lines = text.lines();

        for line in lines {
            if !line.starts_with("#") {
                chunks.last_mut().unwrap().text.push_str(line);
                curr_line += 1;
            }

            let mut parts = line.split_whitespace();
            match &parts.next().unwrap()[1..] {
                "include" => {
                    let file_path = std::path::Path::new(parts.next().unwrap());
                    let included_chunks = self.process_file(file_path)?;
                    chunks.extend(included_chunks);
                    curr_line += 1;
                    chunks.push(CodeChunk {
                        starting_line: curr_line,
                        text: String::new(),
                    });
                },
                _ => chunks.last_mut().unwrap().text.push_str(line),
            };


        }

        Ok(chunks)
    }
}


pub struct CodeChunk {
    starting_line: usize,
    text: String
}