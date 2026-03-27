//! HEMTT - Arma 3 Preprocessor

pub mod codes {
    automod::dir!(pub "src/codes");
}

pub mod builtin_macro;
pub mod context;
pub mod directive_handler;
pub mod expansion_metadata_store;
pub mod macro_expander;
pub mod processed_with_metadata;
pub mod token_stream;

mod defines;
mod definition;
mod error;
mod ifstate;
pub mod parse;
mod processor;

pub use error::Error;
pub use processor::Processor;
pub use context::{PreprocessingContext, LineContext, MacroExpansionContext, MacroFrame};
pub use token_stream::TokenStream;
pub use directive_handler::{DirectiveHandler, DirectiveResult, DirectiveConfig};
pub use builtin_macro::{BuiltInMacro, BuiltInMacroRegistry, ConstantMacro, CounterMacro, create_default_registry};
pub use macro_expander::{MacroExpander, ExpansionMetadata};
pub use expansion_metadata_store::ExpansionMetadataStore;
pub use processed_with_metadata::ProcessedWithMetadata;
