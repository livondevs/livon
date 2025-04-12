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
use swc_ecma_transforms_typescript::strip;

/// TypeScript のコード文字列を受け取り、型注釈等を除去した JavaScript のコード文字列を返す
pub fn transform_ts_to_js(ts_code: &str) -> Result<String, Box<dyn Error>> {
    // ソースマップ用インスタンスの生成（Lrc は Arc のエイリアス）
    let cm: Lrc<SourceMap> = Default::default();

    // エラーハンドラの生成（TTY 向けエミッタ）
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // 入力ソースファイルを作成（ファイル名は任意）
    let fm = cm.new_source_file(
        Lrc::new(FileName::Custom("input.ts".into())),
        ts_code.into(),
    );

    // コメント管理用インスタンスの生成
    let comments = SingleThreadedComments::default();

    // Lexer の生成。Syntax::Typescript により TypeScript としてパースする
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: false, // TSX を利用する場合は true に変更
            ..Default::default()
        }),
        Default::default(),
        StringInput::from(&*fm),
        Some(&comments),
    );

    // Lexer から Parser を生成
    let mut parser = Parser::new_from(lexer);

    // パーサーエラーがあれば出力
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    // プログラム（AST）をパース
    let module = parser
        .parse_program()
        .map_err(|e| e.into_diagnostic(&handler).emit())
        .expect("failed to parse module.");

    // グローバルコンテキスト内で変換処理を実行
    let globals = Globals::default();
    let code = GLOBALS.set(&globals, || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        // 1. resolver: 識別子のスコープ解析を実施
        let module = module.apply(resolver(unresolved_mark, top_level_mark, true));
        // 2. strip: TypeScript 固有の型情報などを除去
        let module = module.apply(strip(unresolved_mark, top_level_mark));
        // 3. hygiene: 識別子の衝突回避のための修正
        let module = module.apply(hygiene());
        // 4. fixer: 括弧など不足している部分の補完
        let program = module.apply(fixer(Some(&comments)));

        // AST から JavaScript コード文字列を生成
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
            interface Args {
                name: string;
            }
            function greet(arg: Args): void {
                console.log(`Hello, ${arg.name}!`);
            }
        "#;
        let js = transform_ts_to_js(ts).expect("変換に失敗しました");
        // TypeScript の型注釈が除去され、JS のコードが得られることを確認
        assert!(js.contains("function greet("));
        assert!(!js.contains("string")); // 型注釈が含まれていないこと
    }
}
