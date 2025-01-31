test *FLAGS:
    cargo nextest run {{FLAGS}}

clippy *FLAGS:
    cargo clippy --workspace {{FLAGS}} 

build *FLAGS:
    cargo check --workspace {{FLAGS}} 

check *FLAGS:
    cargo check --workspace {{FLAGS}} 

todo:
    rg "todo|FIX|FIXME|TODO|HACK|WARN|PERF|NOTE|TEST" ./basm*

cov:
    cargo llvm-cov --html
