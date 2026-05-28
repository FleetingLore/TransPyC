//! 表达式处理 (HandleExpr 及特殊函数调用)

use rustpython_parser::ast::{self, CmpOp, Constant, Expr as PyExpr, Operator, UnaryOp};

use super::translator::Translator;

impl Translator {
    /// 处理表达式，返回 C 代码行
    pub fn handle_expr(&self, node: &PyExpr) -> Vec<String> {
        match node {
            PyExpr::Constant { value, .. } => self.handle_constant(value),
            PyExpr::Name { id, .. } => match id.as_str() {
                "True" => vec!["1".to_string()],
                "False" => vec!["0".to_string()],
                "None" => vec!["0".to_string()],
                _ => vec![id.clone()],
            },
            PyExpr::BinOp {
                left, op, right, ..
            } => {
                let l = self.handle_expr(left);
                let r = self.handle_expr(right);
                let op_sym = self.get_op_symbol(op);
                vec![format!("({} {} {})", l[0], op_sym, r[0])]
            }
            PyExpr::BoolOp { op, values, .. } => {
                let parts: Vec<String> = values
                    .iter()
                    .map(|v| self.handle_expr(v)[0].clone())
                    .collect();
                match op {
                    ast::BoolOp::And => vec![parts.join(" && ")],
                    ast::BoolOp::Or => vec![parts.join(" || ")],
                }
            }
            PyExpr::UnaryOp { op, operand, .. } => {
                let oper = self.handle_expr(operand);
                let op_sym = self.get_unary_op_symbol(op);
                vec![format!("{}{}", op_sym, oper[0])]
            }
            PyExpr::Call {
                func,
                args,
                keywords,
            } => self.handle_call(func, args, keywords),
            PyExpr::Subscript { value, slice, .. } => {
                // 检查后置自增模式 (k, k:=k+1)[0]
                if let PyExpr::Tuple { elts, .. } = value.as_ref() {
                    if elts.len() == 2 {
                        if let PyExpr::Name { id: elt0_name, .. } = &elts[0] {
                            if let PyExpr::NamedExpr {
                                target, value: nv, ..
                            } = &elts[1]
                            {
                                if let PyExpr::Name {
                                    id: target_name, ..
                                } = target.as_ref()
                                {
                                    if elt0_name == target_name {
                                        if let PyExpr::BinOp {
                                            left,
                                            op: binop,
                                            right,
                                            ..
                                        } = nv.as_ref()
                                        {
                                            if matches!(binop.as_ref(), Operator::Add) {
                                                if let PyExpr::Name { id: left_name, .. } =
                                                    left.as_ref()
                                                {
                                                    if left_name == target_name {
                                                        if let PyExpr::Constant {
                                                            value: cv, ..
                                                        } = right.as_ref()
                                                        {
                                                            if let Constant::Int(val) = cv {
                                                                if *val == 1 {
                                                                    if let PyExpr::Constant {
                                                                        value: idx_val,
                                                                        ..
                                                                    } = slice.as_ref()
                                                                    {
                                                                        if let Constant::Int(idx) =
                                                                            idx_val
                                                                        {
                                                                            if *idx == 0 {
                                                                                return vec![
                                                                                    format!(
                                                                                        "{}++",
                                                                                        elt0_name
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

                let val = self.handle_expr(value);
                let idx = self.handle_expr(slice);
                if val.is_empty() || idx.is_empty() {
                    return vec!["0".to_string()];
                }
                vec![format!("{}[{}]", val[0], idx[0])]
            }
            PyExpr::Tuple { elts, .. } => {
                let parts: Vec<String> = elts
                    .iter()
                    .map(|e| self.handle_expr(e)[0].clone())
                    .collect();
                vec![format!("({})", parts.join(", "))]
            }
            PyExpr::List { elts, .. } | PyExpr::Set { elts, .. } => {
                let parts: Vec<String> = elts
                    .iter()
                    .map(|e| self.handle_expr(e)[0].clone())
                    .collect();
                vec![format!("{{{}}}", parts.join(", "))]
            }
            PyExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                let mut comparisons = Vec::new();
                let mut left_val = self.handle_expr(left)[0].clone();
                for (i, op) in ops.iter().enumerate() {
                    let cmp = self.get_comparator_symbol(op);
                    let right_val = self.handle_expr(&comparators[i])[0].clone();
                    comparisons.push(format!("{} {} {}", left_val, cmp, right_val));
                    left_val = right_val;
                }
                if comparisons.len() == 1 {
                    comparisons
                } else {
                    vec![comparisons.join(" && ")]
                }
            }
            PyExpr::Attribute { value, attr, .. } => self.handle_attribute(value, attr),
            PyExpr::IfExp {
                test, body, orelse, ..
            } => {
                let t = self.handle_expr(test);
                let b = self.handle_expr(body);
                let o = self.handle_expr(orelse);
                vec![format!("({} ? {} : {})", t[0], b[0], o[0])]
            }
            PyExpr::NamedExpr { target, value, .. } => {
                // 海象运算符: 检查是否是前置自增 k := k + 1
                if let PyExpr::BinOp {
                    left,
                    op: binop,
                    right,
                    ..
                } = value.as_ref()
                {
                    if matches!(binop.as_ref(), Operator::Add) {
                        if let PyExpr::Name { id: left_name, .. } = left.as_ref() {
                            if let PyExpr::Name {
                                id: target_name, ..
                            } = target.as_ref()
                            {
                                if left_name == target_name {
                                    if let PyExpr::Constant { value: cv, .. } = right.as_ref() {
                                        if let Constant::Int(val) = cv {
                                            if *val == 1 {
                                                return vec![format!("++{}", target_name)];
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let t = self.handle_expr(target);
                let v = self.handle_expr(value);
                vec![format!("(({} = {}), {})", t[0], v[0], t[0])]
            }
            _ => vec!["0".to_string()],
        }
    }

    fn handle_constant(&self, value: &Constant) -> Vec<String> {
        match value {
            Constant::Str(s) => {
                // 尝试从原始代码中获取带引号的字符串字面量
                vec![format!("\"{}\"", s)]
            }
            Constant::Bool(b) => vec![if *b { "1".to_string() } else { "0".to_string() }],
            Constant::Int(i) => vec![i.to_string()],
            Constant::Float(f) => {
                // 尝试保留原始格式
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
        }
    }

    fn handle_call(
        &self,
        func: &PyExpr,
        args: &[PyExpr],
        keywords: &[ast::Keyword],
    ) -> Vec<String> {
        match func {
            // c.* 调用
            PyExpr::Attribute { value, attr, .. } => {
                if let PyExpr::Name { id, .. } = value.as_ref() {
                    if id == "c" {
                        return self.handle_c_special_call(attr, args, keywords);
                    }
                    if id == "t" {
                        return self.handle_t_special_call(attr, args, keywords);
                    }
                }
                // 普通对象方法调用: obj.method(args)
                let obj = self.handle_expr(value);
                let method = attr;
                let mut method_args = Vec::new();

                // 查找结构体名
                let struct_name = self.find_struct_name(value);

                if let Some(sname) = &struct_name {
                    let func_name = format!("{}__{}", sname, method);
                    method_args.push(format!("&{}", obj[0]));
                    for arg in args {
                        let a = self.handle_expr(arg);
                        if !a.is_empty() {
                            method_args.push(a[0].clone());
                        }
                    }
                    vec![format!("{}({})", func_name, method_args.join(", "))]
                } else {
                    vec![format!("{}.{}()", obj[0], method)]
                }
            }
            // 普通函数调用
            PyExpr::Name { id, .. } => {
                let func_name = id;
                let func_args: Vec<String> = args
                    .iter()
                    .map(|a| self.handle_expr(a)[0].clone())
                    .collect();
                let args_str = func_args.join(", ");

                match func_name.as_str() {
                    "len" => {
                        if !func_args.is_empty() {
                            vec![format!(
                                "(sizeof({}) / sizeof({}[0]))",
                                func_args[0], func_args[0]
                            )]
                        } else {
                            vec!["0".to_string()]
                        }
                    }
                    "sizeof" => {
                        if !func_args.is_empty() {
                            vec![format!("sizeof({})", func_args[0])]
                        } else {
                            vec!["0".to_string()]
                        }
                    }
                    "print" => {
                        let mut lines = Vec::new();
                        if !func_args.is_empty() {
                            let first = &func_args[0];
                            if first.starts_with('"') && first.ends_with('"') {
                                lines.push(format!("printf({});", first));
                                lines.push("printf(\"\\n\");".to_string());
                            } else {
                                lines.push(format!("printf(\"%d\\n\", {});", args_str));
                            }
                        } else {
                            lines.push("printf(\"\\n\");".to_string());
                        }
                        lines
                    }
                    _ => vec![format!("{}({})", func_name, args_str)],
                }
            }
            _ => vec!["0".to_string()],
        }
    }

    fn handle_attribute(&self, value: &PyExpr, attr: &str) -> Vec<String> {
        // 处理 c.State / t.CInt 形式
        if let PyExpr::Name { id, .. } = value {
            if id == "c" {
                return vec![format!("c.{}", attr)];
            }
        }

        // 构建访问链
        let mut parts = Vec::new();
        let mut current = value;
        let mut chain: Vec<String> = Vec::new();
        chain.push(attr.to_string());

        loop {
            match current {
                PyExpr::Attribute {
                    value: inner,
                    attr: inner_attr,
                    ..
                } => {
                    chain.push(inner_attr.clone());
                    current = inner;
                }
                PyExpr::Name { id, .. } => {
                    let base = id.clone();
                    let mut is_ptr = false;

                    // 检查 self -> 总是指针
                    if base == "self" {
                        is_ptr = true;
                    } else {
                        // 从作用域查找
                        for scope in self.var_scopes.iter().rev() {
                            if let Some(t) = scope.get(&base) {
                                if t.contains('*') {
                                    is_ptr = true;
                                }
                                break;
                            }
                        }
                        // 从符号表查找
                        if !is_ptr {
                            if let Some(sym) = self.symbol_table.get(&base) {
                                if let super::types::SymbolKind::Variable { is_pointer, .. } = sym {
                                    is_ptr = *is_pointer;
                                }
                            }
                        }
                    }

                    // 从内到外构建
                    chain.reverse();
                    let mut result = base;
                    let mut current_is_ptr = is_ptr;
                    let mut _current_struct: Option<String> = None;

                    for (i, member) in chain.iter().enumerate() {
                        if current_is_ptr {
                            result.push_str("->");
                        } else {
                            result.push('.');
                        }
                        result.push_str(member);

                        // 查找成员类型，更新指针状态
                        if i < chain.len() - 1 {
                            // 简化：不深入成员类型分析，保持当前指针状态
                        }
                    }

                    return vec![result];
                }
                _ => {
                    // 非 Name 基础表达式
                    let base = self.handle_expr(current);
                    if base.is_empty() {
                        return vec!["0".to_string()];
                    }
                    chain.reverse();
                    let mut result = base[0].clone();
                    for member in &chain {
                        result.push('.');
                        result.push_str(member);
                    }
                    return vec![result];
                }
            }
        }
    }

    /// 从表达式中推断结构体名称
    fn find_struct_name(&self, expr: &PyExpr) -> Option<String> {
        match expr {
            PyExpr::Name { id, .. } => {
                // 从符号表查找类型
                if let Some(sym) = self.symbol_table.get(id) {
                    if let super::types::SymbolKind::Variable { declared_type, .. } = sym {
                        if declared_type.starts_with("struct ") {
                            let name = declared_type.split(' ').nth(1)?;
                            return Some(name.trim_end_matches('*').to_string());
                        }
                    }
                }
                // 从作用域查找
                for scope in self.var_scopes.iter().rev() {
                    if let Some(t) = scope.get(id) {
                        if t.starts_with("struct ") {
                            let name = t.split(' ').nth(1)?;
                            return Some(name.trim_end_matches('*').to_string());
                        }
                    }
                }
                None
            }
            PyExpr::Call { func, .. } => {
                if let PyExpr::Name { id, .. } = func.as_ref() {
                    Some(id.clone())
                } else if let PyExpr::Attribute { attr, .. } = func.as_ref() {
                    Some(attr.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    // ── 特殊调用处理 ──

    fn handle_c_special_call(
        &self,
        attr: &str,
        args: &[PyExpr],
        _keywords: &[ast::Keyword],
    ) -> Vec<String> {
        match attr {
            "Asm" => {
                if args.is_empty() {
                    return vec!["__asm__ volatile (\"nop\");".to_string()];
                }
                match &args[0] {
                    PyExpr::Constant { value, .. } => {
                        if let Constant::Str(code) = value {
                            let lines: Vec<&str> = code.split('\n').collect();
                            if lines.len() > 1 {
                                let joined = lines.join("\\n\\t\"\n        \"");
                                vec![format!(
                                    "__asm__ volatile (\n        \"{}\"\n    );",
                                    joined
                                )]
                            } else {
                                vec![format!("__asm__ volatile (\"{}\");", code)]
                            }
                        } else {
                            vec!["__asm__ volatile (\"nop\");".to_string()]
                        }
                    }
                    _ => vec!["__asm__ volatile (\"nop\");".to_string()],
                }
            }
            "Memory" => {
                if !args.is_empty() {
                    let addr = self.handle_expr(&args[0]);
                    if !addr.is_empty() {
                        return vec![format!("((void *){})", addr[0])];
                    }
                }
                vec!["((void *)0)".to_string()]
            }
            "Set" => {
                if args.len() >= 2 {
                    let target = self.handle_expr(&args[0]);
                    let value = self.handle_expr(&args[1]);
                    if !target.is_empty() && !value.is_empty() {
                        return vec![format!("{} = {};", target[0], value[0])];
                    }
                }
                Vec::new()
            }
            "TypeCast" => {
                if args.len() >= 2 {
                    let type_name = match &args[0] {
                        PyExpr::Constant { value, .. } if let Constant::Str(s) = value => s.clone(),
                        _ => "void".to_string(),
                    };
                    let value = self.handle_expr(&args[1]);
                    if !value.is_empty() {
                        return vec![format!("(({}){})", type_name, value[0])];
                    }
                }
                vec!["((void *)0)".to_string()]
            }
            "Macro" => {
                if args.len() >= 2 {
                    let name = match &args[0] {
                        PyExpr::Constant { value, .. } if let Constant::Str(s) = value => s.clone(),
                        _ => "MACRO".to_string(),
                    };
                    let value = match &args[1] {
                        PyExpr::Constant { value, .. } if let Constant::Str(s) = value => s.clone(),
                        _ => "0".to_string(),
                    };
                    return vec![format!("#define {} {}", name, value)];
                }
                Vec::new()
            }
            "Addr" => {
                if !args.is_empty() {
                    let expr = self.handle_expr(&args[0]);
                    if !expr.is_empty() {
                        return vec![format!("&{}", expr[0])];
                    }
                }
                vec!["0".to_string()]
            }
            "Ptr" => {
                if args.is_empty() {
                    return vec!["((void *)0)".to_string()];
                }
                let addr = self.handle_expr(&args[0]);
                if addr.is_empty() {
                    return vec!["((void *)0)".to_string()];
                }
                // 处理 value 和 type 参数
                let mut value_code: Option<String> = None;
                let mut type_code: Option<String> = None;

                if args.len() > 1 {
                    value_code = Some(self.handle_expr(&args[1])[0].clone());
                }
                // keywords 参数在这里不可用，简化处理

                if let Some(v) = value_code {
                    if let Some(t) = type_code {
                        vec![format!("*(({}*){}) = {};", t, addr[0], v)]
                    } else {
                        vec![format!("*((void *){}) = {};", addr[0], v)]
                    }
                } else if let Some(t) = type_code {
                    vec![format!("(({}*){})", t, addr[0])]
                } else {
                    vec![format!("((void *){})", addr[0])]
                }
            }
            "Cast" => {
                if !args.is_empty() {
                    let expr = self.handle_expr(&args[0]);
                    if !expr.is_empty() {
                        return vec![format!("*({})", expr[0])];
                    }
                }
                vec!["0".to_string()]
            }
            _ => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.handle_expr(a)[0].clone())
                    .collect();
                vec![format!("c.{}({});", attr, args_str.join(", "))]
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

        let value = self.handle_expr(&args[0]);
        if value.is_empty() {
            return vec!["0".to_string()];
        }
        let val_str = &value[0];

        match attr {
            "CType" => {
                // t.CType(addr, Type, ...)
                let mut types: Vec<String> = Vec::new();
                for arg in &args[1..] {
                    let t = self.get_type_name(arg);
                    if !t.is_empty() {
                        types.push(t);
                    }
                }
                if types.is_empty() {
                    vec![val_str.clone()]
                } else {
                    vec![format!("(({}){})", types.join(" "), val_str)]
                }
            }
            "CStruct" => {
                let struct_name = if args.len() >= 2 {
                    if let PyExpr::Name { id, .. } = &args[1] {
                        id.clone()
                    } else {
                        "BOOTINFO".to_string()
                    }
                } else {
                    "BOOTINFO".to_string()
                };
                vec![format!("((struct {} *){})", struct_name, val_str)]
            }
            _ => {
                // t.CInt(x), t.CChar(x, t.CPtr) 等类型转换
                let mut type_parts = Vec::new();
                // 查找 attr 在 TYPE_MAP 中的对应关系
                if let Some(c_type) = super::types::lookup_type(attr) {
                    type_parts.push(c_type.to_string());
                } else {
                    type_parts.push(attr.to_string());
                }
                for arg in &args[1..] {
                    let t = self.get_type_name(arg);
                    if !t.is_empty() {
                        type_parts.push(t);
                    }
                }
                let type_str = type_parts.join(" ");
                vec![format!("(({}){})", type_str, val_str)]
            }
        }
    }
}
