use lunas_generator::{lunas_compile_from_block, lunas_script_compile_from_block};
use lunas_parser::{parse_lunas_file, parse_lunas_script_file};
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
    lunas_code: String,
    engine_path: Option<String>,
) -> Result<LunasCompilerOutput, String> {
    let blocks = match parse_lunas_file(&lunas_code) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }?;
    let code = lunas_compile_from_block(&blocks, engine_path)?;
    Ok(LunasCompilerOutput {
        js: code.0,
        css: code.1,
    })
}

#[wasm_bindgen]
pub fn compile_lunas_script(
    lunas_code: String,
    engine_path: Option<String>,
) -> Result<String, String> {
    let js_block = match parse_lunas_script_file(&lunas_code) {
        Ok(r) => Ok(r),
        Err(e) => Err(e.to_string()),
    }?;
    let code = lunas_script_compile_from_block(&js_block, engine_path)?;
    Ok(code)
}
