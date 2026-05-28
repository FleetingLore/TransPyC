//! C 类型系统 — 对应 Python 的 `t` 模块
//!
//! 提供 Python 类型名到 C 类型名的映射。`core::type_name` 通过
//! `get_type_name()` 递归解析 Python 类型注解 AST 节点，最终调用
//! 本模块的 `c_name()` 获取叶子类型的 C 名称。
//!
//! # 示例
//!
//! ```text
//! types::c_name("CInt")   // → "int"
//! types::c_name("CPtr")   // → "*"
//! types::c_name("CStatic") // → "static"
//! types::struct_type("Foo") // → "struct Foo"
//! ```

/// 给定 Python 类型名 (例: "CInt")，返回对应的 C 类型名 (例: "int")
pub fn c_name(py_type: &str) -> &'static str {
    match py_type {
        "CChar" => "char",
        "CUnsignedChar" => "unsigned char",
        "CSignedChar" => "signed char",
        "CInt" => "int",
        "CUnsignedInt" => "unsigned int",
        "CShort" => "short",
        "CUnsignedShort" => "unsigned short",
        "CLong" => "long",
        "CUnsignedLong" => "unsigned long",
        "CLongLong" => "long long",
        "CFloat" => "float",
        "CDouble" => "double",
        "CVoid" => "void",
        "CBool" => "bool",
        "CSizeT" => "size_t",
        "CInt8T" => "int8_t",
        "CInt16T" => "int16_t",
        "CInt32T" => "int32_t",
        "CInt64T" => "int64_t",
        "CUInt8T" => "uint8_t",
        "CUInt16T" => "uint16_t",
        "CUInt32T" => "uint32_t",
        "CUInt64T" => "uint64_t",
        "CIntPtrT" => "intptr_t",
        "CUIntPtrT" => "uintptr_t",
        "CPtrDiffT" => "ptrdiff_t",
        "CWCharT" => "wchar_t",
        "CChar16T" => "char16_t",
        "CChar32T" => "char32_t",
        "CComplex" => "_Complex",
        "CImaginary" => "_Imaginary",
        "CPtr" => "*",
        "CArrayPtr" => "(*)",
        "CDefine" => "#define",
        "CStatic" => "static",
        "CExtern" => "extern",
        "CConst" => "const",
        "CVolatile" => "volatile",
        "CAuto" => "auto",
        "CRegister" => "register",
        "CTypedef" => "typedef",
        "CUnion" => "union",
        "CEnum" => "enum",
        _ => "int",
    }
}

/// 判断是否为存储类修饰符 (static, extern, const, volatile 等)
pub fn is_storage_class(s: &str) -> bool {
    matches!(
        s,
        "static" | "extern" | "const" | "volatile" | "auto" | "register"
    )
}

/// 生成 `t.CStruct(name="Foo")` 对应的 C 类型名
pub fn struct_type(name: &str) -> String {
    if name.is_empty() {
        "struct".to_string()
    } else {
        format!("struct {}", name)
    }
}
