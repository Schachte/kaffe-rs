use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer, Result};
use deno_core::futures::TryFutureExt; // Add this import
use deno_core::{v8, JsRuntime, PollEventLoopOptions, RuntimeOptions};
use std::fs as std_fs;

#[get("/")]
async fn index() -> Result<HttpResponse> {
    let mut runtime = JsRuntime::new(RuntimeOptions::default());

    // Read the SSR script
    let ssr_script = std_fs::read_to_string("client/dist/ssr.js").map_err(|e| {
        eprintln!("Failed to read SSR script: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to read SSR script")
    })?;

    // Wrap the script in an async IIFE
    let wrapped_script = format!(
        r#"
    globalThis.ssrResult = await (async () => {{
        const {{default: ssr}} = await import('data:text/javascript;base64,{}');
        return ssr;
    }})();
    "#,
        base64::encode(&ssr_script)
    );

    println!("{}", wrapped_script);

    // Execute the SSR script
    runtime
        .execute_script("<anon>", wrapped_script)
        .map_err(|e| {
            eprintln!("Failed to execute SSR script: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to execute SSR script")
        })?;

    // Run the event loop
    runtime
        .run_event_loop(PollEventLoopOptions::default())
        .await
        .map_err(|e| {
            eprintln!("Error in event loop: {}", e);
            actix_web::error::ErrorInternalServerError("Error in event loop")
        })?;

    // Retrieve the SSR result from the global scope
    let result = runtime
        .execute_script("<anon>", "globalThis.ssrResult")
        .map_err(|e| {
            eprintln!("Failed to get SSR result: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get SSR result")
        })?;

    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, result);
    let result_string = local.to_string(scope).unwrap().to_rust_string_lossy(scope);

    println!("Rendered HTML: {}", result_string);

    // Construct the full HTML response
    let html = format!(
        r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>React SSR with Rust</title>
            </head>
            <body>
                <div id="root">{}</div>
                <script src="/static/bundle.js"></script>
            </body>
        </html>
        "#,
        result_string
    );

    // Return the result as an HTTP response
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(fs::Files::new("/static", "client/dist").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
