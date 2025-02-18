//! The Tx3 language
//!
//! This crate provides the parser, analyzer and lowering logic for the Tx3
//! language.
//!
//! # Parsing
//!
//! ```
//! let program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! ```
//!
//! # Analyzing
//!
//! ```
//! let mut program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! tx3_lang::analyze(&mut program).unwrap();
//! ```
//!
//! # Lowering
//!
//! ```
//! let mut program = tx3_lang::parse_string("tx swap() {}").unwrap();
//! tx3_lang::analyze(&mut program).unwrap();
//! let ir = tx3_lang::lower(&program).unwrap();
//! ```

pub mod ast;
pub mod ir;

mod analyzing;
mod lowering;
mod parsing;

pub use analyzing::{analyze, Error as AnalyzeError};
pub use lowering::{lower, Error as LowerError};

pub use parsing::Error as ParseError;

/// Parses a Tx3 source string into a Program AST.
///
/// # Arguments
///
/// * `input` - String containing Tx3 source code
///
/// # Returns
///
/// * `Result<Program, Error>` - The parsed Program AST or an error
///
/// # Errors
///
/// Returns an error if:
/// - The input string is not valid Tx3 syntax
/// - The AST construction fails
///
/// # Example
///
/// ```
/// use tx3_lang::parse_string;
///
/// let program = parse_string("tx swap() {}").unwrap();
/// ```
pub use parsing::parse_string;

/// Parses a Tx3 source file into a Program AST.
///
/// # Arguments
///
/// * `path` - Path to the Tx3 source file to parse
///
/// # Returns
///
/// * `Result<Program, Error>` - The parsed Program AST or an error
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file contents are not valid Tx3 syntax
/// - The AST construction fails
///
/// # Example
///
/// ```no_run
/// use tx3_lang::parse_file;
///
/// let program = parse_file("path/to/program.tx3").unwrap();
/// ```
pub use parsing::parse_file;

/// Helper to load Tx3 code from a string into a lazy static variable.
///
/// This macro will parse the Tx3 code, analyze it and lower it into an
/// intermediate representation. The resulting program will be stored in a
/// lazy static variable, so it will be initialized only once and will be
/// thread-safe.
///
/// # Example
///
/// ```
/// use tx3_lang::load_string;
///
/// load_string!(VESTING, "tx vesting() {}");
/// ```
#[macro_export]
macro_rules! load_string {
    ($name:ident,$code:expr) => {
        const $name: std::sync::LazyLock<$crate::ir::Program> = std::sync::LazyLock::new(|| {
            let mut program = tx3_lang::parse_string(&$code).unwrap();
            tx3_lang::analyze(&mut program).unwrap();
            tx3_lang::lower(&mut program).unwrap()
        });
    };
}

/// Helper to load Tx3 code from a file into a lazy static variable.
///
/// This macro will parse the Tx3 code, analyze it and lower it into an
/// intermediate representation. The resulting program will be stored in a
/// lazy static variable, so it will be initialized only once and will be
/// thread-safe.
///
/// # Example
///
/// ```no_run
/// use tx3_lang::load_file;
///
/// load_file!(VESTING, "path/to/vesting.tx3");
/// ```
#[macro_export]
macro_rules! load_file {
    ($name:ident,$path:expr) => {
        const $name: std::sync::LazyLock<$crate::ir::Program> = std::sync::LazyLock::new(|| {
            let mut program = tx3_lang::parse_file(&$path).unwrap();
            tx3_lang::analyze(&mut program).unwrap();
            tx3_lang::lower(&mut program).unwrap()
        });
    };
}
