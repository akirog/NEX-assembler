use crate::DumpKind;
use crate::compiler::lexer::Lexer;
use crate::compiler::lowering_stages;
use crate::compiler::parser::Parser;
use crossterm::style::Stylize;
use std::fs;
use crate::compiler::errors::ErrorPrinter;

struct Compiler {
    input_path: std::path::PathBuf,
    output_path: std::path::PathBuf,
    dump_flags: Vec<DumpKind>,
}

impl Compiler {
    fn new(input_path: String, output_path: Option<String>, dump_flags: Vec<DumpKind>) -> Compiler {
        let output_path = output_path.unwrap_or_else(|| {
            let path = std::path::Path::new(&input_path);
            let filename = path.file_name();

            let Some(config) = crate::config_loader::load_config() else { panic!("Failed to load config file, nex.toml not found.") };

            config.paths.build.join(filename.unwrap()).with_extension("bin").to_str().unwrap().to_string()
        });

        let Some(project_root) = crate::config_loader::find_project_root() else { panic!("Failed to find project root.") };

        let input_path = project_root.join(input_path);
        let output_path = project_root.join(output_path);
        
        Compiler {
            input_path,
            output_path,
            dump_flags,
        }
    }

    fn compile(&self) -> Result<(), String> {
        println!("Compiling file: {}", self.input_path.display());

        // Preprocessor
        let mut preprocessor = crate::compiler::preprocessor::PreProcessor {};
        let mut error_printer = ErrorPrinter::new();
        let result = preprocessor.process(&mut error_printer, &std::path::Path::new(&self.input_path));
        if let Err(err) = result {
            return Err(
                error_printer.print_error(err)
            )
        };

        // Lexer file
        let mut lexer = Lexer::new();
        let result = lexer.tokenize(&error_printer.get_text());
        let tokens = match result {
            Err(err) => {
                return Err(format!(
                    "Unable to tokenize file: \n{}",
                    error_printer.print_error(err)
                ));
            }
            Ok(tokens) => tokens,
        };

        if self.dump_flags.contains(&DumpKind::Tokens) {
            for token in &tokens {
                println!("{}", token);
            }
        }

        // Parser
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Err(err) => {
                return Err(format!("Unable to parse file:\n{}", error_printer.print_error(err)));
            }
            Ok(ast) => ast,
        };

        if self.dump_flags.contains(&DumpKind::Ast) {
            for node in &ast {
                println!("{}", node);
            }
        }

        // Lowering stages
        // Ast -> Hir
        let mut ast_lowerer = lowering_stages::ast_to_hir::AstToHir::new();
        let hir = match ast_lowerer.lower(ast) {
            Err(err) => {
                return Err(format!("ast -> hir failed on file:\n{}", error_printer.print_error(err)))
            }
            Ok(hir) => hir,
        };

        if self.dump_flags.contains(&DumpKind::Hir) {
            for node in &hir {
                println!("{}", node);
            }
        }


        // Hir -> Thir
        let mut hir_lowerer = lowering_stages::hir_to_thir::HirLowerer::new(ast_lowerer.type_table);
        let mut thir = match hir_lowerer.lower(hir) {
            Err(err) => {
                return Err(format!("hir -> thir failed on file:\n{}", error_printer.print_error(err)))
            }
            Ok(thir) => thir,
        };
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


        // Type resolver for unresolved types
        let mut type_resolver = lowering_stages::type_resolver::TypeResolver::new(&mut hir_lowerer.type_table);
        match type_resolver.resolve(thir.as_mut_slice()) {
            Err(err) => {
                return Err(format!("Type resolution failed on file:\n{}", error_printer.print_error(err)))
            }
            Ok(_) => (),
        };
        if self.dump_flags.contains(&DumpKind::ResolvedThir) {
            for stmt in &thir {
                println!("{}", stmt.pretty_print(&type_resolver.type_table));
            }
        }



        // Thir -> Mir
        let mut thir_lowerer = lowering_stages::thir_to_mir::ThirLowerer::new(&mut hir_lowerer.type_table);
        let mir = thir_lowerer.lower(thir);
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
            .arg(self.output_path.to_string_lossy().to_string())
            .stdout(std::process::Stdio::piped())
            .spawn().unwrap();

        let output = output.wait_with_output().unwrap();
        let output = String::from_utf8(output.stdout).unwrap();
        println!("{}", output.trim());

        println!("{}", format!("Compilation done!\nCompiled to: {}", self.output_path.to_string_lossy().to_string()).green().bold().to_string());

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