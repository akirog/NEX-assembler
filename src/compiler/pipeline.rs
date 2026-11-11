use crate::DumpKind;
use crate::compiler::lexer::Lexer;
use crate::compiler::lowering_stages;
use crate::compiler::parser::Parser;
use crossterm::style::Stylize;
use std::fs;

struct Compiler {
    input_path: String,
    output_path: String,
    dump_flags: Vec<DumpKind>,
}

impl Compiler {
    fn new(input_path: String, output_path: Option<String>, dump_flags: Vec<DumpKind>) -> Compiler {
        let output_path = output_path.unwrap_or_else(|| {
            let path = std::path::Path::new(&input_path);
            let filename = path.file_name();

            let Some(root) = crate::config_loader::find_project_root() else { panic!("Failed to find project root.") };
            let Some(config) = crate::config_loader::load_config() else { panic!("Failed to load config file, nex.toml not found.") };

            root.join(config.paths.build).join(filename.unwrap()).with_extension("bin").to_str().unwrap().to_string()
        });
        
        Compiler {
            input_path,
            output_path,
            dump_flags,
        }
    }

    fn compile(&self) -> Result<(), String> {
        println!("Compiling file: {}/{}", std::env::current_dir().unwrap().to_str().unwrap(), self.input_path);

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
        if self.dump_flags.contains(&DumpKind::Tokens) {
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
        let ast = result.unwrap();
        if self.dump_flags.contains(&DumpKind::Ast) {
            for node in &ast {
                println!("{}", node);
            }
        }

        // Lowering stages
        // Ast -> Hir
        let mut ast_lowerer = lowering_stages::ast_to_hir::AstToHir::new();
        let result = ast_lowerer.lower(ast);
        if result.is_err() {
            return Err(format!("ast -> hir failed on file:\n{}", result.unwrap_err()));
        }
        let hir = result.unwrap();
        if self.dump_flags.contains(&DumpKind::Hir) {
            for node in &hir {
                println!("{}", node);
            }
        }


        // Hir -> Thir
        let mut hir_lowerer = lowering_stages::hir_to_thir::HirLowerer::new(ast_lowerer.type_table);
        let result = hir_lowerer.lower(hir);
        if result.is_err() {
            return Err(format!("hir -> thir failed on file:\n{}", result.unwrap_err()));
        }
        let thir = result.unwrap();
        if self.dump_flags.contains(&DumpKind::Thir) {
            for stmt in &thir {
                println!("{}", stmt.pretty_print(&hir_lowerer.type_table));
            }
        }
        if self.dump_flags.contains(&DumpKind::TypeTable) {
            for (id, ty) in &hir_lowerer.type_table.types {
                println!("type id: {:?} : {:?}", id, ty)
            }
        }


        // Thir -> Mir
        let mut thir_lowerer = lowering_stages::thir_to_mir::ThirLowerer::new(hir_lowerer.type_table);
        let result = thir_lowerer.lower(thir);
        let mir = result;
        if self.dump_flags.contains(&DumpKind::Mir) {
            for node in &mir {
                println!("{}", node);
            }
        }


        // Code Generator
        let mut code_gen = crate::compiler::assembly_generation::assembly_generator::AssemblyGenerator::new();
        let asm = code_gen.generate(mir, &ast_lowerer.main_func.unwrap());
        if self.dump_flags.contains(&DumpKind::Asm) {
            println!();
            for line in asm {
                println!("{}", line);
            }
        }

        fs::write("/tmp/tmp.nesm", asm.join("\n")).unwrap();

        let output = std::process::Command::new("python3")
            .arg("-m")
            .arg("nex.cli")
            .arg("asm")
            .arg("/tmp/tmp.nesm")
            .arg(self.output_path.as_str())
            .stdout(std::process::Stdio::piped())
            .spawn().unwrap();

        let output = output.wait_with_output().unwrap();
        let output = String::from_utf8(output.stdout).unwrap();
        println!("{}", output.trim());

        println!("{}", format!("Compilation done!\nCompiled to: {}", self.output_path).green().bold().to_string());

        Ok(())
    }
}

pub fn compile_file(
    input_path: String,
    output_path: Option<String>,
    dump_flags: Vec<DumpKind>,
) -> Result<(), String> {
    let compiler = Compiler::new(input_path, output_path, dump_flags);

    if let Err(e) = compiler.compile() {
        eprintln!("{}", e.to_string().red());
    }

    Ok(())
}