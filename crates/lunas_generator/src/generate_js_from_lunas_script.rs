use lunas_parser::structs::detailed_language_blocks::JsBlock;

use crate::{
    structs::js_utils::JsSearchParent,
    transformers::{
        js_utils::{find_variable_declarations, search_json},
        utils::add_or_remove_strings_to_script,
    },
};

pub fn generate_js_from_lunas_script_blk(
    js_block: &JsBlock,
    runtime_path: Option<String>,
) -> Result<String, String> {
    let runtime_path = match runtime_path.is_none() {
        true => "lunas/dist/runtime".to_string(),
        false => runtime_path.unwrap(),
    };

    let mut variables = vec![];

    let mut positions = vec![];
    let imports = vec![];
    // find all variable declarations
    let str_positions = find_variable_declarations(&js_block.ast, 0, &mut variables, true);
    // add all variable declarations to positions to add custom variable declaration function
    positions.extend(str_positions);
    let variable_names = variables.iter().map(|v| v.name.clone()).collect();
    let (position_result, _, _, _) = search_json(
        &js_block.ast,
        &js_block.raw,
        &variable_names,
        Some(&imports),
        JsSearchParent::NoneValue,
        false,
    );

    positions.extend(position_result);
    let output = add_or_remove_strings_to_script(positions, &js_block.raw);

    Ok(format!(
        r#"import {{ $$lunasCreateNonReactive }} from "{}";

{}
"#,
        runtime_path, output,
    ))
}
