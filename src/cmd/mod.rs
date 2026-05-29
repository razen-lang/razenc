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

    /// Run only a specific phase: 1=Lexer, 2=Parser, 3=Semantic, 4=IR
    #[arg(short = 'p', long = "phase")]
    pub phase: Option<u8>,

    /// Capture all output to a .log file (auto-named from input file)
    #[arg(long = "log")]
    pub log: bool,
}

pub fn run() {
    let args = Args::parse();

    for file_path in &args.files {
        if args.log {
            let log_path = build_log_path(file_path);
            let output = run_to_string(file_path, args.verbose, args.phase);
            match std::fs::write(&log_path, &output) {
                Ok(_) => {
                    eprintln!("Log written to: {}", log_path.display());
                }
                Err(e) => {
                    eprintln!("Failed to write log file: {}", e);
                    // Fallback: print to stdout
                    print!("{}", output);
                }
            }
        } else {
            process_file(file_path, args.verbose, args.phase);
        }
    }
}

fn build_log_path(file_path: &std::path::Path) -> PathBuf {
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let parent = file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    parent.join(format!("{}.log", stem))
}

fn run_to_string(file_path: &PathBuf, verbose: bool, phase: Option<u8>) -> String {
    let mut buf = String::new();

    let source = match std::fs::read_to_string(file_path) {
        Ok(src) => src,
        Err(e) => {
            buf.push_str(&format!(
                "Error: Failed to read source file '{}': {}\n",
                file_path.display(),
                e
            ));
            return buf;
        }
    };

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("source");

    // Only show source dump + lexer output when phase 1 is the target (or no phase specified)
    let show_lexer = phase != Some(2) && phase != Some(3) && phase != Some(4);
    if verbose && show_lexer {
        buf.push_str(&format_source(&source, file_stem));
        buf.push_str("Phase 1  Parsing\t\tDone\n\n");
    }

    // Phase 1: Lexer
    let lexer = crate::lexer::Lexer::new(&source);
    let token_result = lexer.tokenize();

    if !token_result.errors.is_empty() {
        for err in &token_result.errors {
            buf.push_str(&format!(
                "Error: [Lexer] {} at line {}, col {}\n",
                err.message, err.line, err.col
            ));
        }
    }

    if verbose && show_lexer {
        buf.push_str(&format!("\nTokens ({})\n", token_result.tokens.len()));
        buf.push_str(&format_tokens(&token_result.tokens));
        buf.push_str("\n\t\tLexer\t\tDone\n");
    }

    // If only phase 1 requested, stop here
    if phase == Some(1) {
        return buf;
    }

    // Phase 2: Parser
    let mut parser = crate::parser::Parser::new_with_source(token_result.tokens, &source);
    let program = match parser.parse() {
        Ok(program) => {
            if verbose {
                buf.push_str("\nPhase 2  AST Build\tDone\n");
                buf.push_str(&format_ast(&program));
                buf.push_str("\n\t\tParser\t\tDone\n");
            }
            Some(program)
        }
        Err(errors) => {
            for err in &errors {
                buf.push_str(&format_parse_error(&source, file_stem, err));
            }
            None
        }
    };

    // If only phase 2 requested, stop here
    if phase == Some(2) || program.is_none() {
        return buf;
    }

    let program = program.unwrap();

    // Phase 3: Semantic Analysis
    if verbose {
        buf.push_str("\nPhase 3  Semantic Analysis\tRunning\n");
    }

    let mut sema = crate::sema::SemanticAnalyzer::new();
    match sema.analyze(&program) {
        Ok(_) => {
            if verbose {
                buf.push_str("\t\tSemantic Analysis\tDone\n");
            }
        }
        Err(errors) => {
            for err in &errors {
                buf.push_str(&format!("Error: [{}] {}\n", err.code, err.message));
            }
            return buf;
        }
    }

    // If only phase 3 requested, stop here
    if phase == Some(3) {
        return buf;
    }

    // Phase 4: IR Generation
    if verbose {
        buf.push_str("\nPhase 4  IR Generation\tRunning\n");
    }

    let mut ir_gen = crate::ir::IrGenerator::new();
    let ir_program = ir_gen.generate(&program);

    if verbose {
        buf.push_str(&format_ir(&ir_program));
        buf.push_str("\n\t\tIR Generation\t\tDone\n");
    }

    buf
}

fn process_file(file_path: &PathBuf, verbose: bool, phase: Option<u8>) {
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

    // Only show source dump + lexer output when phase 1 is the target (or no phase specified)
    let show_lexer = phase != Some(2) && phase != Some(3) && phase != Some(4);
    if verbose && show_lexer {
        crate::bdg::print_source(&source, file_stem);
        crate::bdg::print_phase_header("Parsing", "Done");
    }

    // Phase 1: Lexer
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

    if verbose && show_lexer {
        crate::bdg::print_token_count(token_result.tokens.len());
        crate::bdg::print_tokens(&token_result.tokens);
        crate::bdg::print_footer("Lexer\t\tDone");
    }

    // If only phase 1 requested, stop here
    if phase == Some(1) {
        return;
    }

    // Phase 2: Parser
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

    // If only phase 2 requested, stop here
    if phase == Some(2) || program.is_none() {
        if program.is_none() {
            std::process::exit(1);
        }
        return;
    }

    let program = program.unwrap();

    // Phase 3: Semantic Analysis
    if verbose {
        crate::bdg::print_phase3_header("Running");
    }

    let mut sema = crate::sema::SemanticAnalyzer::new();
    match sema.analyze(&program) {
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

    // If only phase 3 requested, stop here
    if phase == Some(3) {
        return;
    }

    // Phase 4: IR Generation
    if verbose {
        crate::bdg::print_phase4_header("Running");
    }

    let mut ir_gen = crate::ir::IrGenerator::new();
    let ir_program = ir_gen.generate(&program);

    if verbose {
        crate::bdg::print_ir(&ir_program);
        crate::bdg::print_footer("IR Generation\t\tDone");
    }
}

// ---- String-based formatters for --log mode ----

fn format_source(source: &str, label: &str) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("\n━━━ {} ━━━\n", label));
    buf.push_str("Source:\n");
    for line in source.lines() {
        buf.push_str(line);
        buf.push('\n');
    }
    buf.push('\n');
    buf
}

