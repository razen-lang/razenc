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
    let token_result = lexer.tokenize();

    if !token_result.errors.is_empty() {
        for err in &token_result.errors {
            crate::bdg::print_error(&format!(
                "[Lexer] {} at line {}, col {}",
                err.message, err.line, err.col
            ));
        }
    }

    if verbose {
        crate::bdg::print_token_count(token_result.tokens.len());
        crate::bdg::print_tokens(&token_result.tokens);
        crate::bdg::print_footer("Lexer\t\tDone");
    }

    let mut parser = crate::parser::Parser::new_with_source(token_result.tokens, &source);
    let program = match parser.parse() {
        Ok(program) => {
            if verbose {
                crate::bdg::print_phase2_header("Done");
                crate::bdg::print_ast(&program);
                crate::bdg::print_footer("Parser\t\tDone");
            }
            Some(program)
        }
        Err(errors) => {
            for err in &errors {
                crate::bdg::print_parse_error(&source, file_stem, err);
            }
            None
        }
    };

    if let Some(ref program) = program {
        if verbose {
            crate::bdg::print_phase3_header("Running");
        }

        let mut sema = crate::sema::SemanticAnalyzer::new();
        match sema.analyze(program) {
            Ok(_) => {
                if verbose {
                    crate::bdg::print_footer("Semantic Analysis\tDone");
                }
            }
            Err(errors) => {
                for err in &errors {
                    crate::bdg::print_error(&format!("[{}] {}", err.code, err.message));
                }
                std::process::exit(1);
            }
        }

        // Phase 4: IR Generation
        if verbose {
            crate::bdg::print_phase4_header("Running");
        }

        let mut ir_gen = crate::ir::IrGenerator::new();
        let ir_program = ir_gen.generate(program);

        if verbose {
            crate::bdg::print_ir(&ir_program);
            crate::bdg::print_footer("IR Generation\t\tDone");
        }
    } else {
        std::process::exit(1);
    }
}
