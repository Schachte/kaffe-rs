# kaffe

**kaffe** is a transpiler that converts Markdown files with embedded React components into a static site. Written in Rust, **kaffe** offers a fast and efficient way to generate web content from Markdown, allowing you to integrate React components directly within your documents.

## Features

- **Transpilation:** Converts Markdown files into static HTML, processing any embedded React components.
- **Rust Performance:** Built in Rust for speed and efficiency, ensuring quick processing times.
- **Easy Integration:** Use React components seamlessly in your Markdown for enhanced interactivity.
- **Static Site Generation:** Create a complete static site ready for deployment.

## Background

This is a low-level engine that exists to handle statically generating files to HTML from Markdown as a first-class file format.

Let's assume you want to deploy a new blog, but don't want to learn complex new frameworks, but just focus on the content and maybe some basic styling and interactive React components as you need.

1. Write a Markdown file
2. `kaffe -m your_blog_article.mdx`
3. HTML file generated

The Markdown supports React components with import statements like so:

```
import Home from "./components/Home";

<Home />

# hi
this is text
```

and Kaffe will generate:

```html
<!DOCTYPE html>
<html>
  <head>
    <title>{{TITLE}}</title>
  </head>
  <body>
    <div id="root">
      <div>
        <div>
          Button clicked:
          <!-- -->0<!-- -->
          times
        </div>
        <button>YAY</button>
      </div>
      <h1>hi</h1>
      <p>this is text</p>
      <h2>heading</h2>
      <pre><code class="language-javascript">var x = 5;</code></pre>
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

## Getting Started

1. `cd client && yarn`
2. From root, you can run the following:

```bash
cargo run -- \
    -m examples/example_with_react.mdx \
    -p 9090 \
    -c client/src/components \
    -b client/dist
```

_Output:_

```
Files copied successfully from client/src/components to client/dist
Files generated successfully
Starting server...
Server running successfully!
Open your browser and navigate to: http://localhost:9090
```

## File structure & processing explained

1. The `client` dir expects all components to live within `client/src/components`.

2. The templates for the entrypoints when doing SSR (server-side rendering) and client-side hydration exist in `client/src/*-entry.template.tsx`.

3. When the program runs, it loads the markdown file into memory _(see: [./examples](examples/directory))_, creates an AST from the source and handles the HTML compilation for both React and Markdown.

4. Since this supports Typescript out of the box, Kaffe transpiles the React source (.tsx) into a single Javascript bundle using `esbuild`.

5. On the server, we can do the SSR piece by invoking the bundle inside of a new V8 context (the Javascript engine that will compile and execute the bundle). Kaffe uses the `deno_core` implementation of the V8 engine.

6. In tandem, Kaffe will produce the client-side equivalent bundle that gets loaded on the client to handle any interactivity required by React, event handlers, etc.
