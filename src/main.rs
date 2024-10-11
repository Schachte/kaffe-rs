use std::rc::Rc;
use std::task::Poll;

use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer, Result};
use deno_core::anyhow::Error;
use deno_core::{resolve_path, v8, JsRuntime, PollEventLoopOptions, RuntimeOptions};

use deno_core::error::AnyError;
use deno_core::ModuleSpecifier;
use std::path::PathBuf;

async fn run_js(js_runtime: &mut deno_core::JsRuntime, file_path: &str) -> Result<usize, AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(mod_id)
}

#[get("/")]
async fn index() -> Result<HttpResponse> {
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    let file_path = "client/dist/ssr.js";
    let module_id = run_js(&mut js_runtime, file_path).await.unwrap();
    let global = js_runtime.get_module_namespace(module_id).unwrap();
    let ssr_result = {
        let mut scope = js_runtime.handle_scope();
        let global = scope.get_current_context().global(&mut scope);
        let func_name = v8::String::new(&mut scope, "ssrResult").unwrap();
        let func = global.get(&mut scope, func_name.into()).unwrap();
        let func = v8::Local::<v8::String>::try_from(func).unwrap();
        func.to_string(&mut scope)
            .unwrap()
            .to_rust_string_lossy(&mut scope)
    };

    println!("Rendered HTML: {}", ssr_result);

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
                <script type="module" src="/static/bundle.js"></script>
            </body>
        </html>
        "#,
        ssr_result
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
