# CLI Migration Demos

## Basic

To (re-)run the 'basic' demo of the CLI, using [just][just], run

[just]: https://github.com/casey/just

```sh
just cli-demo-basic
```

This will:

- run `cargo check` on the original demo crate (`demo_basic` directory) which will enforce the `missing_docs` lint,
  confirming that the initial crate builds and has all required docstrings.
- delete the output directory (`demo_basic_migrated`) if present
- rename the copied crate with `sed` so that it can live in the same workspace
- run `syncdoc` from inside the output directory, modifying the new crate
- run `cargo check` on the new crate which will enforce the `missing_docs` lint again, confirming
  that the syndoc migration preserved its documentation coverage.
