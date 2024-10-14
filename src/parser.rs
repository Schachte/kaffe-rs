use anyhow::anyhow;
use deno_core::error::AnyError;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_until, take_while1},
    character::complete::{char, multispace0, multispace1, newline, none_of, space0},
    combinator::{map, opt, peek, recognize, value},
    error::Error,
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug)]
pub enum ASTNode {
    Import(ImportType),
    Link(String, String),
    Heading(u8, String),
    ReactComponent(String),
    Paragraph(String),
    Text(String),
    Strong(String),
    Emphasis(String),
    Code(String),
    CodeBlock(String, String),
    Image(String, String),
    List(Vec<String>),
    BlockQuote(String),
    Whitespace(String),
}

#[derive(Debug)]
pub enum ImportType {
    Named(String, String),
    Default(String, String),
}

fn parse_default_import(input: &str) -> IResult<&str, ImportType> {
    let (input, component) = take_until(" from")(input)?;
    let (input, _) = tag(" from")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, path) = delimited(char('"'), take_until("\""), char('"'))(input)?;
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
    let (input, path) = delimited(char('"'), take_until("\""), char('"'))(input)?;
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

fn parse_inside_brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('['), is_not("]"), char(']'))(input)
}

fn parse_url(input: &str) -> IResult<&str, &str> {
    delimited(char('('), take_until(")"), char(')'))(input)
}

fn parse_link(input: &str) -> IResult<&str, ASTNode> {
    map(tuple((parse_inside_brackets, parse_url)), |(text, url)| {
        ASTNode::Link(text.to_string(), url.to_string())
    })(input)
}

fn parse_image(input: &str) -> IResult<&str, ASTNode> {
    map(
        preceded(char('!'), tuple((parse_inside_brackets, parse_url))),
        |(alt_text, url)| ASTNode::Image(alt_text.to_string(), url.to_string()),
    )(input)
}

fn parse_emphasis(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = char('*')(input)?;
    let (input, text) = take_until("*")(input)?;
    let (input, _) = char('*')(input)?;
    Ok((input, ASTNode::Emphasis(text.to_string())))
}

fn parse_strong(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = tag("**")(input)?;
    let (input, text) = take_until("**")(input)?;
    let (input, _) = tag("**")(input)?;
    Ok((input, ASTNode::Strong(text.to_string())))
}

fn parse_code(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = char('`')(input.trim())?;
    let (input, code) = take_until("`")(input)?;
    let (input, _) = char('`')(input)?;
    Ok((input, ASTNode::Code(code.to_string())))
}

fn parse_list(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = many0(alt((tag("\n"), tag("\r\n"))))(input.trim())?;
    let (input, items) = many1(parse_list_item)(input)?;
    let (input, _) = many0(alt((tag("\n"), tag("\r\n"))))(input)?;
    Ok((input, ASTNode::List(items)))
}

fn parse_list_item(input: &str) -> IResult<&str, String> {
    let (input, _) = many0(alt((tag("\n"), tag("\r\n"))))(input)?;
    let (input, _) = preceded(space0, alt((tag("- "), tag("* "))))(input)?;
    let (input, item) = alt((
        terminated(take_until("\n"), peek(alt((tag("\n"), tag("\r\n"))))),
        recognize(many1(none_of("\n"))),
    ))(input)?;
    Ok((input, item.trim().to_string()))
}

fn parse_blockquote(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = char('>')(input)?;
    let (input, _) = space0(input)?;
    let (input, content) = take_until("\n")(input)?;
    let (input, _) = newline(input)?;
    Ok((input, ASTNode::BlockQuote(content.trim().to_string())))
}

pub fn parse_paragraph(input: &str) -> IResult<&str, ASTNode> {
    let (input, content) = take_until("\n\n")(input.trim())?;
    let (input, _) = opt(tag("\n\n"))(input)?;
    if content.starts_with("```") && content.ends_with("```") {
        parse_code_block(content)
    } else if content.starts_with("- ") || content.starts_with("* ") {
        parse_list(content)
    } else {
        Ok((input, ASTNode::Paragraph(content.trim().to_string())))
    }
}

pub fn parse_text(input: &str) -> IResult<&str, ASTNode> {
    let (input, content) = take_until("\n")(input.trim())?;
    let (input, _) = opt(newline)(input)?;
    Ok((input, ASTNode::Text(content.trim().to_string())))
}

fn parse_code_block(input: &str) -> IResult<&str, ASTNode> {
    let (input, _) = tag("```")(input)?;
    let (input, lang) = opt(take_while1(|c: char| c.is_alphanumeric()))(input)?;
    let (input, _) = opt(newline)(input)?;
    let (input, code) = take_until("```")(input)?;
    let (input, _) = tag("```")(input)?;
    let (input, _) = opt(newline)(input)?;
    Ok((
        input,
        ASTNode::CodeBlock(
            code.to_string().trim().to_string(),
            lang.unwrap_or("").to_string(),
        ),
    ))
}

fn parse_whitespace(input: &str) -> IResult<&str, ()> {
    value((), multispace0)(input)
}

pub fn parse_markdown(input: &str) -> Result<Vec<ASTNode>, AnyError> {
    let (_, nodes) = many0(delimited(
        parse_whitespace,
        alt((
            parse_import,
            parse_link,
            parse_code_block,
            parse_heading,
            parse_react_component,
            parse_list,
            parse_paragraph,
            parse_text,
            parse_code,
            parse_emphasis,
            parse_strong,
            parse_image,
            parse_blockquote,
        )),
        parse_whitespace,
    ))(input)
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

pub async fn generate_html(
    ast: &[ASTNode],
) -> Result<(String, Vec<String>, Vec<String>), anyhow::Error> {
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
            ASTNode::CodeBlock(content, lang) => {
                html.push_str(&format!(
                    "<pre><code className=\"language-{}\">{}</code></pre>\n",
                    lang, content
                ));
            }
            ASTNode::Text(content) => {
                html.push_str(&format!("{}\n", content));
            }
            ASTNode::Strong(content) => {
                html.push_str(&format!("<strong>{}</strong>", content));
            }
            ASTNode::Emphasis(content) => {
                html.push_str(&format!("<em>{}</em>", content));
            }
            ASTNode::Code(content) => {
                html.push_str(&format!("<code>{}</code>", content));
            }
            ASTNode::Link(text, url) => {
                html.push_str(&format!("<a href=\"{}\">{}</a>", url, text));
            }
            ASTNode::Image(alt_text, url) => {
                html.push_str(&format!("<img src=\"{}\" alt=\"{}\">", url, alt_text));
            }
            ASTNode::List(items) => {
                html.push_str("<ul>\n");
                for item in items {
                    html.push_str(&format!("  <li>{}</li>\n", item));
                }
                html.push_str("</ul>\n");
            }
            ASTNode::BlockQuote(content) => {
                html.push_str(&format!("<blockquote>{}</blockquote>\n", content));
            }
            ASTNode::ReactComponent(component_name) => {
                if !react_components.contains(component_name) {
                    react_components.push(component_name.clone());
                }
                html.push_str(&format!("<{}></{}>\n", component_name, component_name));
            }
            ASTNode::Whitespace(_) => {}
        }
    }

    Ok((html, imports, react_components))
}
