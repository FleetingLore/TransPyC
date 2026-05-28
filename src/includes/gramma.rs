//! C 语法操作 — 对应 Python 的 `c` 模块
//!
//! Python 代码中 `c.Asm(...)` `c.Memory(...)` 等调用最终通过
//! 本模块生成对应的 C 代码字符串。
//!
//! # 设计
//!
//! 每个函数对应一个 `c.*` 操作，输入是已翻译好的 C 表达式字符串，
//! 输出是完整的 C 代码片段。调用方（`core::expressions`）负责
//! 从 Python AST 中提取参数并递归翻译。
//!
//! # 示例
//!
//! ```text
//! // Python: c.Memory(0x1000)
//! gramma::memory_addr("4096")  // → "((void *)4096)"
//!
//! // Python: c.Asm('nop')
//! gramma::asm_inline("nop")   // → "__asm__ volatile (\"nop\");"
//! ```

/// `c.Asm(...)` — 内联汇编
pub fn asm_inline(code: &str) -> String {
    let lines: Vec<&str> = code.trim().lines().collect();
    if lines.len() > 1 {
        format!(
            "__asm__ volatile (\n        \"{}\"\n    );",
            lines.join("\\n\\t\"\n        \"")
        )
    } else {
        format!("__asm__ volatile (\"{}\");", code)
    }
}

/// `c.Memory(addr)` — 字面地址指针
pub fn memory_addr(addr: &str) -> String {
    format!("((void *){})", addr)
}

/// `c.Macro(name, value)` — #define
pub fn macro_define(name: &str, value: &str) -> String {
    format!("#define {} {}", name, value)
}

/// `c.TypeCast(type_name, value)` — C 强制类型转换
pub fn type_cast(type_name: &str, value: &str) -> String {
    format!("(({}){})", type_name, value)
}

/// `c.Addr(expr)` — 取地址 &
pub fn addr_of(expr: &str) -> String {
    format!("&{}", expr)
}

/// `c.Cast(ptr)` — 解引用 *
pub fn ptr_deref(ptr: &str) -> String {
    format!("*({})", ptr)
}

/// `c.Ptr(addr)` — 指针转换 / 赋值
pub fn ptr_write(addr: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("*((void *){}) = {};", addr, v),
        None => format!("((void *){})", addr),
    }
}
