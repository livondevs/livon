mod ast_analyzer;
mod consts;
mod generate_js;
mod generate_statements;
mod js_utils;
mod orig_html_struct;
mod structs;
mod transformers;
use generate_js::generate_js_from_blocks;
use livon_parser::DetailedBlock;
#[macro_use]
extern crate lazy_static;

pub fn livon_compile_from_block(
    b: &DetailedBlock,
    runtime_path: Option<String>,
) -> Result<(String, Option<String>), String> {
    let compiled_code = generate_js_from_blocks(b, runtime_path);
    compiled_code
}
