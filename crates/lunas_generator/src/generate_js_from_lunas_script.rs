use lunas_parser::structs::detailed_language_blocks::JsBlock;

use crate::transformers::{
    js_utils::{find_variable_declarations, search_json},
    utils::add_or_remove_strings_to_script,
};

pub fn generate_js_from_lunas_script_blk(
    js_block: &JsBlock,
    runtime_path: Option<String>,
) -> Result<String, String> {
    // Determine the runtime path, or use default if none provided
    let runtime_path = runtime_path.unwrap_or_else(|| "lunas/dist/runtime".to_string());

    // Prepare lists for variable names and transformation positions
    let mut variables = Vec::new();
    let mut positions = Vec::new();

    // 1) Collect all variable declaration positions
    let (decl_positions, _) = find_variable_declarations(&js_block.ast, 0, &mut variables, true);
    positions.extend(decl_positions);

    // Build a list of variable names from the collected declarations
    let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();

    // 2) Invoke search_json with mutable buffers for results
    let mut search_transforms = Vec::new();
    let mut search_imports = Vec::new();
    let mut dep_vars = Vec::new();
    let mut funcs = Vec::new();

    search_json(
        &js_block.ast,
        &js_block.raw,
        &variable_names,
        &vec![],
        false,
        &mut search_transforms,
        &mut search_imports,
        &mut dep_vars,
        &mut funcs,
    );
    positions.extend(search_transforms);

    // 3) Apply all collected transformations to the raw script
    let (output, tails) = add_or_remove_strings_to_script(positions, &js_block.raw);

    let output = format!(
        "{}{}",
        output,
        if tails.is_empty() {
            "".to_string()
        } else {
            format!("\n{}", tails)
        }
    );

    // 4) Prepend the import statement and assemble the final script
    Ok(format!(
        r#"import {{ $$lunasCreateNonReactive }} from "{}";

{}"#,
        runtime_path, output,
    ))
}
