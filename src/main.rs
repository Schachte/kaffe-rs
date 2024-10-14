use actix_files as fs;
use actix_web::Result;
use actix_web::{middleware, App, HttpServer};
use clap::Parser;
use std::fs::create_dir_all;
use std::{fs as std_fs, io};

use std::path::{Path, PathBuf};

use anyhow::anyhow;
use deno_core::{error::AnyError, v8, JsRuntime};
use std::{
    fs::{read_to_string, File},
    io::Write,
    process::Command,
    rc::Rc,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'm', long, default_value = "examples/example_with_react.mdx")]
    markdown_path: PathBuf,

    #[arg(short = 'p', long, default_value = "8080")]
    server_port: String,

    #[arg(short = 'c', long, default_value = "client/src/components")]
    client_component_directory: String,

    #[arg(short = 'b', long, default_value = "client/dist/components")]
    client_build_dir: String,

    #[arg(short = 'o', long, default_value = "output")]
    output_dir: PathBuf,
}

use kaffe::parser::{generate_html, parse_markdown};

async fn bundle_react_component(
    markdown_input: &str,
    client_component_dir: &str,
    client_build_dir: &str,
    _file_name: &str,
) -> Result<String, anyhow::Error> {
    let parsed_nodes = match parse_markdown(markdown_input) {
        Ok(nodes) => nodes,
        Err(e) => return Err(anyhow!("Failed to parse markdown: {:?}", e)),
    };

    let (html_content, imports, react_components) = generate_html(&parsed_nodes)
        .await
        .map_err(|e| anyhow!("Failed to generate HTML: {:?}", e))?;

    let imports_string = imports.join("\n");
    let components_string = react_components.join(", ");

    let server_entry_content = load_file_contents("client/src/server-entry.template.tsx")?;
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_IMPORTS }}%", &imports_string);
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_COMPONENTS }}%", &components_string);
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_CONTENT }}%", &html_content);

    let client_entry_content = load_file_contents("client/src/client-entry.template.tsx")?;
    let client_entry_content =
        client_entry_content.replace("%{{ REPLACE_IMPORTS }}%", &imports_string);
    let client_entry_content =
        client_entry_content.replace("%{{ REPLACE_COMPONENTS }}%", &components_string);
    let client_entry_content =
        client_entry_content.replace("%{{ REPLACE_CONTENT }}%", &html_content);

    let output_dir = "client/dist";

    create_dir_all(output_dir)
        .map_err(|e| anyhow!("Failed to create directory at {}: {:?}", output_dir, e))?;

    // Write the server entry file
    let server_file_path = format!("{}/server-entry.tsx", output_dir);
    let mut server_file = File::create(&server_file_path)
        .map_err(|e| anyhow!("Failed to create file at {}: {:?}", server_file_path, e))?;
    server_file
        .write_all(server_entry_content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file at {}: {:?}", server_file_path, e))?;

    // Write the client entry file
    let client_file_path = format!("{}/client-entry.tsx", output_dir);
    let mut client_file = File::create(&client_file_path)
        .map_err(|e| anyhow!("Failed to create file at {}: {:?}", client_file_path, e))?;
    client_file
        .write_all(client_entry_content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file at {}: {:?}", client_file_path, e))?;

    // The generated server/client entrypoints will need the components to exist relative to
    // the files, so we just copy them to the build dir
    let _ = copy_files(&client_component_dir, &client_build_dir);

    let status = Command::new("node")
        .args(&["client/build.cjs"])
        .output()
        .map_err(|e| anyhow!("Failed to execute build script: {:?}", e))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        let stdout = String::from_utf8_lossy(&status.stdout);
        return Err(anyhow!(
            "Build script failed with status: {}. Output: {}, Errors: {}",
            status.status,
            stdout,
            stderr
        ));
    }

    Ok(html_content)
}

fn copy_files(source_dir: &str, target_dir: &str) -> io::Result<()> {
    std_fs::create_dir_all(target_dir)?;

    for entry in std_fs::read_dir(source_dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let source_path = entry.path();
        let target_path = Path::new(target_dir).join(file_name);

        if source_path.is_file() {
            std_fs::copy(&source_path, &target_path)?;
        }
    }

    println!(
        "Files copied successfully from {} to {}",
        source_dir, target_dir
    );
    Ok(())
}

async fn run_js(js_runtime: &mut JsRuntime, file_path: &str) -> Result<usize, AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(mod_id)
}

fn load_file_contents(file_path: &str) -> std::io::Result<String> {
    read_to_string(file_path)
}

fn retrieve_rendered_html(js_runtime: &mut JsRuntime) -> Result<String> {
    let scope = &mut js_runtime.handle_scope();
    let source = v8::String::new(scope, "renderToString();").unwrap();
    let script = v8::Script::compile(scope, source, None).unwrap();
    let result = script.run(scope).unwrap();
    let result_str = result.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Ok(result_str)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    if let Err(e) = run(&args).await {
        eprintln!("Error during file generation: {:?}", e);
        return Ok(());
    }

    println!("Starting server...");

    let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", &args.server_port))?;
    let port = listener.local_addr()?.port();

    println!("Server running successfully!");
    println!(
        "Open your browser and navigate to: http://localhost:{}",
        port
    );

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .service(fs::Files::new("/static", "output/static").show_files_listing())
            .service(fs::Files::new("/", "output").index_file("index.html"))
    })
    .listen(listener)?
    .run()
    .await
}

fn filename_from_path(filepath: &PathBuf) -> Result<String, String> {
    let path = filepath.as_path();
    if let Some(filename) = path.file_name() {
        if let Some(filename_str) = filename.to_str() {
            return Ok(filename_str.to_string());
        }
    }
    println!("Err");
    Err("No valid filename found.".to_string())
}

async fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    let markdown_input = std_fs::read_to_string(&args.markdown_path)
        .map_err(|e| anyhow!("Failed to read markdown file: {:?}", e))?;

    let md_filename = match filename_from_path(&args.markdown_path) {
        Ok(filename) => filename,
        Err(e) => format!("Error: {}", e),
    };

    println!("{} is filename", md_filename);

    let _ = bundle_react_component(
        &markdown_input,
        &args.client_component_directory,
        &args.client_build_dir,
        &md_filename,
    )
    .await?;

    match run_js(&mut js_runtime, "client/dist/ssr.js").await {
        Ok(_) => (),
        Err(e) => println!("Error running JavaScript bundle: {}", e),
    }

    let rendered_html = retrieve_rendered_html(&mut js_runtime)
        .map_err(|e| anyhow!("Error retrieving rendered HTML: {}", e))?;

    let template = load_file_contents("client/template.html")?;

    let final_html = template.replace("{{SSR_CONTENT}}", &rendered_html);
    let final_html = final_html.replace("{{CLIENT_BUNDLE_PATH}}", "bundle.js");
    let final_html = final_html.replace("{{TITLE}}", &md_filename);

    std_fs::create_dir_all("output/static")?;

    let output_file_path = "output/index.html";
    std_fs::write(output_file_path, final_html)?;

    let client_bundle = load_file_contents("client/dist/bundle.js")?;
    std_fs::write("output/static/bundle.js", client_bundle)?;
    println!("Files generated successfully");
    Ok(())
}
