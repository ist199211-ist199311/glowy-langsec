use crate::{ast::SourceFileNode, errors::ParsingError, TokenStream};

// awkwardly named to prevent confusion with
// [`crate::parse`], this crate's public interface
pub fn build_ast_from_tokens(s: TokenStream) -> Result<SourceFileNode, ParsingError> {
    todo!()
}
