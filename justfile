default: 
    just --list

doc: 
    cargo doc --document-private-items --open

run: 
    cargo run -- -b bios.bin

test: 
    cargo test
