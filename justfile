@default: 
    just --list

doc: 
    cargo doc --document-private-items --open

run: 
    cargo run -- -b bios.bin

test: 
    cargo test

doctor TEST:
    cargo run --release -- -d --rom 'test/{{TEST}}' > test.log
    sed -i '1,2d' test.log