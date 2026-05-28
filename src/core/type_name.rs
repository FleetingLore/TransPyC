//! 类型名称解析 — 将 Python 类型注解 AST 转换为 C 类型字符串
//!
//! # 核心函数: `get_type_name()`
//!
//! 递归遍历 Python 类型注解 AST 节点，输出对应的 C 类型名。
//!
//! Python 类型注解通过 `|` 运算符组合：
//!
//! ```text
//! t.CInt                        → "int"
//! t.CPtr | t.CInt               → "int*"
//! t.CStatic | t.CInt            → "static int"
//! t.CStruct(name="Point")       → "struct Point"
//! t.CPtr | t.CStruct(name="X") | t.CInt[10] → "struct X*[10]"? (复合)
//! ```
//!
//! # 节点类型处理
//!
//! | AST 节点 | 示例 | 处理方式 |
//! |----------|------|---------|
//! | `Name` | `int`, `float`, `CInt` | 查 Python 内置 / types 映射 |
//! | `Attribute` | `t.CInt`, `c.State` | 区分模块来源 |
//! | `Call` | `t.CStruct(name="X")` | 解析关键字参数 |
//! | `Subscript` | `t.CChar[16]` | 递归基类型 + 提取数组大小 |
//! | `BinOp(BitOr)` | `t.CInt \| t.CPtr` | 分别解析左右侧后组合 |
//!
//! # 指针 vs 结构体的歧义
//!
//! `CPtr` 返回 `"*"` 而不是 C 关键字。当和结构体名组合时
//! (`struct Foo | t.CPtr`)，`resolve_bitor_type` 负责重排为
//! `"struct Foo*"` 而非 `"struct Foo *"`。
//!
//! # 组合规则 (`resolve_bitor_type`)
//!
//! 左右两侧的类型字符串按优先级合并：
//! 1. 存储修饰符优先: `static | int` → `"static int"`
//! 2. 指针置后: `int | *` → `"int*"`, `struct Foo | *` → `"struct Foo*"`
//! 3. 类型合并: `long | int` → `"long int"`
//! 4. 数组指针: `const char[16] | (*)` → `"const char (*)[16]"`

use rustpython_parser::ast::{self, Constant, Expr as PyExpr, Operator};

use super::translator::Translator;

impl Translator {
    /// 获取类型名称
    pub fn get_type_name(&self, node: &PyExpr) -> String {
        match node {
            PyExpr::Constant(c) if matches!(&c.value, Constant::None) => "void".to_string(),
            PyExpr::Name(name) => self.resolve_name_type(name.id.as_str()),
            PyExpr::Attribute(attr) => self.resolve_attr_type(&attr.value, attr.attr.as_str()),
            PyExpr::Call(call) => self.resolve_call_type(&call.func, &call.args, &call.keywords),
            PyExpr::Subscript(sub) => self.resolve_subscript_type(&sub.value, &sub.slice),
            PyExpr::BinOp(binop) if matches!(&binop.op, Operator::BitOr) => {
                self.resolve_bitor_type(&binop.left, &binop.right)
            }
            _ => "int".to_string(),
        }
    }

    fn resolve_name_type(&self, id: &str) -> String {
        if let Some(c_type) = super::types::lookup_type(id) {
            return c_type.to_string();
        }
        let builtin = match id {
            "int" => "int",
            "str" => "char*",
            "bool" => "bool",
            "float" => "float",
            "double" => "double",
            "list" => "void*",
            "dict" => "void*",
            "set" => "void*",
            "tuple" => "void*",
            "None" => "void",
            _ => "",
        };
        if !builtin.is_empty() {
            return builtin.to_string();
        }

        let basic = match id {
            "CChar" => "char",
            "CUnsignedChar" => "unsigned char",
            "CInt" => "int",
            "CUnsignedInt" => "unsigned int",
            "CShort" => "short",
            "CUnsignedShort" => "unsigned short",
            "CLong" => "long",
            "CUnsignedLong" => "unsigned long",
            "CFloat" => "float",
            "CDouble" => "double",
            "CVoid" => "void",
            "CPtr" => "*",
            _ => "",
        };
        if !basic.is_empty() {
            return basic.to_string();
        }

        format!("struct {}", id)
    }