fn format_tokens(tokens: &[crate::lexer::token::Token]) -> String {
    let mut buf = String::new();
    for (i, token) in tokens.iter().enumerate() {
        let kind_str = format!("{}", token.kind);
        let value_str = if token.value.is_empty() {
            "''".to_string()
        } else {
            format!("'{}'", token.value)
        };
        buf.push_str(&format!(
            "  [{:>3}]  Type:{}  Value:{}  Line:{}  Span:{}:{}\n",
            i, kind_str, value_str, token.line, token.span.0, token.span.1
        ));
    }
    buf
}

fn format_ast(program: &crate::ast::Program) -> String {
    let mut buf = String::new();
    buf.push_str("\n━━━ AST Tree ━━━\n");
    format_decls(&program.decls, 0, &mut buf);
    buf.push('\n');
    buf
}

fn format_decls(decls: &[crate::ast::Decl], indent: usize, buf: &mut String) {
    for decl in decls {
        format_decl(decl, indent, buf);
    }
}

fn format_decl(decl: &crate::ast::Decl, indent: usize, buf: &mut String) {
    let i = "  ".repeat(indent);
    match decl {
        crate::ast::Decl::Use(path) => {
            buf.push_str(&format!("{}Use {}\n", i, path.join(".")));
        }
        crate::ast::Decl::Fn(f) => {
            buf.push_str(&format!("{}Fn {}\n", i, f.name));
        }
        crate::ast::Decl::Struct(s) => {
            buf.push_str(&format!("{}Struct {}\n", i, s.name));
        }
        crate::ast::Decl::Enum(e) => {
            buf.push_str(&format!("{}Enum {}\n", i, e.name));
        }
        crate::ast::Decl::Union(u) => {
            buf.push_str(&format!("{}Union {}\n", i, u.name));
        }
        crate::ast::Decl::Error_(name, _) => {
            buf.push_str(&format!("{}Error {}\n", i, name));
        }
        crate::ast::Decl::Behave(b) => {
            buf.push_str(&format!("{}Behave {}\n", i, b.name));
        }
        crate::ast::Decl::Var(v) => {
            buf.push_str(&format!("{}Var {}\n", i, v.name));
        }
        crate::ast::Decl::Const(c) => {
            buf.push_str(&format!("{}Const {}\n", i, c.name));
        }
        crate::ast::Decl::TypeAlias(name, _) => {
            buf.push_str(&format!("{}TypeAlias {}\n", i, name));
        }
        crate::ast::Decl::Test(name, _) => {
            buf.push_str(&format!("{}Test {}\n", i, name));
        }
        crate::ast::Decl::Mod(name, _) => {
            buf.push_str(&format!("{}Mod {}\n", i, name));
        }
    }
}

fn format_ir(ir_program: &crate::ir::IrProgram) -> String {
    let mut buf = String::new();
    buf.push_str("\n━━━ Generated IR ━━━\n");

    for func in &ir_program.functions {
        buf.push_str(&format!("\nFunction: {}\n", func.name));
        if !func.params.is_empty() {
            let params: Vec<String> = func
                .params
                .iter()
                .map(|(n, t)| format!("{}: {:?}", n, t))
                .collect();
            buf.push_str(&format!("  params: {}\n", params.join(", ")));
        }
        if let Some(ref rt) = func.return_type {
            buf.push_str(&format!("  return: {:?}\n", rt));
        }
        buf.push('\n');

        for inst in &func.insts {
            buf.push_str(&format!("  {}\n", inst));
        }
    }

    if !ir_program.globals.is_empty() {
        buf.push_str("\nGlobals:\n");
        for g in &ir_program.globals {
            let init_str = g
                .init
                .as_ref()
                .map(|v| format!(" = {}", v))
                .unwrap_or_default();
            buf.push_str(&format!("  {}{}\n", g.name, init_str));
        }
    }

    buf.push('\n');
    buf
}

fn format_tokens_colored(tokens: &[crate::lexer::token::Token]) -> String {
    // For log mode, use plain text (no ANSI)
    format_tokens(tokens)
}

fn format_parse_error(source: &str, file_stem: &str, err: &crate::parser::ParseError) -> String {
    let mut buf = String::new();
    buf.push_str(&format!("Error: {}\n", err.message));
    if err.line > 0 {
        buf.push_str(&format!("  --> {}:{}:{}\n", file_stem, err.line, err.col));
        if let Some(source_line) = source.lines().nth(err.line - 1) {
            buf.push_str("  |\n");
            buf.push_str(&format!("  | {}\n", source_line));
            let mut caret = String::with_capacity(err.col.saturating_sub(1) + 5);
            caret.push_str(&" ".repeat(4));
            caret.push(' ');
            caret.push_str(&" ".repeat(err.col.saturating_sub(1)));
            caret.push('^');
            buf.push_str(&format!("  {}\n", caret));
        }
    }
    buf
}
