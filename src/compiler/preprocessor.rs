use crate::compiler::errors::{CompilerError, ErrorPrinter, Span};

pub struct PreProcessor {}

impl<'chunks> PreProcessor {

    pub fn process(&mut self, error_printer: &mut ErrorPrinter, file_path: &std::path::Path) -> Result<(), CompilerError> {
        self.process_file(error_printer, file_path)?;
        Ok(())
    }

    fn process_file(&mut self, error_printer: &mut ErrorPrinter, file_path: &std::path::Path) -> Result<(), CompilerError> {
        let text = std::fs::read_to_string(&file_path).expect("Failed to read file");

        let lines = text.lines();
        let mut start_position = error_printer.program_text.len();
        let mut curr_line = 0;
        let mut next_line = 0;
        let mut replaces: Vec<(String, String)> = Vec::new();

        let mut text_piece = String::new();


        for unreplaced_line in lines {
            next_line += 1;

            let mut line = String::from(unreplaced_line);
            for (find, replace) in replaces.iter() {
                line = line.replace(find, replace);
            }

            if !line.starts_with("#") {
                text_piece.push_str(unreplaced_line);
                text_piece.push('\n');
                continue;
            }


            let mut parts = line.split_whitespace();
            match &parts.next().unwrap()[1..] {
                "include" => {
                    let path_part = parts.next().unwrap();

                    let given_path = std::path::Path::new(path_part[1..path_part.len() - 1].trim());
                    let root = crate::config_loader::find_project_root().expect("Couldnt find project root.");

                    let include_path = if path_part.starts_with('<') && path_part.ends_with('>') {
                        let config = crate::config_loader::load_config().expect("Couldnt load config file.");
                        let path = root.join(config.paths.libs.as_path().join(given_path));
                        path
                    } else if path_part.starts_with('"') && path_part.ends_with('"') {
                        let path = root.join(given_path);
                        path
                    } else {
                        panic!("Invalid include path, missing \"\" or <>");
                    };

                    if text_piece.len() > 0 {
                        error_printer.add_entry(
                            &text_piece,
                            file_path.to_string_lossy().to_string(),
                            start_position,
                            curr_line
                        );
                    }

                    let include_path = &include_path;

                    text_piece = String::new();
                    curr_line = next_line - 1;
                   self.process_file(error_printer, include_path)?;
                    start_position = error_printer.program_text.len();
                },
                "define" => {
                    let name = parts.next().unwrap();

                    let expression = parts.collect::<Vec<_>>().join(" ");

                    let value = match rustyard::ShuntingYard::new().calculate(&expression) {
                        Err(_) => expression,
                        Ok(n) => (n as i64).to_string()
                    };

                    replaces.push((name.to_string(), value));
                }
                _ => return Err(CompilerError::new(
                    Span::new(start_position - unreplaced_line.len(), start_position),
                    "Unknown preprocessor directive"
                )),
            };


        }

        error_printer.add_entry(
            &text_piece,
            file_path.to_string_lossy().to_string(),
            start_position,
            curr_line
        );

        Ok(())
    }
}