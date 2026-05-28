//! 表达式处理 — Python AST 节点 → C 字符串
//!
//! # 核心函数: `handle_expr()`
//!
//! 翻译器中最复杂的部分。接收任意 Python 表达式 AST 节点，
//! 返回对应的 C 代码字符串数组（通常是单元素，多元素用于
//! `print()` 等多行语句）。
//!
//! # 节点 → C 映射
//!
//! | Python AST | C 输出 |
//! |------------|--------|
//! | `Constant(42)` | `"42"` |
//! | `Name("x")` | `"x"` |
//! | `BinOp(Add, a, b)` | `"(a + b)"` |
//! | `BoolOp(And, a, b)` | `"a && b"` |
//! | `Compare(a > b)` | `"a > b"` |
//! | `Call(func, [a])` | `"func(a)"` |
//! | `Attribute(x, "field")` | `"x.field"` 或 `"x->field"` |
//! | `IfExp(test, a, b)` | `"(test ? a : b)"` |
//!
//! # 特殊调用
//!
//! `c.*` → `handle_c_special_call()` → 委托 `includes::gramma`
//! `t.*` → `handle_t_special_call()` → 委托 `includes::types`
//! `obj.method()` → 转换 `structName__method(&obj, ...)`
//!
//! # `.` vs `->` 选择
//!
//! `handle_attribute()` 是 `.` vs `->` 的决策核心：
//! - `self.field` → `self->field` (self 总是指针)
//! - 从 VarScopes / SymbolTable 查变量是否声明为指针
//! - 指针变量 → `->`，非指针 → `.`

use rustpython_parser::ast::{self, Constant, Expr as PyExpr};

use super::translator::Translator;
use crate::includes::gramma;

/// 判断是否是 c 模块的已知操作（用于区分模块调用和普通方法调用）
fn is_c_special(attr: &str) -> bool {
    matches!(
        attr,
        "Asm" | "Memory" | "Set" | "TypeCast" | "Macro" | "Addr" | "Ptr" | "Cast"
    )
}

