# kaffe

**kaffe** is a transpiler that converts Markdown files with embedded React components into a static site. Written in Rust, **kaffe** offers a fast and efficient way to generate web content from Markdown, allowing you to integrate React components directly within your documents.

## Features

- **Transpilation:** Converts Markdown files into static HTML, processing any embedded React components.
- **Rust Performance:** Built in Rust for speed and efficiency, ensuring quick processing times.
- **Easy Integration:** Use React components seamlessly in your Markdown for enhanced interactivity.
- **Static Site Generation:** Create a complete static site ready for deployment.

## Getting Started

Currently, this repo is built as a POC to prove the possibilities of working with React, V8 and Rust. You can find an example input inside of [./src/main.rs](./src/main.rs).

_Example Input_

````markdown
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
````

_Example Output_

Note: The clientside bundle is automatically injected to support hydrating the DOM with interactivity powered by React.

```html
<!DOCTYPE html>
<html>
  <head>
    <title>{{TITLE}}</title>
  </head>
  <body>
    <div id="root">
      <div>
        <button>YAY</button>
      </div>
      <h1>hi</h1>
      <p>this is text ## heading</p>
      <pre>
        <code class="language-javascript">var x = 5;</code>
      </pre>
      <ul>
        <li>hello</li>
        <li>this is list</li>
        <li>item again</li>
      </ul>
    </div>
    <script type="module" src="/static/bundle.js"></script>
  </body>
</html>
```
