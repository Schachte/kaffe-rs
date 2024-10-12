use actix_files as fs;
use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use deno_core::{error::AnyError, v8, JsRuntime};
use std::rc::Rc;

/// Runs a JavaScript file in the provided JsRuntime.
///
/// # Arguments
///
/// * `js_runtime` - A mutable reference to the JsRuntime.
/// * `file_path` - The path to the JavaScript file.
///
/// # Returns
///
/// * `Result<usize, AnyError>` - The module ID if successful, or an error if failed.
async fn run_js(js_runtime: &mut JsRuntime, file_path: &str) -> Result<usize, AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(mod_id)
}

/// Retrieves the rendered page by executing a JavaScript function.
///
/// # Arguments
///
/// * `js_runtime` - A mutable reference to the JsRuntime.
/// * `function_name` - The name of the JavaScript function to execute.
/// * `argument` - The argument to pass to the JavaScript function.
///
/// # Returns
///
/// * `Result<String>` - The rendered page as a string if successful, or an error if failed.
fn retrieve_rendered_page(
    js_runtime: &mut JsRuntime,
    function_name: &str,
    argument: &str,
) -> Result<String> {
    println!("arg: {}", argument);
    let script = format!("{}('{}');", function_name, argument);
    let scope = &mut js_runtime.handle_scope();
    let source = v8::String::new(scope, &script).unwrap();
    let script = v8::Script::compile(scope, source, None).unwrap();
    let result = script.run(scope).unwrap();
    let result_str = result.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Ok(result_str)
}

/// Handles the index route and renders the server-side HTML.
///
/// # Arguments
///
/// * `req` - The HTTP request.
///
/// # Returns
///
/// * `impl Responder` - The HTTP response containing the rendered HTML.
#[get("/{tail:.*}")]
async fn index(req: HttpRequest) -> impl Responder {
    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    // TODO: Determine if we'll actually want this psuedo polyfill
    let _ = &mut js_runtime
        .execute_script(
            "process.js",
            r#"
        const process = {
            env: { NODE_ENV: "production" },
            browser: false,
            cwd: () => "/",
            emitWarning: (warning) => console.warn(warning),
        };
        globalThis.process = process;
        "#,
        )
        .unwrap();

    let path = req.path();
    let file_path = "client/dist/ssr.js";
    run_js(&mut js_runtime, file_path)
        .await
        .expect("Should run without issue");

    let ssr_result = match retrieve_rendered_page(&mut js_runtime, "renderSSR", path) {
        Ok(result) => result,
        Err(e) => format!("Error: {}", e),
    };

    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>React SSR with Rust</title>
            </head>
            <body>
                <div id="app">{}</div>
                <script type="module" src="/static/bundle.js"></script>
            </body>
        </html>
        "#,
        ssr_result
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/static", "client/dist").show_files_listing())
            .service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