impl Translator {
    /// 处理表达式，返回 C 代码行
    pub fn handle_expr(&self, node: &PyExpr) -> Vec<String> {
        match node {
            PyExpr::Constant(c) => self.handle_constant(&c.value),
            PyExpr::Name(name) => match name.id.as_str() {
                "True" => vec!["1".to_string()],
                "False" => vec!["0".to_string()],
                "None" => vec!["0".to_string()],
                _ => vec![name.id.to_string()],
            },
            PyExpr::BinOp(binop) => {
                let l = self.handle_expr(&binop.left);
                let r = self.handle_expr(&binop.right);
                let op_sym = self.get_op_symbol(&binop.op);
                vec![format!("({} {} {})", l[0], op_sym, r[0])]
            }
            PyExpr::BoolOp(boolop) => {
                let parts: Vec<String> = boolop
                    .values
                    .iter()
                    .map(|v| self.handle_expr(v)[0].clone())
                    .collect();
                match &boolop.op {
                    ast::BoolOp::And => vec![parts.join(" && ")],
                    ast::BoolOp::Or => vec![parts.join(" || ")],
                }
            }
            PyExpr::UnaryOp(unary) => {
                let oper = self.handle_expr(&unary.operand);
                let op_sym = self.get_unary_op_symbol(&unary.op);
                vec![format!("{}{}", op_sym, oper[0])]
            }
            PyExpr::Call(call) => self.handle_call(&call.func, &call.args, &call.keywords),
            PyExpr::Subscript(sub) => {
                // 后置自增 (k, k:=k+1)[0]
                if let PyExpr::Tuple(tup) = sub.value.as_ref() {
                    if tup.elts.len() == 2 {
                        if let PyExpr::Name(n0) = &tup.elts[0] {
                            if let PyExpr::NamedExpr(named) = &tup.elts[1] {
                                if let PyExpr::Name(tn) = named.target.as_ref() {
                                    if n0.id == tn.id {
                                        if let PyExpr::BinOp(b) = named.value.as_ref() {
                                            if matches!(&b.op, ast::Operator::Add) {
                                                if let PyExpr::Name(ln) = b.left.as_ref() {
                                                    if ln.id == tn.id {
                                                        if let PyExpr::Constant(rc) =
                                                            b.right.as_ref()
                                                        {
                                                            if let Constant::Int(val) = &rc.value {
                                                                if val.to_string() == "1" {
                                                                    if let PyExpr::Constant(ic) =
                                                                        sub.slice.as_ref()
                                                                    {
                                                                        if let Constant::Int(idx) =
                                                                            &ic.value
                                                                        {
                                                                            if idx.to_string()
                                                                                == "0"
                                                                            {
                                                                                return vec![
                                                                                    format!(
                                                                                        "{}++",
                                                                                        n0.id
                                                                                    ),
                                                                                ];
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let val = self.handle_expr(&sub.value);
                let idx = self.handle_expr(&sub.slice);
                if val.is_empty() || idx.is_empty() {
                    return vec!["0".to_string()];
                }
                vec![format!("{}[{}]", val[0], idx[0])]
            }
            PyExpr::Tuple(tup) => {
                let parts: Vec<String> = tup
                    .elts
                    .iter()
                    .map(|e| self.handle_expr(e)[0].clone())
                    .collect();
                vec![format!("({})", parts.join(", "))]
            }
            PyExpr::List(list) => {
                let parts: Vec<String> = list
                    .elts
                    .iter()
                    .map(|e| self.handle_expr(e)[0].clone())
                    .collect();
                vec![format!("{{{}}}", parts.join(", "))]
            }
            PyExpr::Set(set) => {
                let parts: Vec<String> = set
                    .elts
                    .iter()
                    .map(|e| self.handle_expr(e)[0].clone())
                    .collect();
                vec![format!("{{{}}}", parts.join(", "))]
            }
            PyExpr::Compare(comp) => {
                let mut comparisons = Vec::new();
                let mut left_val = self.handle_expr(&comp.left)[0].clone();
                for (i, op) in comp.ops.iter().enumerate() {
                    let cmp = self.get_comparator_symbol(op);
                    let right_val = self.handle_expr(&comp.comparators[i])[0].clone();
                    comparisons.push(format!("{} {} {}", left_val, cmp, right_val));
                    left_val = right_val;
                }
                if comparisons.len() == 1 {
                    comparisons
                } else {
                    vec![comparisons.join(" && ")]
                }
            }
            PyExpr::Attribute(attr) => self.handle_attribute(&attr.value, &attr.attr),
            PyExpr::IfExp(ifexp) => {
                let t = self.handle_expr(&ifexp.test);
                let b = self.handle_expr(&ifexp.body);
                let o = self.handle_expr(&ifexp.orelse);
                vec![format!("({} ? {} : {})", t[0], b[0], o[0])]
            }
            PyExpr::Starred(starred) => {
                // 指针解引用表达式: *ptr → *((void *)ptr)
                let ptr = self.handle_expr(&starred.value);
                if ptr.is_empty() {
                    return vec!["0".to_string()];
                }
                vec![format!("*((void *){})", ptr[0])]
            }
            PyExpr::NamedExpr(named) => {
                // 前置自增 k := k + 1
                if let PyExpr::BinOp(binop) = named.value.as_ref() {
                    if matches!(&binop.op, ast::Operator::Add) {
                        if let PyExpr::Name(ln) = binop.left.as_ref() {
                            if let PyExpr::Name(tn) = named.target.as_ref() {
                                if ln.id == tn.id {
                                    if let PyExpr::Constant(c) = binop.right.as_ref() {
                                        if let Constant::Int(val) = &c.value {
                                            if val.to_string() == "1" {
                                                return vec![format!("++{}", tn.id)];
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                let t = self.handle_expr(&named.target);
                let v = self.handle_expr(&named.value);
                vec![format!("(({} = {}), {})", t[0], v[0], t[0])]
            }
            _ => vec!["0".to_string()],
        }
    }

    fn handle_constant(&self, value: &Constant) -> Vec<String> {
        match value {
            Constant::Str(s) => vec![format!("\"{}\"", s)],
            Constant::Bool(b) => vec![if *b { "1" } else { "0" }.to_string()],
            Constant::Int(i) => vec![i.to_string()],
            Constant::Float(f) => {
                let s = format!("{}", f);
                if !s.contains('.') {
                    vec![format!("{}.0", s)]
                } else {
                    vec![s]
                }
            }
            Constant::Complex { real, imag } => vec![format!("({} + {}*I)", real, imag)],
            Constant::None => vec!["0".to_string()],
            Constant::Ellipsis => vec!["...".to_string()],
            Constant::Bytes(b) => vec![format!("\"{}\"", String::from_utf8_lossy(b))],
            Constant::Tuple(tup) => {
                let parts: Vec<String> = tup
                    .iter()
                    .map(|c| self.handle_constant(c)[0].clone())
                    .collect();
                vec![format!("({})", parts.join(", "))]
            }
        }
    }

    fn handle_call(
        &self,
        func: &PyExpr,
        args: &[PyExpr],
        keywords: &[ast::Keyword],
    ) -> Vec<String> {
        match func {
            PyExpr::Attribute(attr) => {
                if let PyExpr::Name(name) = attr.value.as_ref() {
                    if name.id.as_str() == "c" && is_c_special(attr.attr.as_str()) {
                        return self.handle_c_special_call(attr.attr.as_str(), args, keywords);
                    }
                    if name.id.as_str() == "t" {
                        return self.handle_t_special_call(attr.attr.as_str(), args, keywords);
                    }
                }
                let obj = self.handle_expr(&attr.value);
                if obj.is_empty() {
                    return vec!["0".to_string()];
                }
                if let Some(sname) = self.find_struct_name(&attr.value) {
                    let func_name = format!("{}__{}", sname, attr.attr);
                    let mut ma = vec![format!("&{}", obj[0])];
                    for arg in args {
                        let a = self.handle_expr(arg);
                        if !a.is_empty() {
                            ma.push(a[0].clone());
                        }
                    }
                    vec![format!("{}({})", func_name, ma.join(", "))]
                } else {
                    vec![format!("{}.{}()", obj[0], attr.attr)]
                }
            }
            PyExpr::Name(name) => {
                let fname = name.id.as_str();
                let fa: Vec<String> = args
                    .iter()
                    .map(|a| self.handle_expr(a)[0].clone())
                    .collect();
                let as_ = fa.join(", ");
                match fname {
                    "len" => {
                        if !fa.is_empty() {
                            vec![format!("(sizeof({}) / sizeof({}[0]))", fa[0], fa[0])]
                        } else {
                            vec!["0".to_string()]
                        }
                    }
                    "sizeof" => {
                        if !fa.is_empty() {
                            vec![format!("sizeof({})", fa[0])]
                        } else {
                            vec!["0".to_string()]
                        }
                    }
                    "print" => {
                        let mut lines = Vec::new();
                        if !fa.is_empty() {
                            if fa[0].starts_with('"') && fa[0].ends_with('"') {
                                lines.push(format!("printf({});", fa[0]));
                                lines.push("printf(\"\\n\");".to_string());
                            } else {
                                lines.push(format!("printf(\"%d\\n\", {});", as_));
                            }
                        } else {
                            lines.push("printf(\"\\n\");".to_string());
                        }
                        lines
                    }
                    _ => vec![format!("{}({})", fname, as_)],
                }
            }
            _ => vec!["0".to_string()],
        }
    }

    fn handle_attribute(&self, value: &PyExpr, attr: &str) -> Vec<String> {
        if let PyExpr::Name(name) = value {
            if name.id.as_str() == "c" {
                return vec![format!("c.{}", attr)];
            }
        }
        let mut current = value;
        let mut chain: Vec<String> = vec![attr.to_string()];
        loop {
            match current {
                PyExpr::Attribute(ia) => {
                    chain.push(ia.attr.to_string());
                    current = &ia.value;
                }
                PyExpr::Name(name) => {
                    let base = name.id.to_string();
                    let mut is_ptr = base == "self";
                    if !is_ptr {
                        for scope in self.var_scopes.iter().rev() {
                            if let Some(t) = scope.get(&base) {
                                if t.contains('*') {
                                    is_ptr = true;
                                }
                                break;
                            }
                        }
                    }
                    if !is_ptr {
                        if let Some(sym) = self.symbol_table.get(&base) {
                            if let super::types::SymbolKind::Variable { is_pointer, .. } = sym {
                                is_ptr = *is_pointer;
                            }
                        }
                    }
                    chain.reverse();
                    let mut result = base;
                    for m in &chain {
                        if is_ptr {
                            result.push_str("->");
                        } else {
                            result.push('.');
                        }
                        result.push_str(m);
                    }
                    return vec![result];
                }
                _ => {
                    let base = self.handle_expr(current);
                    if base.is_empty() {
                        return vec!["0".to_string()];
                    }
                    chain.reverse();
                    let mut result = base[0].clone();
                    for m in &chain {
                        result.push('.');
                        result.push_str(m);
                    }
                    return vec![result];
                }
            }
        }
    }

    fn find_struct_name(&self, expr: &PyExpr) -> Option<String> {
        match expr {
            PyExpr::Name(name) => {
                let id = name.id.as_str();
                // 先从符号表查找（全局变量/结构体）
                if let Some(sym) = self.symbol_table.get(id) {
                    if let super::types::SymbolKind::Variable { declared_type, .. } = sym {
                        if declared_type.starts_with("struct ") {
                            return declared_type
                                .split(' ')
                                .nth(1)
                                .map(|n| n.trim_end_matches('*').to_string());
                        }
                    }
                }
                // 再从作用域查找（局部变量）
                for scope in self.var_scopes.iter().rev() {
                    if let Some(t) = scope.get(id) {
                        if t.starts_with("struct ") {
                            return t
                                .split(' ')
                                .nth(1)
                                .map(|n| n.trim_end_matches('*').to_string());
                        }
                    }
                }
                None
            }
            PyExpr::Call(call) => {
                if let PyExpr::Name(n) = call.func.as_ref() {
                    Some(n.id.to_string())
                } else if let PyExpr::Attribute(a) = call.func.as_ref() {
                    Some(a.attr.to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // ── 特殊调用 ──

    fn handle_c_special_call(
        &self,
        attr: &str,
        args: &[PyExpr],
        _keywords: &[ast::Keyword],
    ) -> Vec<String> {
        match attr {
            "Asm" => {
                if let Some(PyExpr::Constant(c)) = args.first() {
                    if let Constant::Str(code) = &c.value {
                        return vec![gramma::asm_inline(code)];
                    }
                }
                vec![gramma::asm_inline("nop")]
            }
            "Memory" => {
                if let Some(a) = args
                    .first()
                    .and_then(|a| self.handle_expr(a).into_iter().next())
                {
                    return vec![gramma::memory_addr(&a)];
                }
                vec![gramma::memory_addr("0")]
            }
            "Set" => {
                if args.len() >= 2 {
                    let t = self.handle_expr(&args[0]);
                    let v = self.handle_expr(&args[1]);
                    if !t.is_empty() && !v.is_empty() {
                        return vec![format!("{} = {};", t[0], v[0])];
                    }
                }
                Vec::new()
            }
            "TypeCast" => {
                if args.len() >= 2 {
                    let tn = match &args[0] {
                        PyExpr::Constant(c) if let Constant::Str(s) = &c.value => s.clone(),
                        _ => "void".to_string(),
                    };
                    if let Some(v) = self.handle_expr(&args[1]).into_iter().next() {
                        return vec![gramma::type_cast(&tn, &v)];
                    }
                }
                vec![gramma::type_cast("void", "0")]
            }
            "Macro" => {
                if args.len() >= 2 {
                    let n = match &args[0] {
                        PyExpr::Constant(c) if let Constant::Str(s) = &c.value => s.clone(),
                        _ => "MACRO".to_string(),
                    };
                    let v = match &args[1] {
                        PyExpr::Constant(c) if let Constant::Str(s) = &c.value => s.clone(),
                        _ => "0".to_string(),
                    };
                    return vec![gramma::macro_define(&n, &v)];
                }
                Vec::new()
            }
            "Addr" => {
                if let Some(a) = args
                    .first()
                    .and_then(|a| self.handle_expr(a).into_iter().next())
                {
                    return vec![gramma::addr_of(&a)];
                }
                vec!["0".to_string()]
            }
            "Ptr" => {
                if let Some(a) = args
                    .first()
                    .and_then(|a| self.handle_expr(a).into_iter().next())
                {
                    let v = args
                        .get(1)
                        .and_then(|a| self.handle_expr(a).into_iter().next());
                    return vec![gramma::ptr_write(&a, v.as_deref())];
                }
                vec![gramma::ptr_write("0", None)]
            }
            "Cast" => {
                if let Some(a) = args
                    .first()
                    .and_then(|a| self.handle_expr(a).into_iter().next())
                {
                    return vec![gramma::ptr_deref(&a)];
                }
                vec!["0".to_string()]
            }
            _ => {
                let as_: Vec<String> = args
                    .iter()
                    .map(|a| self.handle_expr(a)[0].clone())
                    .collect();
                vec![format!("c.{}({});", attr, as_.join(", "))]
            }
        }
    }

    fn handle_t_special_call(
        &self,
        attr: &str,
        args: &[PyExpr],
        _keywords: &[ast::Keyword],
    ) -> Vec<String> {
        if args.is_empty() {
            return vec!["0".to_string()];
        }
        let val = match self.handle_expr(&args[0]).into_iter().next() {
            Some(v) => v,
            None => return vec!["0".to_string()],
        };
        match attr {
            "CType" => {
                let types: Vec<String> = args[1..]
                    .iter()
                    .map(|a| self.get_type_name(a))
                    .filter(|t| !t.is_empty())
                    .collect();
                if types.is_empty() {
                    vec![val]
                } else {
                    vec![format!("(({}){})", types.join(" "), val)]
                }
            }
            "CStruct" => {
                let sn = if args.len() >= 2 {
                    if let PyExpr::Name(n) = &args[1] {
                        n.id.to_string()
                    } else {
                        "BOOTINFO".to_string()
                    }
                } else {
                    "BOOTINFO".to_string()
                };
                vec![format!("((struct {} *){})", sn, val)]
            }
            _ => {
                let mut tp = Vec::new();
                if let Some(ct) = super::types::lookup_type(attr) {
                    tp.push(ct.to_string());
                } else {
                    tp.push(attr.to_string());
                }
                for a in &args[1..] {
                    let t = self.get_type_name(a);
                    if !t.is_empty() {
                        tp.push(t);
                    }
                }
                vec![format!("(({}){})", tp.join(" "), val)]
            }
        }
    }
}
