use errors::AnalysisError;

use crate::{context::AnalysisContext, taint::visit_source_file};

mod context;
pub mod errors;
pub mod labels;
mod symbols;
mod taint;

// files is an iterator of file id and file content
pub fn analyze_files<'a>(
    files: impl IntoIterator<Item = (usize, &'a str)>,
) -> Result<(), Vec<AnalysisError<'a>>> {
    let mut context = AnalysisContext::new();

    let mut errors = vec![];
    let mut parsed = vec![];
    for (file_id, contents) in files {
        match parser::parse(contents) {
            Ok(ast) => parsed.push((file_id, ast)),
            Err(error) => errors.push(AnalysisError::Parsing {
                file: file_id,
                error,
            }),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (file_id, node) in &parsed {
            changed |= visit_source_file(&mut context, *file_id, node);
        }
    }

    if context.errors.is_empty() {
        Ok(())
    } else {
        Err(context.errors.into_values().collect::<Vec<_>>())
    }
}
