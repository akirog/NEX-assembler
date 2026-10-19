pub mod compiler;

extern crate clap;
use clap::{Args, Parser, Subcommand};



#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
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


fn main() -> Result<(), String> {
    let args = Cli::parse();

    let start_time = std::time::Instant::now();

    let result = match args.command {
        Commands::Assemble(_args) => { Err("Assembler not Implemented".to_string()) },
        Commands::Compile(args) => { compile(&args) },
        Commands::Build(_args) => { Err("Build not Implemented".to_string()) },
        Commands::Run(_args) => { Err("Run not Implemented".to_string()) },
    };

    let duration = start_time.elapsed();
    if args.elapsed { println!("Execution time: {:?}", duration.as_secs_f64()); }


    result
}




fn compile(args: &CompileArgs) -> Result<(), String> {
    compiler::compiler::compile_file(args.input_path.clone(), args.output_path.clone())
}