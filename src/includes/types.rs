// src/includes/types.rs

/// 支持的值类型
#[derive(Debug, Clone)]
pub enum CValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Char(u8),
    String(String),
    Bool(bool),
    Pointer(usize),
    None,
}

/// C 类型基类
#[derive(Debug, Clone)]
pub struct CType {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: String,
}

impl CType {
    pub fn new(value: Option<CValue>, types: Vec<CType>) -> Self {
        Self {
            value,
            types,
            c_name: String::new(),
        }
    }

    pub fn merge(&self, types: Vec<CType>) -> Vec<CType> {
        types
    }

    pub fn or(&self, other: Self) -> Vec<Self> {
        vec![self.clone(), other]
    }
}

/// 定义 C 类型宏
macro_rules! define_c_type {
    ($name:ident, $c_name:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            pub value: Option<CValue>,
            pub types: Vec<CType>,
            pub c_name: &'static str,
        }

        impl $name {
            pub fn new(value: Option<CValue>) -> Self {
                Self {
                    value,
                    types: Vec::new(),
                    c_name: $c_name,
                }
            }
        }
    };
}

// 使用宏定义所有类型
define_c_type!(CChar, "char");
define_c_type!(CInt, "int");
define_c_type!(CShort, "short");
define_c_type!(CLong, "long");
define_c_type!(CFloat, "float");
define_c_type!(CDouble, "double");
define_c_type!(CVoid, "void");
define_c_type!(CUnsigned, "unsigned");
define_c_type!(CUnsignedChar, "unsigned char");
define_c_type!(CUnsignedInt, "unsigned int");
define_c_type!(CUnsignedShort, "unsigned short");
define_c_type!(CUnsignedLong, "unsigned long");
define_c_type!(CSignedChar, "signed char");

/// C struct 类型
#[derive(Debug, Clone)]
pub struct CStruct {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: String,
    pub name: Option<String>,
}

impl CStruct {
    pub fn new(value: Option<CValue>, name: Option<String>) -> Self {
        let c_name = match &name {
            Some(n) => format!("struct {}", n),
            None => "struct".to_string(),
        };
        Self {
            value,
            types: Vec::new(),
            c_name,
            name,
        }
    }
}

/// C union 类型
#[derive(Debug, Clone)]
pub struct CUnion {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUnion {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "union",
        }
    }
}

/// C enum 类型
#[derive(Debug, Clone)]
pub struct CEnum {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CEnum {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "enum",
        }
    }
}

/// C typedef 类型
#[derive(Debug, Clone)]
pub struct CTypedef {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CTypedef {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "typedef",
        }
    }
}

/// C auto 类型
#[derive(Debug, Clone)]
pub struct CAuto {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CAuto {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "auto",
        }
    }
}

/// C register 类型
#[derive(Debug, Clone)]
pub struct CRegister {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CRegister {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "register",
        }
    }
}

/// C static 类型
#[derive(Debug, Clone)]
pub struct CStatic {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CStatic {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "static",
        }
    }
}

/// C extern 类型
#[derive(Debug, Clone)]
pub struct CExtern {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CExtern {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "extern",
        }
    }
}

/// C const 类型
#[derive(Debug, Clone)]
pub struct CConst {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CConst {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "const",
        }
    }
}

/// C volatile 类型
#[derive(Debug, Clone)]
pub struct CVolatile {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CVolatile {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "volatile",
        }
    }
}

/// C size_t 类型
#[derive(Debug, Clone)]
pub struct CSizeT {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CSizeT {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "size_t",
        }
    }
}

/// C int8_t 类型
#[derive(Debug, Clone)]
pub struct CInt8T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CInt8T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "int8_t",
        }
    }
}

/// C int16_t 类型
#[derive(Debug, Clone)]
pub struct CInt16T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CInt16T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "int16_t",
        }
    }
}

/// C int32_t 类型
#[derive(Debug, Clone)]
pub struct CInt32T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CInt32T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "int32_t",
        }
    }
}

/// C int64_t 类型
#[derive(Debug, Clone)]
pub struct CInt64T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CInt64T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "int64_t",
        }
    }
}

/// C uint8_t 类型
#[derive(Debug, Clone)]
pub struct CUInt8T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUInt8T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "uint8_t",
        }
    }
}

/// C uint16_t 类型
#[derive(Debug, Clone)]
pub struct CUInt16T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUInt16T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "uint16_t",
        }
    }
}

/// C uint32_t 类型
#[derive(Debug, Clone)]
pub struct CUInt32T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUInt32T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "uint32_t",
        }
    }
}

/// C uint64_t 类型
#[derive(Debug, Clone)]
pub struct CUInt64T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUInt64T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "uint64_t",
        }
    }
}

/// C intptr_t 类型
#[derive(Debug, Clone)]
pub struct CIntPtrT {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CIntPtrT {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "intptr_t",
        }
    }
}

/// C uintptr_t 类型
#[derive(Debug, Clone)]
pub struct CUIntPtrT {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CUIntPtrT {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "uintptr_t",
        }
    }
}

/// C ptrdiff_t 类型
#[derive(Debug, Clone)]
pub struct CPtrDiffT {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CPtrDiffT {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "ptrdiff_t",
        }
    }
}

/// C wchar_t 类型
#[derive(Debug, Clone)]
pub struct CWCharT {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CWCharT {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "wchar_t",
        }
    }
}

/// C char16_t 类型
#[derive(Debug, Clone)]
pub struct CChar16T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CChar16T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "char16_t",
        }
    }
}

/// C char32_t 类型
#[derive(Debug, Clone)]
pub struct CChar32T {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CChar32T {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "char32_t",
        }
    }
}

/// C bool 类型
#[derive(Debug, Clone)]
pub struct CBool {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CBool {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "bool",
        }
    }
}

/// C _Complex 类型
#[derive(Debug, Clone)]
pub struct CComplex {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CComplex {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "_Complex",
        }
    }
}

/// C _Imaginary 类型
#[derive(Debug, Clone)]
pub struct CImaginary {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CImaginary {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "_Imaginary",
        }
    }
}

/// C 指针类型
#[derive(Debug, Clone)]
pub struct CPtr {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CPtr {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "*",
        }
    }
}

/// 数组指针类型，用于声明指向数组的指针，如 char (*ptr)[16]
#[derive(Debug, Clone)]
pub struct CArrayPtr {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CArrayPtr {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "(*)",
        }
    }
}

/// C #define 类型
#[derive(Debug, Clone)]
pub struct CDefine {
    pub value: Option<CValue>,
    pub types: Vec<CType>,
    pub c_name: &'static str,
}

impl CDefine {
    pub fn new(value: Option<CValue>) -> Self {
        Self {
            value,
            types: Vec::new(),
            c_name: "#define",
        }
    }
}
