//! 类型名称解析 (GetTypeName)
//!
//! 将 Python 类型注解 (t.CInt, t.CPtr, t.CStatic | t.CInt 等)
//! 转换为 C 类型名称字符串。

use rustpython_parser::ast::{self, Constant, Expr as PyExpr, Operator};

use super::translator::Translator;

impl Translator {
    /// 获取类型名称
    /// 处理 t.CInt, t.CPtr, t.CStatic | t.CInt, t.CStruct(name="X"), 等
    pub fn get_type_name(&self, node: &PyExpr) -> String {
        match node {
            PyExpr::Name { id, .. } => self.resolve_name_type(id),
            PyExpr::Attribute { value, attr, .. } => self.resolve_attr_type(value, attr),
            PyExpr::Call {
                func,
                args,
                keywords,
                ..
            } => self.resolve_call_type(func, args, keywords),
            PyExpr::Subscript { value, slice, .. } => self.resolve_subscript_type(value, slice),
            PyExpr::BinOp {
                left, op, right, ..
            } => {
                if matches!(op.as_ref(), Operator::BitOr) {
                    self.resolve_bitor_type(left, right)
                } else {
                    "int".to_string()
                }
            }
            _ => "int".to_string(),
        }
    }

    // ── 各类型解析 ──

    fn resolve_name_type(&self, id: &str) -> String {
        // 优先检查 TYPE_MAP 中的类型
        if let Some(c_type) = super::types::lookup_type(id) {
            return c_type.to_string();
        }

        // 处理 Python 内置类型
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

        // 检查是否是基本类型名 (CChar, CInt, ...)
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

        // 普通名称视为结构体名: struct Name
        format!("struct {}", id)
    }

    fn resolve_attr_type(&self, value: &PyExpr, attr: &str) -> String {
        if let PyExpr::Name { id, .. } = value {
            if id == "t" {
                // t.CInt, t.CPtr 等
                if let Some(c_type) = super::types::lookup_type(attr) {
                    return c_type.to_string();
                }
                return attr.to_string();
            }
            if id == "c" {
                // c.State 等
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
        if let PyExpr::Attribute { value, attr, .. } = func {
            if let PyExpr::Name { id, .. } = value.as_ref() {
                if id == "t" {
                    // t.CStruct(name="X") 等
                    let mut kwargs = std::collections::HashMap::new();
                    for kw in keywords {
                        if let ast::Expr::Constant { value, .. } = &kw.node.value {
                            if let Constant::Str(s) = value {
                                kwargs.insert(kw.node.arg.clone(), s.clone());
                            }
                        }
                    }

                    match attr.as_str() {
                        "CStruct" => {
                            let name = kwargs.get("name").map(|s| s.as_str()).unwrap_or("");
                            if name.is_empty() {
                                // 从位置参数获取
                                if let Some(first) = args.first() {
                                    if let PyExpr::Name { id, .. } = first {
                                        return format!("struct {}", id);
                                    }
                                }
                                "struct".to_string()
                            } else {
                                format!("struct {}", name)
                            }
                        }
                        _ => {
                            if let Some(c_type) = super::types::lookup_type(attr) {
                                c_type.to_string()
                            } else {
                                attr.clone()
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

        // 处理存储类修饰符
        // static | int → static int
        if left_type == "static" || left_type == "extern" {
            return format!("{} {}", left_type, right_type);
        }
        if right_type == "static" || right_type == "extern" {
            return format!("{} {}", right_type, left_type);
        }

        // 处理 const/volatile
        if left_type == "const" || left_type == "volatile" {
            return format!("{} {}", left_type, right_type);
        }
        if right_type == "const" || right_type == "volatile" {
            return format!("{} {}", right_type, left_type);
        }

        // 处理指针类型 char | * → char*
        if left_type == "*" {
            return format!("{}*", right_type);
        }
        if right_type == "*" {
            return format!("{}*", left_type);
        }

        // 处理 struct 与具体名称的组合
        if left_type == "struct" && !right_type.starts_with("struct ") {
            return format!("struct {}", right_type);
        }
        if right_type == "struct" && !left_type.starts_with("struct ") {
            return format!("struct {}", left_type);
        }

        // 处理 long | int → long int
        if (left_type == "long" && right_type == "int")
            || (left_type == "int" && right_type == "long")
        {
            return "long int".to_string();
        }

        // 处理 unsigned int | long → unsigned long
        if (left_type == "unsigned int" && right_type == "long")
            || (left_type == "long" && right_type == "unsigned int")
        {
            return "unsigned long".to_string();
        }

        // 处理数组指针 const char[16] | (*) → const char (*)[16]
        if right_type == "(*)" {
            if let Some(pos) = left_type.find('[') {
                let type_part = &left_type[..pos];
                let array_part = &left_type[pos..];
                return format!("{} (*){}", type_part, array_part);
            }
            return format!("{} (*)", left_type);
        }
        if left_type == "(*)" {
            if let Some(pos) = right_type.find('[') {
                let type_part = &right_type[..pos];
                let array_part = &right_type[pos..];
                return format!("{} (*){}", type_part, array_part);
            }
            return format!("{} (*)", right_type);
        }

        // 处理 struct X | * → struct X*
        if left_type.starts_with("struct ") && right_type == "*" {
            return format!("{}*", left_type);
        }
        if right_type.starts_with("struct ") && left_type == "*" {
            return format!("{}*", right_type);
        }

        // 普通组合
        format!("{} {}", left_type, right_type)
    }
}
