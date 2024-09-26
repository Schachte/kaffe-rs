use actix_files as fs;
use actix_web::{get, App, HttpResponse, HttpServer};
use deno_core::{serde_v8, v8, JsRuntime, RuntimeOptions};
use std::fs as std_fs;

use std::io::Result;
use std::process::Command;

fn run_node_script(script_path: &str) -> Result<String> {
    let output = Command::new("node").arg(script_path).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Node.js script exited with error: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

#[get("/")]
async fn index() -> HttpResponse {
    let mut runtime = JsRuntime::new(RuntimeOptions::default());

    // match run_node_script("client/dist/ssr.js") {
    //     Ok(output) => println!("Node.js script output: {}", output),
    //     Err(e) => eprintln!("Error running Node.js script: {}", e),
    // }

    // Read the bundled SSR script
    let ssr_script = std_fs::read_to_string("client/dist/ssr.js").unwrap();
    println!("{}", &ssr_script);
    let ssr_script_static = Box::leak(ssr_script.into_boxed_str());
    let output: serde_json::Value = eval(&mut runtime, ssr_script_static).expect("Eval failed");
    println!("{}", output);
    let html = format!(
        r#"
    <!DOCTYPE html>
    <html>
        <body>
            <div id="root">{}</div>
            <script src="/static/bundle.js"></script>
        </body>
    </html>
    "#,
        output
    );
    HttpResponse::Ok().content_type("text/html").body(html)
}

fn eval(context: &mut JsRuntime, code: &'static str) -> Result<serde_json::Value, String> {
    let res = context.execute_script("<anon>", code);
    match res {
        Ok(global) => {
            let scope = &mut context.handle_scope();
            let local = v8::Local::new(scope, global);
            // Deserialize a `v8` object into a Rust type using `serde_v8`,
            // in this case deserialize to a JSON `Value`.
            let deserialized_value = serde_v8::from_v8::<serde_json::Value>(scope, local);
            match deserialized_value {
                Ok(value) => Ok(value),
                Err(err) => Err(format!("Cannot deserialize value: {err:?}")),
            }
        }
        Err(err) => Err(format!("Evaling error: {err:?}")),
    }
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

// use std::io::Result;
// use std::process::Command;

// fn run_node_script(script_path: &str) -> Result<String> {
//     let output = Command::new("node").arg(script_path).output()?;

//     if output.status.success() {
//         Ok(String::from_utf8_lossy(&output.stdout).into_owned())
//     } else {
//         Err(std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!(
//                 "Node.js script exited with error: {}",
//                 String::from_utf8_lossy(&output.stderr)
//             ),
//         ))
//     }
// }

// // Example usage
// fn main() -> Result<()> {
//     match run_node_script("path/to/your/script.js") {
//         Ok(output) => println!("Node.js script output: {}", output),
//         Err(e) => eprintln!("Error running Node.js script: {}", e),
//     }
//     Ok(())
// }
