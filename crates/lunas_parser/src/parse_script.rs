use crate::{
    structs::detailed_language_blocks::JsBlock, swc_parser::parse_with_swc,
    ts_to_js::transform_ts_to_js,
};

pub fn parse_lunas_script(input: &str) -> Result<JsBlock, String> {
    let js = transform_ts_to_js(input).map_err(|e| e.to_string())?;
    let parsed = parse_with_swc(&js);
    let parsed_json = serde_json::to_value(&parsed).map_err(|e| e.to_string())?;
    Ok(JsBlock {
        ast: parsed_json,
        raw: js.trim().into(),
    })
}
