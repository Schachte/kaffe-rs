use anyhow::anyhow;
use std::{io::Write, path::Path, rc::Rc};

use deno_core::{anyhow, error::AnyError, v8, JsRuntime};
use tempfile::NamedTempFile;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, multispace0, multispace1, newline},
    combinator::{map, opt},
    error::Error,
    multi::many0,
    sequence::delimited,
    IResult,
};

#[derive(Debug)]
pub enum ASTNode {
    Import(ImportType),
    Heading(u8, String),
    ReactComponent(String),
    Paragraph(String),
    Text(String),
}

#[derive(Debug)]
pub enum ImportType {
    Named(String, String), // For imports like: import { Home } from "./components/Home";
    Default(String, String), // For imports like: import Home from "./components/Home";
}

fn parse_default_import(input: &str) -> IResult<&str, ImportType> {
    let (input, component) = take_until(" from")(input)?;
    let (input, _) = tag(" from")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, path) = delimited(char('\"'), take_until("\""), char('\"'))(input)?;
    Ok((
        input,
        ImportType::Default(component.trim().to_owned(), path.to_owned()),
    ))
}

fn parse_named_import(input: &str) -> IResult<&str, ImportType> {
    let (input, component) = delimited(char('{'), take_until("}"), char('}'))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("from")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, path) = delimited(char('\"'), take_until("\""), char('\"'))(input)?;
    Ok((
        input,
        ImportType::Named(component.trim().to_owned(), path.to_owned()),
    ))
}

fn parse_import(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = tag("import")(input.trim())?;
    let (input, _) = multispace1(input)?;
    let (input, import_type) = alt((parse_default_import, parse_named_import))(input)?;
    let (input, _) = opt(char(';'))(input)?;
    Ok((input, ASTNode::Import(import_type)))
}

pub fn parse_heading(input: &str) -> IResult<&str, ASTNode> {
    let (input, level) = map(take_while1(|c| c == '#'), |s: &str| s.len() as u8)(input.trim())?;
    let (input, _) = char(' ')(input)?;
    let (input, content) = take_until("\n")(input)?;
    let (input, _) = opt(newline)(input)?;
    Ok((input, ASTNode::Heading(level, content.trim().to_string())))
}

pub fn parse_paragraph(input: &str) -> IResult<&str, ASTNode> {
    let (input, content) = take_until("\n\n")(input.trim())?;
    let (input, _) = opt(tag("\n\n"))(input)?;
    Ok((input, ASTNode::Paragraph(content.trim().to_string())))
}

pub fn parse_text(input: &str) -> IResult<&str, ASTNode> {
    let (input, content) = take_until("\n")(input.trim())?;
    let (input, _) = opt(newline)(input)?;
    Ok((input, ASTNode::Text(content.trim().to_string())))
}

pub fn parse_markdown(input: &str) -> Result<Vec<ASTNode>, anyhow::Error> {
    let (_, nodes) = many0(alt((
        parse_import,
        parse_heading,
        parse_react_component,
        parse_paragraph,
        parse_text,
    )))(input)
    .map_err(|e| anyhow!("Failed to parse markdown: {:?}", e))?;

    Ok(nodes)
}

fn parse_react_component(input: &str) -> IResult<&str, ASTNode, Error<&str>> {
    // Parse a React component in the form of <ComponentName> or <ComponentName />
    let (input, _) = tag("<")(input.trim())?; // Parse the opening '<'
    let (input, component_name) = take_while1(|c: char| c.is_alphanumeric())(input)?; // Parse component name
    let (input, _) = multispace0(input)?; // Allow for any spaces

    // Check if the tag is self-closing or has children
    if let Ok((input, _)) = tag::<_, _, Error<&str>>("/>")(input) {
        // If it's a self-closing tag, we successfully parsed it
        return Ok((input, ASTNode::ReactComponent(component_name.to_string())));
    }

    // If not self-closing, it should have a closing tag
    let (input, _) = tag(">")(input)?; // Parse the closing '>'
    let closing_tag = format!("</{}>", component_name); // Construct the closing tag
    let (input, _) = take_until(closing_tag.as_str())(input)?; // Skip over the content
    let (input, _) = tag(closing_tag.as_str())(input)?; // Parse the closing tag

    // Return the parsed component
    Ok((input, ASTNode::ReactComponent(component_name.to_string())))
}

async fn run_js(
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

fn retrieve_rendered_page(
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

pub async fn generate_html(ast: &[ASTNode]) -> Result<String, deno_core::anyhow::Error> {
    let mut html = String::new();
    let mut imports = Vec::new();
    let mut react_components = Vec::new();

    for node in ast {
        match node {
            ASTNode::Import(import_type) => match import_type {
                ImportType::Named(component, path) => {
                    imports.push(format!("import {{ {} }} from '{}';", component, path));
                    react_components.extend(component.split(',').map(|s| s.trim().to_string()));
                }
                ImportType::Default(component, path) => {
                    imports.push(format!("import {} from '{}';", component, path));
                    react_components.push(component.to_string());
                }
            },
            ASTNode::Heading(level, content) => {
                html.push_str(&format!("<h{}>{}</h{}>\n", level, content, level));
            }
            ASTNode::Paragraph(content) => {
                html.push_str(&format!("<p>{}</p>\n", content));
            }
            ASTNode::Text(content) => {
                html.push_str(&format!("{}\n", content));
            }
            ASTNode::ReactComponent(component_name) => {
                react_components.push(component_name.clone());
                html.push_str(&format!("<div id=\"{}\" />", component_name));
            }
        }
    }

    let import_string = imports.join("\n");
    let react_component_string = react_components.join(", ");

    let script = format!(
        r#"
        {import_string}
        import React from "react";
        import ReactDOMServer from "react-dom/server";

        const components = {{ {react_component_string} }};

        // Expose the renderToString function globally
        (globalThis).renderToString = (location = "/") => {{
            const content = Object.entries(components).map(([name, Component]) => {{
                return `<div id="${{name}}-ssr">${{
                    ReactDOMServer.renderToString(<Component />)
                }}</div>`;
            }}).join('');

            return `
                <div id="root">
                    {html}
                    ${{content}}
                </div>
            `;
        }};
        "#,
        import_string = import_string,
        react_component_string = react_component_string,
        html = html
    );

    let mut js_runtime = JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        ..Default::default()
    });

    // Write the script to a temporary file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(script.as_bytes())?;

    // Run the script using Deno's V8 runtime
    run_js(&mut js_runtime, temp_file.path().to_str().unwrap()).await?;

    // Retrieve the rendered HTML for each React component
    for component_name in react_components {
        let rendered_html =
            retrieve_rendered_page(&mut js_runtime, "renderReactComponent", &component_name)?;
        html = html.replace(
            &format!("<div id=\"{}\" />", component_name),
            &rendered_html,
        );
    }

    Ok(html)
}
