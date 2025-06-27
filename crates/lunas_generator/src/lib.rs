mod ast_analyzer;
mod consts;
mod generate_js;
mod generate_statements;
mod js_utils;
mod orig_html_struct;
mod structs;
mod transformers;
mod utils;
use generate_js::generate_js_from_blocks;
use lunas_parser::DetailedBlock;
use utils::rand_id::RAND_ID_GENERATOR;
extern crate lazy_static;

pub fn lunas_compile_from_block(
    b: &DetailedBlock,
    engine_path: Option<String>,
) -> Result<(String, Option<String>), String> {
    let compiled_code = generate_js_from_blocks(b, engine_path);
    RAND_ID_GENERATOR.lock().unwrap().reset();
    compiled_code
}
