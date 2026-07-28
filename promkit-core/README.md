# promkit-core

## Renderer layout benchmark

The Criterion benchmark measures `Renderer` layout independently from terminal
size queries and terminal I/O. It covers content size, wrapping and truncation,
terminal width, pane count, and viewport movement:

```bash
cargo bench -p promkit-core --bench renderer_layout
```

Each iteration includes cloning the renderer inputs, matching the content
snapshot cost paid by `Renderer::render`.
