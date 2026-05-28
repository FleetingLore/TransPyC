#!/bin/sh
# 重新生成所有 examples 的 expected.c
set -e
cd "$(dirname "$0")/.."

for dir in examples/*/; do
    name=$(basename "$dir")
    echo "  $name"
    cargo run -- -c "examples/$name" > /dev/null
    cp "examples/$name/target/main.c" "examples/$name/expected.c"
done
echo "完成"
