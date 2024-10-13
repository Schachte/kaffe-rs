use actix_web::{
    get,
    web::{self},
    HttpRequest, HttpResponse, Responder, Result,
};
use deno_core::{error::AnyError, v8, JsRuntime};
use std::{
    fs::{read_to_string, File},
    io::Write,
    process::Command,
    rc::Rc,
};

use kaffe::{
    parser::{parse_markdown, ASTNode, ImportType},
    Kaffe,
};

fn extract_react_components(nodes: &[ASTNode]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|node| {
            if let ASTNode::ReactComponent(name) = node {
                Some(format!("<{} />", name))
            } else {
                None
            }
        })
        .collect()
}

fn extract_imports(nodes: &[ASTNode]) -> Vec<String> {
    nodes
        .iter()
        .filter_map(|node| {
            if let ASTNode::Import(import_type) = node {
                match import_type {
                    ImportType::Named(component, path) => {
                        Some(format!("import {{ {} }} from \"{}\";", component, path))
                    }
                    ImportType::Default(component, path) => {
                        Some(format!("import {} from \"{}\";", component, path))
                    }
                }
            } else {
                None
            }
        })
        .collect()
}

fn bundle_react_component(markdown_input: &str) -> Result<()> {
    // Parse the markdown input
    let parsed_nodes = match parse_markdown(markdown_input) {
        Ok(nodes) => nodes,
        Err(e) => return Ok(()),
    };

    // Extract React components
    let react_components = extract_react_components(&parsed_nodes);

    let imports = extract_imports(&parsed_nodes);

    println!("Parsed nodes: {:?}", parsed_nodes);
    println!("React components: {:?}", react_components);
    println!("Imports: {:?}", imports);

    // Join components
    let components_string = react_components.join(",\n    ");
    let imports_string = imports.join("\n");

    // Create the content for the server-entry file
    let server_entry_content = format!(
        r#"import React from "react";
        import "fast-text-encoding";
        import ReactDOMServer from "react-dom/server";
        {}

        // Expose the renderToString function globally
        (globalThis as any).renderToString = (location = "/") => {{
        return ReactDOMServer.renderToString(
            <React.Fragment>
            {}
            </React.Fragment>
        );
        }};
        "#,
        imports_string, components_string
    );

    // Write to the server-entry.tsx file
    let file_path = "client/src/server-entry.tsx";
    let mut file = File::create(file_path)?;
    file.write_all(server_entry_content.as_bytes())?;

    // Run the build script
    let status = Command::new("node").args(&["client/build.cjs"]).status()?;

    if !status.success() {
        return Ok(());
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

fn load_html_template(file_path: &str) -> std::io::Result<String> {
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

    let html_template = load_html_template(&kaffe.html_template_path)
        .unwrap_or_else(|_| String::from("Error loading HTML template"));

    let html = html_template
        .replace("{{SSR_CONTENT}}", &ssr_result)
        .replace("{{CLIENT_BUNDLE_PATH}}", &kaffe.client_bundle_path)
        .replace("{{TITLE}}", &path);

    HttpResponse::Ok().content_type("text/html").body(html)
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    // Example markdown input
    let markdown_input = r#"
        import Home from "./components/Home";
        <Home/>
    "#;

    bundle_react_component(&markdown_input)?;

    match run_js(&mut js_runtime, "dist/ssr.js").await {
        Ok(_) => println!("JavaScript bundle executed successfully"),
        Err(e) => println!("Error running JavaScript bundle: {}", e),
    }

    match retrieve_rendered_html(&mut js_runtime) {
        Ok(html) => println!("Rendered HTML: {}", html),
        Err(e) => println!("Error retrieving rendered HTML: {}", e),
    }

    Ok(())
}
