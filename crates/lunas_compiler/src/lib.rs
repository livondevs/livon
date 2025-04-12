use livon_generator::livon_compile_from_block;
use livon_parser::parse_livon_file;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug)]
pub struct LunasCompilerOutput {
    js: String,
    css: Option<String>,
}

#[wasm_bindgen]
impl LunasCompilerOutput {
    #[wasm_bindgen(getter)]
    pub fn js(&self) -> String {
        self.js.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn css(&self) -> Option<String> {
        match self.css {
            Some(ref s) => Some(s.clone()),
            None => None,
        }
    }
}

#[wasm_bindgen]
pub fn compile(
    livon_code: String,
    runtime_path: Option<String>,
) -> Result<LunasCompilerOutput, String> {
    let blocks = match parse_livon_file(&livon_code) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }?;
    let code = livon_compile_from_block(&blocks, runtime_path)?;
    Ok(LunasCompilerOutput {
        js: code.0,
        css: code.1,
    })
}
