[default]
run input output:
    cargo run --release -- -i {{input}} -o {{output}}

readme:
    uv run ./scripts/gen_readme.py

download-examples:
    uv run ./scripts/download_examples.py

regen-examples:
    uv run ./scripts/regen_examples.py
