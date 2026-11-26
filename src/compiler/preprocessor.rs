use log::error;
use crate::compiler::errors::{CompilerError, ErrorPrinter, FileEntry, Span};

pub struct PreProcessor {}

impl<'chunks> PreProcessor {

    pub fn process(&mut self, error_printer: &mut ErrorPrinter, file_path: &std::path::Path) -> Result<(), CompilerError> {
        self.process_file(error_printer, file_path)?;
        Ok(())
    }

    fn process_file(&mut self, error_printer: &mut ErrorPrinter, file_path: &std::path::Path) -> Result<(), CompilerError> {
        let text = std::fs::read_to_string(&file_path).expect("Failed to read file");

        let lines = text.lines();
        let mut position = error_printer.program_text.len();
        let mut curr_line = 0;
        let mut replaces: Vec<(String, String)> = Vec::new();

        let mut text_piece = String::new();


        for unreplaced_line in lines {
            curr_line += 1;

            let mut line = String::from(unreplaced_line);
            for (find, replace) in replaces.iter() {
                line = line.replace(find, replace);
            }

            if !line.starts_with("#") {
                position += unreplaced_line.len();
                text_piece.push_str(unreplaced_line);
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

                    let include_path = &include_path;

                    error_printer.add_entry(
                        &text_piece,
                        file_path.to_string_lossy().to_string(),
                        position,
                        curr_line
                    );

                    text_piece = String::new();
                   self.process_file(error_printer, include_path)?;
                },
                "define" => {
                    let name = parts.next().unwrap();

                    let expression = parts.collect::<Vec<_>>().join(" ");

                    replaces.push((name.to_string(), expression));
                }
                _ => return Err(CompilerError::new(
                    Span::new(position-unreplaced_line.len(), position),
                    "Unknown preprocessor directive"
                )),
            };


        }

        Ok(())
    }
}