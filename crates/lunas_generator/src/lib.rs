mod ast_analyzer;
mod consts;
mod generate_js;
mod generate_js_from_lunas_script;
mod generate_statements;
mod js_utils;
mod orig_html_struct;
mod structs;
mod transformers;
use generate_js::generate_js_from_blocks;
use generate_js_from_lunas_script::generate_js_from_lunas_script_blk;
use lunas_parser::{structs::detailed_language_blocks::JsBlock, DetailedBlock};
#[macro_use]
extern crate lazy_static;

pub fn lunas_compile_from_block(
    b: &DetailedBlock,
    runtime_path: Option<String>,
) -> Result<(String, Option<String>), String> {
    let compiled_code = generate_js_from_blocks(b, runtime_path);
    compiled_code
}

pub fn lunas_script_compile_from_block(
    b: &JsBlock,
    runtime_path: Option<String>,
) -> Result<String, String> {
    let compiled_code = generate_js_from_lunas_script_blk(b, runtime_path);
    compiled_code
}
