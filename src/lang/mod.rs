//! ADead-GPU Language Parser (.gpu files)
//!
//! Declarative GPU submission language - migrated from ADead-GPU C++
//!
//! The .gpu language is a clean, deterministic way to describe GPU commands:
//! - No conditionals (if/else)
//! - No loops (for/while)  
//! - No functions
//! - Direct 1:1 mapping to GPU commands

mod lexer;
mod parser;
mod ast;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::{Parser, ParseError};
pub use ast::*;

use thiserror::Error;

/// Language errors
#[derive(Error, Debug)]
pub enum LangError {
    #[error("Lexer error at line {line}: {message}")]
    Lexer { line: usize, message: String },
    #[error("Parser error at line {line}: {message}")]
    Parser { line: usize, message: String },
    #[error("Semantic error: {0}")]
    Semantic(String),
}

pub type LangResult<T> = Result<T, LangError>;

/// Parse a .gpu source file into an AST
pub fn parse_gpu_source(source: &str) -> LangResult<Program> {
    let lexer = Lexer::new(source);
    let tokens: Vec<Token> = lexer.collect();
    
    let mut parser = Parser::new(&tokens);
    parser.parse_program()
}

/// Convenience function to parse and validate
pub fn parse_and_validate(source: &str) -> LangResult<Program> {
    let program = parse_gpu_source(source)?;
    validate_program(&program)?;
    Ok(program)
}

/// Validate a parsed program
pub fn validate_program(program: &Program) -> LangResult<()> {
    // Check that all referenced shaders exist
    let shader_names: Vec<&str> = program.shaders.iter().map(|s| s.name.as_str()).collect();
    
    for pipeline in &program.pipelines {
        if let Some(ref vs) = pipeline.vertex_shader {
            if !shader_names.contains(&vs.as_str()) {
                return Err(LangError::Semantic(format!("Unknown shader: {}", vs)));
            }
        }
        if let Some(ref ps) = pipeline.pixel_shader {
            if !shader_names.contains(&ps.as_str()) {
                return Err(LangError::Semantic(format!("Unknown shader: {}", ps)));
            }
        }
    }
    
    Ok(())
}
