use std::error::Error;
use swc_common::{
    comments::SingleThreadedComments,
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, Globals, Mark, SourceMap, GLOBALS,
};
use swc_ecma_codegen::to_code_default;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::{typescript, Config};

/// Transforms a TypeScript code string into JavaScript by stripping type annotations and TS-specific syntax.
///
/// # Arguments
///
/// * `ts_code` - A string slice containing the TypeScript source code.
///
/// # Returns
///
/// * `Ok(String)` containing the transformed JavaScript code, or an error if parsing fails.
pub fn transform_ts_to_js(ts_code: &str) -> Result<String, Box<dyn Error>> {
    // Create a shared SourceMap instance (Lrc is an Arc alias)
    let cm: Lrc<SourceMap> = Default::default();

    // Set up an error handler with colored output for TTY environments
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // Create a virtual source file for the parser (filename can be arbitrary)
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.ts".into())),
        ts_code.into(),
    );

    // Initialize a comments manager to preserve and re-emit comments
    let comments = SingleThreadedComments::default();

    // Configure the lexer for TypeScript syntax (set `tsx: true` if processing TSX files)
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false,
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );

    // Create a parser from the lexer
    let mut parser = Parser::new_from(lexer);

    // Emit any lexer errors through the handler
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    // Parse the source into an AST program/module
    let module = parser
        .parse_program()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("Failed to parse TypeScript module.");

    // Execute transformations within a global JS context
    let globals = Globals::default();
    let code = GLOBALS.set(&globals, || {
        // Create unique marks for symbol resolution and hygiene
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        // 1. resolver: perform scope analysis and resolve imports/exports
        let module = module.apply(resolver(unresolved_mark, top_level_mark, true));

        // 2. typescript: strip TypeScript-specific syntax (type annotations, interfaces, etc.)
        let module = module.apply(typescript(
            Config {
                no_empty_export: true,
                import_not_used_as_values: typescript::ImportsNotUsedAsValues::Preserve,
                ..Config::default()
            },
            unresolved_mark,
            top_level_mark,
        ));

        // 3. hygiene: rename identifiers to avoid name collisions
        let module = module.apply(hygiene());

        // 4. fixer: insert missing tokens (e.g., semicolons, parentheses)
        let program = module.apply(fixer(Some(&comments)));

        // Generate JavaScript code from the transformed AST
        to_code_default(cm, Some(&comments), &program)
    });

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform() {
        let ts = r#"
            import axios from 'axios';
            interface Args {
                name: string;
            }
            function greet(arg: Args): void {
                console.log(`Hello, ${arg.name}!`);
            }
        "#;

        // Perform the TS -> JS transformation
        let js = transform_ts_to_js(ts).expect("Transformation failed");

        // Verify that type annotations are removed and imports remain
        assert!(
            js.contains("function greet("),
            "Expected function signature in output"
        );
        assert!(
            !js.contains("string"),
            "Type annotations should be stripped"
        );
        assert!(
            js.contains("import axios from 'axios';"),
            "Import statement should be preserved"
        );
    }
}
