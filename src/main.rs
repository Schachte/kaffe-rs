use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use clap::Parser;
use colored::Colorize;
use deno_core::{error::AnyError, v8, JsRuntime};
use std::rc::Rc;

use kaffe::Kaffe;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value = "client/dist")]
    client_build_dir: String,

    #[arg(long, default_value = "bundle.js")]
    client_bundle_path: String,

    #[arg(long, default_value = "ssr.js")]
    server_bundle_path: String,

    #[arg(long, default_value_t = 8080)]
    server_port: u16,
}

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
async fn index(req: HttpRequest, kaffe: web::Data<Kaffe>) -> impl Responder {
    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });
    let path = req.path();
    let file_path = kaffe.server_bundle_path.clone();
    run_js(&mut js_runtime, &file_path)
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
                <script type="module" src="/static/{}"></script>
            </body>
        </html>
        "#,
        ssr_result, kaffe.client_bundle_path
    );
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let server_port = args.server_port;

    let kaffe = web::Data::new(Kaffe::new(
        args.client_build_dir,
        args.client_bundle_path,
        args.server_bundle_path,
        server_port,
    ));

    println!("\n{}", "Starting Kaffe Server...".bright_green().bold());
    println!("{}", "=========================".bright_green());
    println!("{} {}", "Port:".bright_yellow().bold(), server_port);
    println!(
        "{} {}",
        "Client Build Dir:".bright_yellow().bold(),
        kaffe.client_build_dir.display()
    );
    println!(
        "{} {}",
        "Client Bundle:".bright_yellow().bold(),
        kaffe.client_bundle_path
    );
    println!(
        "{} {}",
        "Server Bundle:".bright_yellow().bold(),
        kaffe.server_bundle_path
    );
    println!("{}", "=========================".bright_green());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(kaffe.clone())
            .service(fs::Files::new("/static", kaffe.client_build_dir.clone()).show_files_listing())
            .service(index)
    })
    .bind(format!("127.0.0.1:{}", server_port))?;

    println!("\n{}", "Server is running!".bright_green().bold());
    println!(
        "{} {}",
        "Local:".bright_yellow().bold(),
        format!("http://localhost:{}", server_port)
            .bright_blue()
            .underline()
    );
    println!(
        "{} {}",
        "Network:".bright_yellow().bold(),
        format!("http://127.0.0.1:{}", server_port)
            .bright_blue()
            .underline()
    );
    println!("\n{}", "Press Ctrl+C to stop the server".bright_cyan());

    server.run().await
}
