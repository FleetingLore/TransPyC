//! TransPyC —— Python 子集 → C 源码翻译器
//!
//! # 架构概览
//!
//! ```text
//!                        main.rs
//!                          │
//!                          ▼
//!               ┌──────────────────┐
//!               │   Translator     │  core/translator.rs
//!               │                  │
//!               │ generate_c_code()│ ← 入口
//!               └──────┬───────────┘
//!                      │
//!          ┌───────────┼───────────┐
//!          │           │           │
//!          ▼           ▼           ▼
//!   expressions.rs  type_name.rs  types.rs
//!   (表达式→C代码)   (类型名→C类型)  (符号表/作用域)
//!          │           │
//!          └─────┬─────┘
//!                │
//!     ┌──────────┴──────────┐
//!     │                     │
//!     ▼                     ▼
//! includes/gramma.rs   includes/types.rs
//! (c.Asm / c.Memory / …)  (t.CInt / t.CPtr / …)
//! ```
//!
//! # 翻译流程
//!
//! 1. **解析** — `rustpython-parser` 将 Python 源码转为 AST
//! 2. **收集符号** — 遍历 AST，提取 class/function/全局变量
//! 3. **生成** —— 按序处理每个 AST 节点，产出 C 代码行
//!
//! # 关键概念
//!
//! ## 符号表 (`SymbolTable`)
//!
//! 存储 class→struct、函数名、全局变量及其类型信息。用于后续的
//! `.` vs `->` 运算符选择、类型推断等。
//!
//! ## 变量作用域 (`VarScopes`)
//!
//! 栈结构，每进入一个函数 push 一层，退出时 pop。
//! 用于判断变量是否已声明（避免重复 `int x = ...`）。
//!
//! ## c 模块 / t 模块
//!
//! Python 代码通过 `import c` 和 `import t` 引入特殊语法。
//! `c.*` 对应底层 C 操作（内联汇编、指针、宏），
//! `t.*` 对应 C 类型系统（int、char、struct、指针修饰符等）。
//! Rust 侧实现在 `src/includes/` 中。
//!
//! # 示例
//!
//! ```python
//! # 输入 (Python)
//! def add(a: int, b: int) -> int:
//!     return a + b
//! ```
//!
//! ```c
//! // 输出 (C)
//! int add(int a, int b) {
//!     return (a + b);
//! }
//! ```

/// 常量映射：运算符、比较符、存储修饰符等
pub mod constants;

/// 核心翻译器 —— AST 遍历 → C 代码生成
pub mod core;

/// C 语言概念：gramma = `c.*` 操作, types = `t.*` 类型系统
pub mod includes;
