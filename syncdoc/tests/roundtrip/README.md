To run just one test

```sh
cargo nextest run -F cli roundtrip_section
```

The tests will automatically set the `SYNCDOC_DEBUG` env var
