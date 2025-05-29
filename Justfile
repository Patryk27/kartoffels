run *args:
    cd app \
    && cargo run --release -- \
           serve \
           ../store \
           --admin 127.0.0.1:1312 \
           --open-admin \
           --http 127.0.0.1:1313 \
           --ssh 127.0.0.1:1314 \
           {{ args }}

run-vg *args:
    cd app \
    && cargo build --release \
    && cd .. \
    && valgrind --tool=callgrind --dump-instr=yes --collect-jumps=yes \
          ./app/target/release/kartoffels \
           serve \
           ./store \
           --admin 127.0.0.1:1312 \
           --open-admin \
           --http 127.0.0.1:1313 \
           --ssh 127.0.0.1:1314 \
           {{ args }}

toolbox *args:
    cd app \
    && cargo run --release -- \
           toolbox \
           {{ args }}

web:
    cd web \
    && npm run dev

# ---

check:
    cd app \
    && cargo check --workspace \
    && cargo check --workspace --tests \
    && cargo clippy --workspace \
    && cargo clippy --workspace --tests \
    && cargo doc -p kartoffel

clean:
    cd app \
    && cargo clean

doc *args:
    cd app \
    && cargo doc -p kartoffel {{ args }}

fmt:
    cd app \
    && cargo fmt

test:
    cd app \
    && cargo test --release --workspace

test-and-bless:
    BLESS=1 just test

bless:
    fd -e new --no-ignore-vcs --full-path --exec mv {} {.}

perf:
    cd app \
    && cargo build --release \
    && perf record --call-graph dwarf \
           ./target/release/kartoffels \
               serve \
               ../store \
               --debug \
               --bench \
               --ssh 127.0.0.1:1314

# ---

create-demo:
    sudo nixos-container create demo --flake .
    sudo nixos-container start demo
    @echo
    @echo "ready:"
    @echo "http://$(nixos-container show-ip demo)"

update-demo:
    sudo nixos-container update demo --flake .
    sudo nixos-container start demo
    @echo
    @echo "ready:"
    @echo "http://$(nixos-container show-ip demo)"

stop-demo:
    sudo nixos-container stop demo
