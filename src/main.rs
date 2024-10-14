use actix_web::Result;
use anyhow::anyhow;
use deno_core::{error::AnyError, v8, JsRuntime};
use std::{
    fs::{read_to_string, File},
    io::Write,
    process::Command,
    rc::Rc,
};

use kaffe::parser::{generate_html, parse_markdown};

async fn bundle_react_component(markdown_input: &str) -> Result<String, anyhow::Error> {
    let parsed_nodes = match parse_markdown(markdown_input) {
        Ok(nodes) => nodes,
        Err(e) => return Err(anyhow!("Failed to parse markdown: {:?}", e)),
    };

    let (html_content, imports, react_components) = generate_html(&parsed_nodes)
        .await
        .map_err(|e| anyhow!("Failed to generate HTML: {:?}", e))?;

    let imports_string = imports.join("\n");
    let components_string = react_components.join(", ");

    let server_entry_content = format!(
        r#"import React from "react";
        import "fast-text-encoding";
        import ReactDOMServer from "react-dom/server";

        {imports}

        (globalThis as any).renderToString = (location = "/") => {{
            const components = {{ {components} }};
            return ReactDOMServer.renderToString(
                <>
                    {html}
                </>
            );
        }};
        "#,
        imports = imports_string,
        components = components_string,
        html = html_content
    );

    let client_entry_content = format!(
        r#"import * as React from "react";
import {{ hydrateRoot }} from "react-dom/client";
{imports}

const container = document.getElementById("root");
if (container) {{
  hydrateRoot(
    container,
    <React.StrictMode>
      <>
      {html}
      </>
    </React.StrictMode>
  );
}} else {{
  console.error("Container element with id 'root' not found");
}}
"#,
        imports = imports_string,
        html = html_content // Insert the HTML directly
    );

    let file_path = "client/src/server-entry.tsx";
    let mut file = File::create(file_path)
        .map_err(|e| anyhow!("Failed to create file at {}: {:?}", file_path, e))?;

    file.write_all(server_entry_content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file at {}: {:?}", file_path, e))?;

    let file_path = "client/src/client-entry.tsx";
    let mut file = File::create(file_path)
        .map_err(|e| anyhow!("Failed to create file at {}: {:?}", file_path, e))?;

    file.write_all(client_entry_content.as_bytes())
        .map_err(|e| anyhow!("Failed to write to file at {}: {:?}", file_path, e))?;

    // Run the build script
    let status = Command::new("node")
        .args(&["client/build.cjs"])
        .output() // Capture the output
        .map_err(|e| anyhow!("Failed to execute build script: {:?}", e))?;

    if !status.status.success() {
        // Log the output
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

async fn run_js(js_runtime: &mut JsRuntime, file_path: &str) -> Result<usize, AnyError> {
    let main_module = deno_core::resolve_path(file_path, &std::env::current_dir()?)?;
    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(mod_id)
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

    # hi 
    this is text
    ## heading 

    ```javascript
    var x = 5;
    ```

    - hello
    - this is list
    - item aganin
"#;

    // Call to bundle_react_component and await its result
    let _ = bundle_react_component(&markdown_input).await?;

    // Run the JavaScript bundle
    match run_js(&mut js_runtime, "client/dist/ssr.js").await {
        Ok(_) => (),
        Err(e) => println!("Error running JavaScript bundle: {}", e),
    }

    // Retrieve the rendered HTML
    let rendered_html = retrieve_rendered_html(&mut js_runtime)
        .map_err(|e| anyhow!("Error retrieving rendered HTML: {}", e))?;

    // Load the HTML template
    let template = load_html_template("client/template.html")?;

    // Inject the rendered HTML into the template
    let final_html = template.replace("{{SSR_CONTENT}}", &rendered_html);
    let final_html = final_html.replace("{{CLIENT_BUNDLE_PATH}}", "bundle.js");

    // Write the final HTML to a file
    let output_file_path = "output/index.html";

    // Open the file for writing (overwriting if it exists)
    let mut output_file = File::create(output_file_path)
        .map_err(|e| anyhow!("Failed to create or overwrite output file: {:?}", e))?;

    // Write the final HTML content to the file
    output_file
        .write_all(final_html.as_bytes())
        .map_err(|e| anyhow!("Failed to write to output file: {:?}", e))?;

    println!("Files generated successfully, you may run via python3 server.py to test");
    Ok(())
}
