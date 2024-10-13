## Background

Experimental static site generation engine for React built in Rust using V8 engine from Deno.

My goal is to create a Markdown first SSG generation engine that accepts embedded React components for interactive components.

## How it works

This is a high-level overview of how the engine will run.

### Markdown AST Generation

The markdown piece is fairly trivial given the simplicity of the language itself. We need to parse the characters of the file and generate an AST to iterate over later. The iteration will allow us to create the `ASTNode` -> `Markdown HTML` mapping such as `## hello` -> `<h2>hello</h2>`.

### Bundling & Dynamic HTML generation

The complex piece of this is getting embedded React components to work within the Markdown, similar to MDX. Assume we have an example file such as:

```jsx
import Home from "./components/Home";

# Welcome to My App

This is a paragraph.

<Home></Home>
```

Clearly, this is non-traditional Markdown as we have a React component within. We need a way to resolve the import and also render the component itself to HTML. The way Kaffe handles this is a multi-step process.

1. Generate AST which will create nodes for imports, headings, react components, paragraphs, text, etc.
2. Extract the React components
3. Extract the import statements
4. Dynamically generate a `.tsx/.jsx` React file with the imports and components within.

Once we have this temp file of the react component(s), we can run it through a bundler to get a standard JS output file compatible for running through Node or V8 to extract the generated HTML. The HTML generation is being done through the help of `react-dom/server`, which is traditionally used for SSR (static site rendering).

5. Run the output bundle through the V8 runtime - this will generate the HTML for us and assign to a global variable.
6. Invoke the runtime by grabbing the HTML string we assigned to the global scope and output it to get the HTML

### Example

```
let markdown_input = r#"
   import Home from "./components/Home";
   <Home/>
"#;
```

_Invoke Kaffe_

```
<div><button>YAY</button><div>hello</div></div>
```
