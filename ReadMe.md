# WGPU Render

## Linux

### Linux Prerequisites

```SH
cargo install cargo-watch
```

### Linux Run and Watch

```SH
cargo watch -x run
```

## Web

### WASM Prerequisites

Install `wasm-server-runner`

```SH
cargo install cargo-watch
cargo install wasm-server-runner
```

### WASM Run and Watch

```SH
WASM_SERVER_RUNNER_CUSTOM_INDEX_HTML=index.html cargo watch -x "run --target wasm32-unknown-unknown"
```

### Use `wee_alloc` as allocator

Add optional feature Flag `--features wee_alloc` to compile with `wee_alloc` as global allocator.

## Shader Validation

### Shader Prerequisites

```SH
cargo install cargo-wgsl
```

### Validate Shaders

```SH
cargo wgsl
```
