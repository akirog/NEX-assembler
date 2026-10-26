extern crate colored;
pub mod compiler;
mod config_loader;

extern crate clap;
use clap::{Args, Parser, Subcommand};
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().bold())
    .valid(AnsiColor::Green.on_default())
    .invalid(AnsiColor::Yellow.on_default());

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, styles = STYLES)]
struct Cli {
    #[clap(short, long)]
    elapsed: bool,

    #[command(subcommand)]
    command: Commands,
}



#[derive(Args, Debug)]
struct AssembleArgs {
    input_path: String,
    output_path: Option<String>,
}


#[derive(Args, Debug)]
struct CompileArgs {
    input_path: String,
    output_path: Option<String>,

    /// Dump flags for the compiler, each character enables a different info dump
    /// t: tokens
    /// y: type table
    /// s: scopes
    /// a: ast
    #[arg(long, default_value = "", verbatim_doc_comment)]
    dump_flags: String,
}

#[derive(Args, Debug)]
struct BuildArgs;


#[derive(Args, Debug)]
struct RunArgs;



#[derive(Subcommand, Debug)]
enum Commands {
    Assemble(AssembleArgs),
    Compile(CompileArgs),
    Build(BuildArgs),
    Run(RunArgs),
}


fn main() -> std::process::ExitCode {
    let args = Cli::parse();

    let start_time = std::time::Instant::now();

    let result = match args.command {
        Commands::Assemble(_args) => { Err("Assembler not Implemented".to_string()) },
        Commands::Compile(args) => { compile(&args) },
        Commands::Build(_args) => { Err("Build not Implemented".to_string()) },
        Commands::Run(_args) => { Err("Run not Implemented".to_string()) },
    };

    if args.elapsed {
        println!("Execution time: {:.3}s", start_time.elapsed().as_secs_f64());
    }

    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}




fn compile(args: &CompileArgs) -> Result<(), String> {
    compiler::pipeline::compile_file(args.input_path.clone(), args.output_path.clone(), args.dump_flags.clone())
}