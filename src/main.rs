pub mod bdg;
pub mod lexer;

// Core compiler structure stubs
pub mod ast;
pub mod cmd;
pub mod ir;
pub mod llvm;
pub mod parser;
pub mod sema;
pub mod std;

fn main() {
    cmd::run();
}