    fn resolve_attr_type(&self, value: &PyExpr, attr: &str) -> String {
        if let PyExpr::Name(name) = value {
            if name.id.as_str() == "t" {
                if let Some(c_type) = super::types::lookup_type(attr) {
                    return c_type.to_string();
                }
                return attr.to_string();
            }
            if name.id.as_str() == "c" {
                return format!("c.{}", attr);
            }
        }
        "int".to_string()
    }

    fn resolve_call_type(
        &self,
        func: &PyExpr,
        args: &[PyExpr],
        keywords: &[ast::Keyword],
    ) -> String {
        if let PyExpr::Attribute(attr) = func {
            if let PyExpr::Name(name) = attr.value.as_ref() {
                if name.id.as_str() == "t" {
                    let mut kwargs = std::collections::HashMap::new();
                    for kw in keywords {
                        if let Some(arg_name) = &kw.arg {
                            if let ast::Expr::Constant(c) = &kw.value {
                                if let Constant::Str(s) = &c.value {
                                    kwargs.insert(arg_name.to_string(), s.clone());
                                }
                            }
                        }
                    }

                    match attr.attr.as_str() {
                        "CStruct" => {
                            let n = kwargs.get("name").map(|s| s.as_str()).unwrap_or("");
                            if n.is_empty() {
                                if let Some(first) = args.first() {
                                    if let PyExpr::Name(n2) = first {
                                        return format!("struct {}", n2.id);
                                    }
                                }
                                "struct".to_string()
                            } else {
                                format!("struct {}", n)
                            }
                        }
                        _ => {
                            if let Some(c_type) = super::types::lookup_type(attr.attr.as_str()) {
                                c_type.to_string()
                            } else {
                                attr.attr.to_string()
                            }
                        }
                    }
                } else {
                    "int".to_string()
                }
            } else {
                "int".to_string()
            }
        } else {
            "int".to_string()
        }
    }

    fn resolve_subscript_type(&self, value: &PyExpr, slice: &PyExpr) -> String {
        let base_type = self.get_type_name(value);
        if base_type.is_empty() || base_type == "int" {
            return "int".to_string();
        }
        let size = self.handle_expr(slice);
        let size_str = if size.is_empty() { "" } else { &size[0] };
        format!("{}[{}]", base_type, size_str)
    }

    fn resolve_bitor_type(&self, left: &PyExpr, right: &PyExpr) -> String {
        let left_type = self.get_type_name(left);
        let right_type = self.get_type_name(right);

        if left_type == "static" || left_type == "extern" {
            return format!("{} {}", left_type, right_type);
        }
        if right_type == "static" || right_type == "extern" {
            return format!("{} {}", right_type, left_type);
        }
        if left_type == "const" || left_type == "volatile" {
            return format!("{} {}", left_type, right_type);
        }
        if right_type == "const" || right_type == "volatile" {
            return format!("{} {}", right_type, left_type);
        }
        if left_type == "*" {
            return format!("{}*", right_type);
        }
        if right_type == "*" {
            return format!("{}*", left_type);
        }
        if left_type == "struct" && !right_type.starts_with("struct ") {
            return format!("struct {}", right_type);
        }
        if right_type == "struct" && !left_type.starts_with("struct ") {
            return format!("struct {}", left_type);
        }
        if (left_type == "long" && right_type == "int")
            || (left_type == "int" && right_type == "long")
        {
            return "long int".to_string();
        }
        if (left_type == "unsigned int" && right_type == "long")
            || (left_type == "long" && right_type == "unsigned int")
        {
            return "unsigned long".to_string();
        }
        if right_type == "(*)" {
            if let Some(pos) = left_type.find('[') {
                return format!("{} (*){}", &left_type[..pos], &left_type[pos..]);
            }
            return format!("{} (*)", left_type);
        }
        if left_type == "(*)" {
            if let Some(pos) = right_type.find('[') {
                return format!("{} (*){}", &right_type[..pos], &right_type[pos..]);
            }
            return format!("{} (*)", right_type);
        }
        if left_type.starts_with("struct ") && right_type == "*" {
            return format!("{}*", left_type);
        }
        if right_type.starts_with("struct ") && left_type == "*" {
            return format!("{}*", right_type);
        }
        format!("{} {}", left_type, right_type)
    }
}
