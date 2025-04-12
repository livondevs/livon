use livon_compiler::compile;
use warp::Filter;

#[tokio::main]
async fn main() {
    let compile = warp::path("compile")
        .and(warp::post())
        .and(warp::body::json())
        .map(|body: serde_json::Value| {
            let code = match &body["code"] {
                serde_json::Value::String(s) => s.to_string(),
                _ => panic!("code is not a string"),
            };
            let runtime_path = match body.get("runtimePath") {
                Some(v) => Some(
                    v.as_str()
                        .expect("runtime_path is not a string")
                        .to_string(),
                ),
                None => None,
            };
            match compile(code.clone(), runtime_path) {
                Ok(r) => {
                    warp::reply::with_status(warp::reply::json(&r), warp::http::StatusCode::OK)
                }
                Err(e) => warp::reply::with_status(
                    warp::reply::json(&e),
                    warp::http::StatusCode::BAD_REQUEST,
                ),
            }
        });

    warp::serve(compile).run(([127, 0, 0, 1], 3030)).await;
}
