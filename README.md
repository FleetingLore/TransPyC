# TransPyC

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rust-lang.org)

Python 子集 → C 源码翻译器，面向操作系统内核等底层开发场景。

## 快速示例

**Python 输入**:

```python
def Add(a: int, b: int) -> int:
    return a + b

def TestIf(x: int) -> int:
    if x > 0:
        return x + 1
    elif x < 0:
        return x - 1
    else:
        return 0
```

**生成的 C 输出**:

```c
int Add(int a, int b) {
    return (a + b);
}
int TestIf(int x) {
    if (x > 0) { return (x + 1); }
    else if (x < 0) { return (x - 1); }
    else { return 0; }
}
```

## 构建

```bash
cargo build --release
```

## 使用

```bash
# 初始化新项目
trans_py_c init                   # 在当前目录
trans_py_c init my_project        # 在指定目录

# 翻译（读取 TransPyC.toml）
trans_py_c -c examples/example1
trans_py_c -c examples/example1 -v   # 详细输出

# 直接指定文件（无需配置）
trans_py_c main.py -o out/
trans_py_c "src/*.py" -o out/        # glob

# 调试
trans_py_c main.py --debug

# 帮助
trans_py_c --help
trans_py_c init --help
```

### 项目配置 (`TransPyC.toml`)

```toml
name = "my_project"
output = "target"           # 输出到 ./target/

[[files]]
input = "*.py"              # glob 模式

[[files]]
input = "subdir/*.py"
output = "subdir/target"    # 覆盖全局 output
```

## 项目结构

```
TransPyC/
├── src/
│   ├── main.rs               # 二进制入口
│   ├── lib.rs                 # 库入口 + 架构文档
│   ├── cli/mod.rs             # CLI (clap) + TOML 配置
│   ├── core/                  # 核心翻译器
│   │   ├── translator.rs      # 翻译流程总控
│   │   ├── expressions.rs     # 表达式 → C 代码
│   │   ├── type_name.rs       # 类型注解 → C 类型名
│   │   └── types.rs           # 符号表 / 作用域
│   ├── includes/              # c 模块 + t 模块
│   │   ├── gramma.rs          # Asm, Memory, Macro, ...
│   │   └── types.rs           # CInt, CPtr, CStruct, ...
│   └── constants/             # 运算符映射
├── examples/                  # 翻译样例 (每个子目录 = 一个项目)
│   ├── example1/              # 综合示例
│   └── test_simple/           # 简单综合测试
├── tests/                     # 测试 (18 单元 + 2 文件)
└── docs/                      # 文档
```

## 测试

```bash
cargo test                          # 全部 27 个
cargo test --test translator_test   # 18 个单元测试
cargo test --test file_test         # 8 个文件对比测试
```

### 添加新的翻译样例

```bash
# 1. 初始化项目
trans_py_c init my_example

# 2. 写 Python 代码
vim my_example/main.py

# 3. 翻译（输出到 my_example/target/）
trans_py_c -c my_example

# 4. 保存期望输出用于测试
cp my_example/target/main.c my_example/expected.c

# 5. 在 tests/file_test.rs 中添加:
#    #[test]
#    fn test_my_example() { file_test("my_example"); }
```

## 支持的特性

- 函数 / 结构体 (`class`) / 方法
- 控制流: `if/elif/else`, `for range`, `while`, `do-while` (检测)
- 类型: `int`, `float`, `char`, `bool`, `static`, `extern`, `const`
- 内联汇编 (`c.Asm`), 宏 (`c.Macro`), 指针 (`c.Memory`, `c.Ptr`)
- 三目运算符, 逻辑运算符 (`and`→`&&`, `or`→`||`)
- 数组 / 列表初始化

## 依赖

- [rustpython-parser](https://crates.io/crates/rustpython-parser) — Python AST 解析
- [clap](https://crates.io/crates/clap) — 命令行解析
- [toml](https://crates.io/crates/toml) + [serde](https://crates.io/crates/serde) — 配置文件
- [glob](https://crates.io/crates/glob) — 文件匹配
