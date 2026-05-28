//! 数据类型 —— 符号表、作用域、工具函数
//!
//! # 核心数据结构
//!
//! ## SymbolTable
//!
//! 全局符号表，记录所有已知的变量、函数、结构体。
//! 第一遍扫描时填充，后续翻译时查询。
//!
//! ```text
//! "MyStruct" → Struct { members: { "x": {type:"int", is_ptr:false}, ... } }
//! "my_func"  → Function
//! "global_x" → Variable { declared_type: "int", is_pointer: false }
//! ```
//!
//! 主要用途：
//! - 变量声明时判断已存在还是新声明
//! - 成员访问时判定用 `.` 还是 `->`
//! - 类型推断（从符号表查找变量的 C 类型）
//!
//! ## VarScopes
//!
//! 栈式作用域，模拟 C 的 `{}` 块作用域。
//!
//! ```text
//! [ { global vars },          ← 全局作用域
//!   { func params },           ← 函数参数
//!   { local vars } ]           ← 函数体局部变量
//! ```
//!
//! 每进入一个函数 push 一层，每退出 pop 一层。
//! 查找变量时从栈顶向下搜索（内层覆盖外层）。

use std::collections::HashMap;

/// 符号表条目
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// 变量: 记录 C 类型和是否是指针
    Variable {
        declared_type: String,
        is_pointer: bool,
    },
    /// 函数: 仅标记存在
    Function,
    /// 结构体: 记录成员信息
    Struct {
        members: HashMap<String, MemberInfo>,
    },
}

/// 结构体成员信息
#[derive(Debug, Clone)]
pub struct MemberInfo {
    /// 成员的 C 类型名 (如 `"int"`, `"struct Foo*"`)
    pub type_name: String,
    /// 是否是指针类型 (影响后续成员的 `.` vs `->` 选择)
    pub is_pointer: bool,
}

/// 符号表: 变量/函数/结构体名称 → 类型信息
pub type SymbolTable = HashMap<String, SymbolKind>;

/// 变量作用域栈: 每层是一个 name→C类型 的映射
///
/// 例如 `[{"x": "int", "p": "struct Point*"}, {"y": "char"}]`
pub type VarScopes = Vec<HashMap<String, String>>;

/// 函数→返回类型 的记录
///
/// 用于推断 `x = some_func()` 中 x 的类型
pub type FunctionReturnTypes = HashMap<String, String>;

/// 将 Python 类型名 (如 "CInt") 映射为 C 类型名 (如 "int")
///
/// 委托给 `includes::types::c_name()`。
/// 只对确认为 TransPyC 类型的名称返回 Some，
/// 普通 Python 名称（如自定义 class 名）返回 None。
pub fn lookup_type(type_name: &str) -> Option<&'static str> {
    let c = crate::includes::types::c_name(type_name);
    // c_name 对未知名称默认返回 "int"，
    // 只有确实是已知类型或 \"CInt\" 本身才返回 Some
    if c != "int" || type_name == "CInt" {
        Some(c)
    } else {
        None
    }
}

/// 判断一个 C 类型名是否是基本类型
///
/// 基本类型不需要 `struct` 前缀。
pub fn is_basic_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "int"
            | "char"
            | "float"
            | "double"
            | "void"
            | "long"
            | "short"
            | "unsigned int"
            | "unsigned char"
            | "unsigned long"
            | "unsigned short"
            | "signed char"
            | "size_t"
            | "int8_t"
            | "int16_t"
            | "int32_t"
            | "int64_t"
            | "uint8_t"
            | "uint16_t"
            | "uint32_t"
            | "uint64_t"
            | "intptr_t"
            | "uintptr_t"
            | "ptrdiff_t"
            | "wchar_t"
            | "char16_t"
            | "char32_t"
            | "bool"
            | "_Complex"
            | "_Imaginary"
    )
}

/// 从复合类型名中拆分数组大小
///
/// ```text
/// "char[16]"    → ("char", "[16]")
/// "int[3][4]"   → ("int", "[3][4]")
/// "struct Foo*" → ("struct Foo*", "")
/// ```
///
/// C 语言数组声明需要把数组大小放在变量名后面
/// (如 `char buf[16]`)，此函数用于拆分以便重组。
pub fn extract_array_size(type_name: &str) -> (String, String) {
    let mut array_sizes = Vec::new();
    let mut base = type_name.to_string();

    loop {
        if let Some(start) = base.rfind('[') {
            if let Some(end) = base.rfind(']') {
                let size = &base[start + 1..end];
                array_sizes.push(size.to_string());
                base = base[..start].to_string();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let array_str: String = array_sizes
        .iter()
        .rev()
        .map(|s| format!("[{}]", s))
        .collect();
    (base, array_str)
}

/// 从类型名中拆分存储类修饰符
///
/// ```text
/// "static int"     → ("static", "int")
/// "extern char[16]" → ("extern", "char[16]")
/// "int"            → ("", "int")
/// ```
///
/// 修饰符（static/extern）需要放在类型名前面。
pub fn check_storage_class(type_name: &str) -> (String, String) {
    let trimmed = type_name.trim();
    for &sc in crate::constants::STORAGE_CLASSES.iter() {
        if trimmed.starts_with(sc) {
            let rest = trimmed[sc.len()..].trim().to_string();
            return (sc.to_string(), rest);
        }
    }
    (String::new(), trimmed.to_string())
}

/// 构建 C 数组初始化列表
///
/// 输入 `["1","2","3"]` → 输出 `"{ 1, 2, 3 }"`
pub fn build_array_initialization(elements: &[String]) -> String {
    format!("{{ {} }}", elements.join(", "))
}
