# TransPyC 翻译样例

每个子目录是一个独立项目，包含：

```
example1/
├── TransPyC.toml    # 项目配置
├── main.py           # Python 输入
├── expected.c        # 期望的 C 输出（自动测试用）
├── c.py              # c 模块桩 (需要时才存在)
└── t.py              # t 模块桩 (需要时才存在)
```

## 运行

```bash
trans_py_c -c examples/hello -v
```

## 样例列表

| 目录 | 行数 | 说明 | 用 c/t |
|------|------|------|--------|
| `hello/` | 5 | 最简入口：`main → return` | — |
| `variables/` | 9 | 变量声明：`int`, `str`, `float` | — |
| `struct/` | 13 | `class → struct` + 成员访问 | — |
| `control_flow/` | 16 | `if/elif` + `for range` | — |
| `test_simple/` | 22 | 综合：class + method + constructor | — |
| `pointer/` | 10 | `c.Memory`, `c.Addr`, `c.Cast` | c |
| `asm_macro/` | 9 | `c.Asm`, `c.Macro` | c |
| `example1/` | 71 | 全覆盖：所有特性组合 | c + t |

## 测试

修改翻译器后：

```bash
sh scripts/regenerate.sh          # 重新生成所有 expected.c
cargo test                         # 运行 27 个测试
```
