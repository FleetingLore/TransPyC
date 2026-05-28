# TransPyC 文档

## 快速开始

```bash
trans_py_c init                 # 初始化项目
trans_py_c -c examples/hello -v # 翻译
```

## 翻译样例

| 样例 | 说明 |
|------|------|
| [hello](https://github.com/fleetinglore/TransPyC/blob/main/examples/hello/main.py) | 最简入口：`return 42` |
| [variables](https://github.com/fleetinglore/TransPyC/blob/main/examples/variables/main.py) | 变量声明：`int`, `str`, `float` |
| [struct](https://github.com/fleetinglore/TransPyC/blob/main/examples/struct/main.py) | `class → struct` + 成员访问 |
| [control_flow](https://github.com/fleetinglore/TransPyC/blob/main/examples/control_flow/main.py) | `if/elif` + `for range` |
| [test_simple](https://github.com/fleetinglore/TransPyC/blob/main/examples/test_simple/main.py) | class + method + constructor |
| [pointer](https://github.com/fleetinglore/TransPyC/blob/main/examples/pointer/main.py) | `c.Memory`, `c.Addr`, `c.Cast` |
| [asm_macro](https://github.com/fleetinglore/TransPyC/blob/main/examples/asm_macro/main.py) | `c.Asm`, `c.Macro` |
| [example1](https://github.com/fleetinglore/TransPyC/blob/main/examples/example1/main.py) | 全覆盖：所有特性组合 |

```bash
trans_py_c -c examples/pointer -v
```
