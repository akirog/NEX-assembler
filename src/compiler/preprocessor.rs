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
            text: String::new(),
            file_name: file_path.to_string_lossy().to_string(),
        });

        let lines = text.lines();
        let mut replaces: Vec<(String, String)> = Vec::new();

        for unreplaced_line in lines {
            let mut line = String::from(unreplaced_line);
            for (find, replace) in replaces.iter() {
                line = line.replace(find, replace);
            }


            if !line.starts_with("#") {
                let last_chunk = chunks.last_mut().unwrap();
                last_chunk.text.push_str(&line);
                last_chunk.text.push('\n');
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

                    let included_chunks = self.process_file(include_path)?;
                    chunks.extend(included_chunks);
                    chunks.push(CodeChunk {
                        text: String::new(),
                        file_name: file_path.to_string_lossy().to_string(),
                    });
                },
                "define" => {
                    let name = parts.next().unwrap();

                    let expression = parts.collect::<Vec<_>>().join("");

                    let value = rustyard::shunting_yard::ShuntingYard::new().calculate(&expression);
                    let Ok(value) = value else {
                        panic!("{:?}", value);
                    };

                    replaces.push((name.to_string(), value.to_string()));
                }
                _ => chunks.last_mut().unwrap().text.push_str(&line),
            };


        }

        Ok(chunks)
    }
}


pub struct CodeChunk {
    pub text: String,
    pub file_name: String,
}