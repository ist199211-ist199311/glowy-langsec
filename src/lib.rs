use crate::{context::AnalysisContext, taint::visit_source_file};

mod context;
mod errors;
mod labels;
mod symbols;
mod taint;

// files is an iterator of file name and file content
pub fn analyze_files<'a>(
    files: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> AnalysisContext<'a> {
    let mut context = AnalysisContext::new();

    let parsed = files
        .into_iter()
        .map(|(file_name, contents)| parser::parse(contents).map(|result| (file_name, result)))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let mut changed = true;
    while changed {
        changed = false;
        for (file_name, node) in &parsed {
            changed |= visit_source_file(&mut context, file_name, node);
        }
    }

    context
}
