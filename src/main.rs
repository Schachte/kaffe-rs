use actix_files as fs;
use actix_web::Result;
use actix_web::{middleware, App, HttpServer};
use clap::Parser;
use std::fs::create_dir_all;
use std::{fs as std_fs, io};
use tokio::fs as tokio_fs;
use tokio::io::ErrorKind;

use walkdir::WalkDir;

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
    #[arg(short = 'i', long, default_value = "examples")]
    input_directory: PathBuf,

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // The generated server/client entrypoints will need the components to exist relative to
    // the files, so we just copy them to the build dir
    let _ = copy_files(&args.client_component_directory, &args.client_build_dir);

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

async fn bundle_react_component(
    markdown_input: &str,
    file_name: &str,
) -> Result<String, anyhow::Error> {
    let parsed_nodes = match parse_markdown(markdown_input) {
        Ok(nodes) => nodes,
        Err(e) => return Err(anyhow!("Failed to parse markdown: {:?}", e)),
    };

    let (html_content, imports, react_components) = generate_html(&parsed_nodes)
        .await
        .map_err(|e| anyhow!("Failed to generate HTML: {:?}", e))?;

    let mut imports_string = imports.join("\n");
    let mut components_string = react_components.join(", ");

    if react_components.len() == 0 {
        components_string = "\"\"".to_string();
    }

    if imports.len() == 0 {
        imports_string = "".to_string();
    }

    let server_entry_content = load_file_contents("client/src/server-entry.template")?;
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_IMPORTS }}%", &imports_string);
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_COMPONENTS }}%", &components_string);
    let server_entry_content =
        server_entry_content.replace("%{{ REPLACE_CONTENT }}%", &html_content);

    let client_entry_content = load_file_contents("client/src/client-entry.template")?;
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

    let status = Command::new("node")
        .args(&["client/build.cjs", &format!("{}.js", &file_name)])
        .output()
        .map_err(|e| anyhow!("Failed to execute build script: {:?}", e))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(anyhow!("Build script failed: {}", stderr));
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

        if source_path.is_dir() {
            // Recursively copy the directory
            copy_files(
                &source_path.to_string_lossy(),
                &target_path.to_string_lossy(),
            )?;
        } else if source_path.is_file() {
            // Copy the file
            std_fs::copy(&source_path, &target_path)?;
        }
    }

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

async fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    tokio_fs::create_dir_all(&args.output_dir).await?;
    tokio_fs::create_dir_all(args.output_dir.join("static")).await?;

    process_markdown_files(&args.input_directory, &args.output_dir, &args).await?;

    println!("Files generated successfully");
    Ok(())
}

fn filename_without_extension(path: &PathBuf) -> Option<String> {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(String::from)
}

fn path_to_filename_without_extension(path: &Path) -> String {
    path.file_name()
        .and_then(|os_str| os_str.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string())
}

async fn process_markdown_files(
    input_dir: &Path,
    output_dir: &Path,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(input_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ext == "md" || ext == "mdx")
        {
            let relative_path = path.strip_prefix(input_dir)?.to_path_buf();
            let output_path = output_dir.join(&relative_path).with_extension("html");
            if let Some(parent) = output_path.parent() {
                std_fs::create_dir_all(parent)?;
            }
            process_single_file(&path, &output_path).await?;

            let filename_noext =
                match filename_without_extension(&path.strip_prefix(input_dir)?.to_path_buf()) {
                    Some(name) => name,
                    None => "".to_string(),
                };

            let css_path = format!("client/dist/{}.css", &filename_noext);
            // Check if the file exists
            match tokio_fs::metadata(&css_path).await {
                Ok(metadata) if metadata.is_file() => {
                    // File exists and is a regular file, proceed to read it
                    let client_css = tokio_fs::read_to_string(&css_path).await?;

                    // Write the CSS content to the output directory
                    tokio_fs::write(
                        args.output_dir
                            .join(format!("static/{}.css", &filename_noext)),
                        client_css,
                    )
                    .await?;
                }
                Ok(_) => {
                    eprintln!("The path exists but is not a file: {}", css_path);
                }
                Err(err) if err.kind() == ErrorKind::NotFound => {}
                Err(err) => {
                    eprintln!("An error occurred while checking the file: {}", err);
                }
            }
        }
    }
    Ok(())
}

async fn process_single_file(
    input_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let markdown_input = tokio_fs::read_to_string(input_path).await?;
    let filename = input_path
        .file_name()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid filename"))?
        .to_string();

    let _ = bundle_react_component(
        &markdown_input,
        path_to_filename_without_extension(&input_path).as_str(),
    )
    .await?;

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    run_js(
        &mut js_runtime,
        format!(
            "client/dist/ssr-{}.js",
            path_to_filename_without_extension(&input_path)
        )
        .as_str(),
    )
    .await?;

    let rendered_html = retrieve_rendered_html(&mut js_runtime)?;
    let template = tokio_fs::read_to_string("client/template.html").await?;

    let final_html = template
        .replace("{{SSR_CONTENT}}", &rendered_html)
        .replace(
            "{{CLIENT_BUNDLE_PATH}}",
            format!("{}.js", path_to_filename_without_extension(&input_path)).as_str(),
        )
        .replace(
            "{{CLIENT_CSS_PATH}}",
            format!("{}.css", path_to_filename_without_extension(&input_path)).as_str(),
        )
        .replace("{{TITLE}}", &filename);

    tokio_fs::write(output_path, final_html).await?;

    Ok(())
}
