use std::fs;
use colored::Colorize;
use crate::compiler::lexer::Lexer;
use crate::compiler::parser::Parser;
use crate::compiler::semantic_analyzer::SemanticAnalyzer;

struct Compiler {
    input_path: String,
    output_path: String,
    dump_flags: String,
}

impl Compiler {
    fn new(input_path: String, output_path: Option<String>, dump_flags: String) -> Compiler {
        let output_path = output_path.unwrap_or_else(|| {
            let path = std::path::Path::new(&input_path);
            let filename = path.file_name();

            let root = crate::config_loader::find_project_root();
            let config = crate::config_loader::load_config();

            root.unwrap().join(config.paths.build).join(filename.unwrap()).with_extension("bin").to_str().unwrap().to_string()
        });
        
        Compiler {
            input_path,
            output_path,
            dump_flags,
        }
    }

    fn compile(&self) -> Result<(), String> {
        println!("Compiling file: {}", self.input_path);

        // Read file
        let file_contents = fs::read_to_string(&self.input_path);
        if file_contents.is_err() {
            return Err(format!("Unable to read file: {}", self.input_path));
        }
        let file_contents = file_contents.unwrap();

        // Preprocessor

        // Lexer file
        let mut lexer = Lexer::new(
            &file_contents,
            self.input_path.as_str()
        );
        let result = lexer.tokenize();
        if result.is_err() {
            return Err(format!(
                "Unable to tokenize file: \n{}",
                result.unwrap_err()
            ));
        }
        let tokens = result.unwrap();
        if self.dump_flags.contains("t") {
            for token in &tokens {
                println!("{}", token);
            }
        }

        // Parser
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if result.is_err() {
            return Err(format!("Unable to parse file:\n{}", result.unwrap_err().to_string()));
        }

        let declarations = result.unwrap();
        if self.dump_flags.contains("a") {
            for node in &declarations {
                println!("{}", node);
            }
        }

        // Semantic analyzer
        // Build type table
        // Build scopes declaring symbols with their types at declaration
        // Assign symbols to identifiers
        // Assign, infer and enforce types using the data in the assigned symbols
        let mut semantic_analyzer = SemanticAnalyzer::new(self.dump_flags.clone(), &declarations);
        let result =semantic_analyzer.analyze();
        if result.is_err() {
            return Err(format!("Unable to parse file:\n{}", result.unwrap_err().to_string()));
        }
        let _scope_handler = result.unwrap();


        // Code Generator


        println!("Compilation done!\nCompiled to: {}", self.output_path);

        Ok(())
    }
}

pub fn compile_file(
    input_path: String,
    output_path: Option<String>,
    dump_flags: String,
) -> Result<(), String> {
    let compiler = Compiler::new(input_path, output_path, dump_flags);

    if let Err(e) = compiler.compile() {
        eprintln!("{}", e.to_string().red());
    }

    Ok(())
}