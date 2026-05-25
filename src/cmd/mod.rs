use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "razenc",
    version = "0.1.0",
    about = "The Razen Programming Language Compiler"
)]
pub struct Args {
    /// Print parsed tokens, stage status, and source layout
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,

    /// Input file(s) to process (.rzn)
    #[arg(short = 'f', long = "files", required = true, num_args = 1..)]
    pub files: Vec<PathBuf>,
}

pub fn run() {
    let args = Args::parse();

    for file_path in &args.files {
        process_file(file_path, args.verbose);
    }
}

fn process_file(file_path: &PathBuf, verbose: bool) {
    let source = match std::fs::read_to_string(file_path) {
        Ok(src) => src,
        Err(e) => {
            crate::bdg::print_error(&format!(
                "Failed to read source file '{}': {}",
                file_path.display(),
                e
            ));
            std::process::exit(1);
        }
    };

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("source");

    if verbose {
        crate::bdg::print_source(&source, file_stem);
        crate::bdg::print_phase_header("Parsing", "Done");
    }

    let lexer = crate::lexer::Lexer::new(&source);
    let tokens = lexer.tokenize();

    if verbose {
        crate::bdg::print_token_count(tokens.len());
        crate::bdg::print_tokens(&tokens);
        crate::bdg::print_footer("Lexer\t\tDone");
    }
}
