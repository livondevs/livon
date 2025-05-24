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

pub fn parse_module_with_swc(
    code: &String,
) -> Result<swc_ecma_ast::Module, Box<dyn std::error::Error>> {
    let cm: Lrc<swc_common::SourceMap> = Default::default();
    let handler = swc_common::errors::Handler::with_tty_emitter(
        swc_common::errors::ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    );
    let fm = cm.new_source_file(Lrc::new(swc_common::FileName::Anon), code.into());
    let lexer = swc_ecma_parser::lexer::Lexer::new(
        swc_ecma_parser::Syntax::Es(Default::default()),
        Default::default(),
        swc_ecma_parser::StringInput::from(&*fm),
        None,
    );
    let mut parser = swc_ecma_parser::Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let module = parser.parse_module().map_err(|e| {
        e.clone().into_diagnostic(&handler).emit();
        Box::<dyn std::error::Error>::from(format!("Failed to parse module: {}", &e.kind().msg()))
    })?;
    Ok(module)
}

pub fn parse_expr_with_swc(
    code: &String,
) -> Result<Box<swc_ecma_ast::Expr>, Box<dyn std::error::Error>> {
    let cm: Lrc<swc_common::SourceMap> = Default::default();
    let handler = swc_common::errors::Handler::with_tty_emitter(
        swc_common::errors::ColorConfig::Auto,
        true,
        false,
        Some(cm.clone()),
    );
    let fm = cm.new_source_file(Lrc::new(swc_common::FileName::Anon), code.into());
    let lexer = swc_ecma_parser::lexer::Lexer::new(
        swc_ecma_parser::Syntax::Es(Default::default()),
        Default::default(),
        swc_ecma_parser::StringInput::from(&*fm),
        None,
    );
    let mut parser = swc_ecma_parser::Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let expr = parser.parse_expr().map_err(|e| {
        e.clone().into_diagnostic(&handler).emit();
        Box::<dyn std::error::Error>::from(format!(
            "Failed to parse expression: {}",
            &e.kind().msg()
        ))
    })?;
    Ok(expr)
}

// NOTE: This function temporarily wraps the input in `export default` so that standalone expressions can be parsed by the TS parser; remove this workaround once direct expr parsing is supported.
pub fn transform_ts_to_js(ts_code: &str) -> Result<String, Box<dyn Error>> {
    // 1) Wrap the input in `export default` so we can parse a standalone expression
    let wrapped_code = format!("export default {};", ts_code);

    // Create a shared SourceMap instance (Lrc is an Arc alias)
    let cm: Lrc<SourceMap> = Default::default();

    // Set up a handler for emitting errors with colored output in TTY environments
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // Create a virtual source file for parsing (filename is arbitrary)
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.ts".into())),
        wrapped_code.into(), // use wrapped_code, not the raw ts_code
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

    // Emit any lexer errors via the handler
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    // TODO: Add line numbers for error positions
    // Parse the source into an AST program/module
    let module = parser.parse_program().map_err(|e| {
        e.clone().into_diagnostic(&handler).emit();
        Box::<dyn std::error::Error>::from(format!(
            "Failed to parse TypeScript code: {}",
            &e.kind().msg()
        ))
    })?;

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
                verbatim_module_syntax: true,
                no_empty_export: true,
                import_not_used_as_values: typescript::ImportsNotUsedAsValues::Preserve,
                ..Config::default()
            },
            unresolved_mark,
            top_level_mark,
        ));

        // 3. hygiene: rename identifiers to avoid collisions
        let module = module.apply(hygiene());

        // 4. fixer: insert missing tokens (e.g., semicolons, parentheses)
        let program = module.apply(fixer(Some(&comments)));

        // Generate JavaScript code from the transformed AST
        to_code_default(cm, Some(&comments), &program)
    });

    // 2) Unwrap the generated code by removing the `export default ` prefix and trailing semicolon
    let trimmed = code
        .trim_start_matches("export default ")
        .trim_end_matches('\n')
        .trim_end_matches(';')
        .to_string();

    Ok(trimmed)
}
