//! 类型系统与符号表

use std::collections::HashMap;

/// 符号表条目
#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable {
        declared_type: String,
        is_pointer: bool,
    },
    Function,
    Struct {
        members: HashMap<String, MemberInfo>,
    },
}

/// 结构体成员信息
#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub type_name: String,
    pub is_pointer: bool,
}

/// 符号表: 变量/函数/结构体名称 → 类型信息
pub type SymbolTable = HashMap<String, SymbolKind>;

/// 变量作用域栈
pub type VarScopes = Vec<HashMap<String, String>>;

/// 函数返回类型记录
pub type FunctionReturnTypes = HashMap<String, String>;

/// C 类型映射 (Python 类型名 → C 类型名)
pub fn lookup_type(type_name: &str) -> Option<&'static str> {
    crate::constants::TYPE_MAP
        .iter()
        .find(|(key, _)| *key == type_name)
        .map(|(_, val)| *val)
}

/// 检查是否是基本类型
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

/// 提取数组大小: "char[16]" → ("char", "[16]")
/// "int[3][4]" → ("int", "[3][4]")
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

/// 检查并提取存储类修饰符
/// "static int" → ("static", "int")
/// "int" → ("", "int")
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

/// 构建数组初始化代码
pub fn build_array_initialization(elements: &[String]) -> String {
    format!("{{ {} }}", elements.join(", "))
}
