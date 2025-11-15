absolve:
    #!/usr/bin/env -S bash -euo pipefail
    if ! cargo tree -e no-dev -i syn 2>/dev/null | grep -q .; then
        echo -e "\033[38;2;255;255;255;48;2;0;0;0m free of \033[38;2;255;255;255;48;2;255;105;180m syn
    \033[38;2;255;255;255;48;2;0;0;0m\033[0m"
    else
        echo -e "\033[1;31m❌ 'syn' found in dependency tree. Here's what's using 'syn':\033[0m"
        cargo tree -i syn -e features
        exit 1
    fi

build:
    cargo build --workspace --all-features

check:
    cargo check --workspace --all-features

clippy:
    cargo clippy --workspace --all-targets --all-features --target-dir target/clippy-all-features -- -D warnings

clippy-fix:
    cargo clippy --fix --workspace --all-targets --all-features --target-dir target/clippy-all-features -- -D warnings

test *args:
    cargo nextest run {{args}} --no-fail-fast --all-features

test-ci *args:
    #!/usr/bin/env -S echo-comment --color bright-green
    # 🏃 Running Rust tests...
    cargo test {{args}} --all-features

    # 📚 Running documentation tests...
    cargo test --doc {{args}} --all-features
