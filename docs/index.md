# TransPyC 文档

欢迎使用 TransPyC！TransPyC 是一个将 **Python 子集代码转换成 C 代码** 的翻译器，面向操作系统内核等底层开发场景。

## 快速开始

```bash
# 初始化一个项目
trans_py_c init

# 翻译（使用 TransPyC.toml 配置）
trans_py_c -c examples/example1 -v

# 直接翻译单个文件
trans_py_c main.py -o out/
```

## 文档列表

### 1. [概述与原理](01-概述与原理.md)

- 项目概述与设计目标
- 翻译流程（解析 → 收集符号 → 生成 C）
- 架构概览（src/core/、src/includes/）
- 符号表、变量作用域
- `.` vs `->` 运算符选择
- CLI 使用方法

### 2. [变量与类型](02-变量与类型.md)

- 基本类型（int, char, float, ...）
- 类型注解语法（`name: type`）
- 指针类型（`t.CPtr`）
- 数组类型（`t.CChar[N]`）
- 存储修饰符（static, extern, const, volatile）
- 类型转换（`c.TypeCast`, `t.CInt(x)`）

### 3. [控制流](03-控制流.md)

- if / elif / else
- for 循环（`range(n)` → C for）
- while 循环
- do-while 检测（`while True` + `break`）
- 三目运算符

### 4. [函数与结构体](04-函数与结构体.md)

- 函数定义（`def name(params) -> type:`）
- 结构体（`class` → `struct`）
- 方法调用（`obj.method()` → `struct_method(&obj)`）
- 构造函数（`__init__`）

### 5. [内联汇编与宏](05-内联汇编与宏.md)

- `c.Asm()` 内联汇编
- `c.Macro()` → `#define`
- 头文件包含（`import` → `#include`）
- 指针操作（`c.Memory`, `c.Addr`, `c.Ptr`, `c.Cast`）

### 6. [高级特性与最佳实践](06-高级特性与最佳实践.md)

- 项目配置（TransPyC.toml）
- 文件组织方式
- 与 C 代码互操作
- 调试技巧
- 常见问题

### 7. [Rust API 参考](07-Rust-API.md)

- Translator 结构体与 `generate_c_code()`
- 符号表（SymbolKind, MemberInfo）
- includes 模块（gramma, types）
- CLI 模块
