extern crate clap;
extern crate colored;
pub mod compiler;
mod config_loader;
mod emulator;

use clap::builder::styling::AnsiColor;
use clap::builder::Styles;
use clap::{Args, Parser, Subcommand, ValueEnum};
use crossterm::style::Stylize;

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

    #[arg(long, value_enum, value_delimiter = ',')]
    dump: Vec<DumpKind>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum DumpKind {
    Tokens,
    Ast,
    Hir,
    Thir,
    TypeTable,
    ResolvedThir,
    Mir,
    Asm,
    All,
}


#[derive(Args, Debug)]
struct BuildArgs;


#[derive(Args, Debug)]
struct RunArgs {
    ram_path: Option<String>,
    rom_path: Option<String>,
}



#[derive(Subcommand, Debug)]
enum Commands {
    Assemble(AssembleArgs),
    Compile(CompileArgs),
    Build(BuildArgs),
    Emu(RunArgs),
}


fn main() -> std::process::ExitCode {

    let args = Cli::parse();

    let start_time = std::time::Instant::now();

    let result = match args.command {
        Commands::Assemble(_args) => { Err("Assembler not Implemented".to_string()) },
        Commands::Compile(args) => { compile(&args) },
        Commands::Build(_args) => { Err("Build not Implemented".to_string()) },
        Commands::Emu(args) => { emulate(&args) },
    };

    if args.elapsed {
        println!("{}", format!("Execution time: {:.3}s", start_time.elapsed().as_secs_f64()).yellow().bold());
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
    compiler::pipeline::compile_file(args.input_path.clone(), args.output_path.clone(), args.dump.clone())
}

fn emulate(args: &RunArgs) -> Result<(), String> {
    emulator::run_emulator(args.ram_path.clone(), args.rom_path.clone())
}