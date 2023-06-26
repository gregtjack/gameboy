default: 
    just --list

doc: 
    cargo doc --document-private-items --open

run: 
    cargo run -- --help

test: 
    cargo test
