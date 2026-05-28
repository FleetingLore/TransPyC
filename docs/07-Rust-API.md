# Rust API 参考

本章面向将 TransPyC 作为 Rust 库使用的开发者。

## 引入

```toml
[dependencies]
trans_py_c = "0.1"
```

## Translator

翻译器的核心结构体，持有翻译过程中的所有状态。

```rust
use trans_py_c::Translator;

let mut t = Translator::new();
```

### generate_c_code()

```rust
pub fn generate_c_code(&mut self, source: &str) -> String
```

解析 Python 源码并生成 C 代码。这是唯一的对外翻译入口。

```rust
let mut t = Translator::new();
let c_code = t.generate_c_code("def add(a: int, b: int) -> int: return a + b")
    .expect("翻译失败");
```

**流程**：解析 AST → 收集符号 → 按序生成 C 代码。

**错误处理**：解析失败时返回 `/* Parse error: ... */` 作为注释嵌入输出。

### 翻译后状态

翻译完成后，以下字段可供读取：

```rust
// 符号表: 记录所有 class/function/全局变量
t.symbol_table     // HashMap<String, SymbolKind>

// 调试日志 (以 [SCOPE] [ENTER] [EXIT] 标记)
t.debug_logs       // Vec<String>

// 函数返回类型记录
t.function_return_types  // HashMap<String, String>

// 变量作用域栈 (翻译完应为空)
t.var_scopes       // Vec<HashMap<String, String>>
```

## SymbolKind

```rust
pub enum SymbolKind {
    Variable {
        declared_type: String,  // C 类型名, 如 "int", "struct TASK*"
        is_pointer: bool,       // 是否是指针类型
    },
    Function,
    Struct {
        members: HashMap<String, MemberInfo>,
    },
}
```

### MemberInfo

```rust
pub struct MemberInfo {
    pub type_name: String,    // 成员 C 类型, 如 "int", "struct Node*"
    pub is_pointer: bool,     // 成员是否指针 (影响 . vs -> 选择)
}
```

### 使用示例

```rust
use trans_py_c::{Translator, SymbolKind};

let mut t = Translator::new();
t.generate_c_code(r#"
class Point:
    x: int
    y: int
def main() -> int:
    return 0
"#).expect("翻译失败");

// 读取符号表
if let Some(SymbolKind::Struct { members }) = t.symbol_table.get("Point") {
    for (name, info) in members {
        println!("  {}: type={} ptr={}", name, info.type_name, info.is_pointer);
    }
}
```

## includes 模块

底层的 C 代码生成模块，一般不需要直接使用，但可用于扩展翻译器。

### gramma — c 模块

```rust
use trans_py_c::includes::gramma;

gramma::asm_inline("nop")            // → __asm__ volatile ("nop");
gramma::memory_addr("4096")          // → ((void *)4096)
gramma::macro_define("MAX", "100")   // → #define MAX 100
gramma::type_cast("float", "x")      // → ((float)x)
gramma::addr_of("x")                 // → &x
gramma::ptr_deref("p")               // → *(p)
gramma::ptr_write("4096", Some("42")) // → *((void *)4096) = 42;
```

### types — t 模块

```rust
use trans_py_c::includes::types;

types::c_name("CInt")       // → "int"
types::c_name("CPtr")       // → "*"
types::c_name("CStatic")    // → "static"
types::struct_type("Task")  // → "struct Task"
types::is_storage_class("static")  // → true
```

## CLI 模块

命令行接口，用于构建自定义工具。

```rust
use trans_py_c::cli::{Args, collect_jobs, handle_init};
use clap::Parser;

let args = Args::parse();
let jobs = collect_jobs(&args);
for job in &jobs {
    // job.input:  PathBuf  (.py 文件路径)
    // job.output: PathBuf  (.c 输出路径)
}
```

> `cli` 模块仅在 binary crate 中可用，library crate 不包含 clap 相关依赖。
