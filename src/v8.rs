use std::path::Path;

use deno_core::{error::AnyError, v8, JsRuntime};

pub async fn run_js(
    js_runtime: &mut JsRuntime,
    file_path: &str,
) -> Result<(), deno_core::anyhow::Error> {
    let main_module = deno_core::resolve_path(file_path, Path::new(file_path))?;
    let mod_id = js_runtime.load_main_es_module(&main_module).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(Default::default()).await?;
    result.await?;
    Ok(())
}

pub fn retrieve_rendered_page(
    js_runtime: &mut JsRuntime,
    function_name: &str,
    argument: &str,
) -> Result<String, AnyError> {
    let script = format!("{}('{}');", function_name, argument);
    let scope = &mut js_runtime.handle_scope();
    let source = v8::String::new(scope, &script).unwrap();
    let script = v8::Script::compile(scope, source, None).unwrap();
    let result = script.run(scope).unwrap();
    let result_str = result.to_string(scope).unwrap().to_rust_string_lossy(scope);
    Ok(result_str)
}
