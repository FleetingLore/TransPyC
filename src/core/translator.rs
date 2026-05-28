//! TransPyC 核心翻译器

use std::collections::HashMap;

use super::types::*;
use crate::constants::{
    AUG_OPERATOR_MAP, BUILTIN_FUNCTIONS, COMPARATOR_MAP, OPERATOR_MAP, TYPE_MAP, UNARY_OPERATOR_MAP,
};

/// TransPyC 主翻译器
pub struct Translator {
    /// 变量作用域栈
    pub var_scopes: VarScopes,
    /// 函数返回类型记录
    pub function_return_types: FunctionReturnTypes,
    /// 符号表
    pub symbol_table: SymbolTable,
    /// 原始代码行
    pub original_lines: Vec<String>,
    /// 源代码内容
    pub content: String,
    /// 调试输出 (Vec of log lines)
    pub debug_logs: Vec<String>,
}

impl Translator {
    pub fn new() -> Self {
        Self {
            var_scopes: Vec::new(),
            function_return_types: HashMap::new(),
            symbol_table: HashMap::new(),
            original_lines: Vec::new(),
            content: String::new(),
            debug_logs: Vec::new(),
        }
    }

    pub fn debug_print(&mut self, msg: &str) {
        self.debug_logs.push(msg.to_string());
    }

    // ── 入口: 从 Python 源码生成 C 代码 ──

    /// 解析 Python AST 并生成 C 代码
    pub fn generate_c_code(&mut self, source: &str) -> String {
        self.content = source.to_string();
        self.original_lines = source.lines().map(|l| l.to_string()).collect();

        // 使用 rustpython-parser 解析
        let ast = match rustpython_parser::parse_program(source, "<embedded>") {
            Ok(ast) => ast,
            Err(e) => {
                return format!("/* Parse error: {:?} */", e);
            }
        };

        // 第一遍: 收集符号 (类/函数/带注解的全局变量)
        self.collect_symbols(&ast);

        let mut code: Vec<String> = Vec::new();

        // 处理导入语句
        for stmt in &ast.body {
            match stmt {
                rustpython_parser::ast::Stmt::Import { names, .. } => {
                    code.extend(self.handle_import(names));
                }
                rustpython_parser::ast::Stmt::ImportFrom {
                    module,
                    names,
                    level,
                    ..
                } => {
                    code.extend(self.handle_import_from(module, names, *level));
                }
                _ => {}
            }
        }

        // 处理宏定义 (c.Macro)
        for stmt in &ast.body {
            self.extract_macros(stmt, &mut code);
        }

        // 处理全局变量和结构体定义
        for stmt in &ast.body {
            match stmt {
                rustpython_parser::ast::Stmt::ClassDef { name, body, .. } => {
                    code.extend(self.handle_class_def(name, body));
                }
                rustpython_parser::ast::Stmt::Assign { targets, value, .. } => {
                    let c = self.handle_assign(targets, value);
                    if !c.is_empty() {
                        code.extend(c);
                    }
                }
                rustpython_parser::ast::Stmt::AnnAssign {
                    target,
                    annotation,
                    value,
                    ..
                } => {
                    let c = self.handle_ann_assign(target, annotation, value.as_deref());
                    if !c.is_empty() {
                        code.extend(c);
                    }
                }
                _ => {}
            }
        }

        // 处理函数定义
        for stmt in &ast.body {
            if let rustpython_parser::ast::Stmt::FunctionDef {
                name,
                args,
                body,
                returns,
                ..
            } = stmt
            {
                code.extend(self.handle_function_def(name, args, body, returns.as_deref()));
            }
        }

        code.join("\n")
    }

    // ── 第一遍: 收集符号 ──

    fn collect_symbols(&mut self, ast: &rustpython_parser::ast::Mod) {
        for stmt in &ast.body {
            match stmt {
                rustpython_parser::ast::Stmt::ClassDef { name, body, .. } => {
                    let mut members = HashMap::new();
                    for item in body {
                        if let rustpython_parser::ast::Stmt::AnnAssign {
                            target, annotation, ..
                        } = item
                        {
                            if let rustpython_parser::ast::Expr::Name { id, .. } = target.as_ref() {
                                let type_name = self.get_type_name(annotation);
                                let is_ptr = type_name.contains('*') || type_name.contains("CPtr");
                                members.insert(
                                    id.clone(),
                                    MemberInfo {
                                        type_name: type_name.clone(),
                                        is_pointer: is_ptr,
                                    },
                                );
                            }
                        }
                    }
                    self.symbol_table
                        .insert(name.to_string(), SymbolKind::Struct { members });
                }
                rustpython_parser::ast::Stmt::FunctionDef { name, .. } => {
                    self.symbol_table
                        .insert(name.to_string(), SymbolKind::Function);
                }
                rustpython_parser::ast::Stmt::AnnAssign {
                    target, annotation, ..
                } => {
                    if let rustpython_parser::ast::Expr::Name { id, .. } = target.as_ref() {
                        let type_name = self.get_type_name(annotation);
                        let is_ptr = type_name.contains('*') || type_name.contains("CPtr");
                        self.symbol_table.insert(
                            id.clone(),
                            SymbolKind::Variable {
                                declared_type: type_name,
                                is_pointer: is_ptr,
                            },
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // ── 导入处理 ──

    fn handle_import(&self, names: &[rustpython_parser::ast::Alias]) -> Vec<String> {
        let mut code = Vec::new();
        for alias in names {
            let name = &alias.node.name;
            if name != "c" && name != "t" {
                if name == "stdio" {
                    code.push(format!("#include <{}.h>", name));
                } else {
                    code.push(format!("#include \"{}.h\"", name));
                }
            }
        }
        code
    }

    fn handle_import_from(
        &self,
        module: &Option<String>,
        names: &[rustpython_parser::ast::Alias],
        _level: u32,
    ) -> Vec<String> {
        let mut code = Vec::new();
        if let Some(mod_name) = module {
            if mod_name != "c" && mod_name != "t" {
                let path = mod_name.replace('.', "/");
                let comment = if names.len() == 1 && names[0].node.name == "*" {
                    format!("from {} import *", mod_name)
                } else {
                    let names_str: Vec<&str> = names.iter().map(|a| a.node.name.as_str()).collect();
                    format!("from {} import {}", mod_name, names_str.join(", "))
                };
                code.push(format!("#include \"{}.h\" // {}", path, comment));
            }
        }
        code
    }

    // ── 宏提取 ──

    fn extract_macros(&self, stmt: &rustpython_parser::ast::Stmt, code: &mut Vec<String>) {
        if let rustpython_parser::ast::Stmt::Expr { value } = stmt {
            if let rustpython_parser::ast::Expr::Call { func, args, .. } = value.as_ref() {
                if let rustpython_parser::ast::Expr::Attribute {
                    value: obj, attr, ..
                } = func.as_ref()
                {
                    if let rustpython_parser::ast::Expr::Name { id, .. } = obj.as_ref() {
                        if id == "c" && attr == "Macro" && args.len() >= 2 {
                            if let (
                                rustpython_parser::ast::Expr::Constant { value: val0, .. },
                                rustpython_parser::ast::Expr::Constant { value: val1, .. },
                            ) = (&args[0], &args[1])
                            {
                                use rustpython_parser::ast::Constant;
                                if let (Constant::Str(name), Constant::Str(value)) = (val0, val1) {
                                    code.push(format!("#define {} {}", name, value));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── 函数定义 ──

    pub fn handle_function_def(
        &mut self,
        name: &str,
        args: &rustpython_parser::ast::Arguments,
        body: &[rustpython_parser::ast::Stmt],
        returns: Option<&rustpython_parser::ast::Expr>,
    ) -> Vec<String> {
        let mut code = Vec::new();
        let mut return_type = "void".to_string();
        let mut is_declaration = false;

        // 检查返回类型和声明标志
        if let Some(ret) = returns {
            let t = self.get_type_name(ret);
            if t == "c.State" || t.contains("c.State") {
                is_declaration = true;
                return_type = self.extract_return_type_from_state(ret);
            } else {
                return_type = if t.is_empty() { "void".to_string() } else { t };
            }
        }

        if name == "main" && return_type == "void" {
            return_type = "int".to_string();
        }

        // 创建新作用域
        self.var_scopes.push(HashMap::new());
        self.debug_print(&format!(
            "[SCOPE] Enter function '{}', depth={}",
            name,
            self.var_scopes.len()
        ));

        // 处理参数
        let params: Vec<String> = args
            .args
            .iter()
            .map(|arg| {
                let param_type = arg
                    .node
                    .annotation
                    .as_ref()
                    .map(|a| self.get_type_name(a))
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "int".to_string());
                // 添加到作用域
                if let Some(scope) = self.var_scopes.last_mut() {
                    scope.insert(arg.node.arg.clone(), param_type.clone());
                }
                format!("{} {}", param_type, arg.node.arg)
            })
            .collect();

        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params.join(", ")
        };

        if is_declaration {
            code.push(format!("{} {}({});", return_type, name, params_str));
        } else {
            code.push(format!("{} {}({}) {{", return_type, name, params_str));
            let body_code = self.handle_body(body, false);
            for line in body_code {
                code.push(format!("    {}", line));
            }
            code.push("}".to_string());
        }

        // 弹出作用域
        self.var_scopes.pop();
        self.debug_print(&format!(
            "[SCOPE] Exit function '{}', depth={}",
            name,
            self.var_scopes.len()
        ));

        self.function_return_types
            .insert(name.to_string(), return_type);
        code
    }

    /// 从 t.CState | SomeType 形式的返回类型中提取实际类型
    fn extract_return_type_from_state(&self, ret: &rustpython_parser::ast::Expr) -> String {
        match ret {
            rustpython_parser::ast::Expr::BinOp { left, op, .. } => {
                if matches!(op.as_ref(), rustpython_parser::ast::Operator::BitOr) {
                    let left_t = self.get_type_name(left);
                    let right_t = self.get_type_name(&ret); // fallback
                    if left_t == "c.State" || left_t.contains("c.State") {
                        // 返回右侧类型
                        self.extract_other_side_of_bitor(left, &ret)
                    } else {
                        left_t
                    }
                } else {
                    "void".to_string()
                }
            }
            _ => "void".to_string(),
        }
    }

    fn extract_other_side_of_bitor(
        &self,
        _left: &rustpython_parser::ast::Expr,
        full: &rustpython_parser::ast::Expr,
    ) -> String {
        // 从完整的 BinOp(BitOr) 中提取另一侧
        if let rustpython_parser::ast::Expr::BinOp { right, .. } = full {
            self.get_type_name(right)
        } else {
            "void".to_string()
        }
    }

    // ── 类定义 ──

    pub fn handle_class_def(
        &mut self,
        name: &str,
        body: &[rustpython_parser::ast::Stmt],
    ) -> Vec<String> {
        let mut code = Vec::new();
        code.push(format!("struct {} {{", name));

        for item in body {
            match item {
                rustpython_parser::ast::Stmt::AnnAssign {
                    target, annotation, ..
                } => {
                    if let rustpython_parser::ast::Expr::Name { id: var_name, .. } = target.as_ref()
                    {
                        let type_name = self.get_type_name(annotation);
                        if !type_name.is_empty() {
                            let processed = self.process_struct_member_type(&type_name, var_name);
                            code.push(format!("    {} {};", processed, var_name));
                        } else {
                            code.push(format!("    int {};", var_name));
                        }
                    }
                }
                rustpython_parser::ast::Stmt::FunctionDef {
                    name: mname,
                    body: fbody,
                    ..
                } => {
                    if mname == "__init__" {
                        for stmt in fbody {
                            if let rustpython_parser::ast::Stmt::AnnAssign {
                                target,
                                annotation,
                                ..
                            } = stmt
                            {
                                if let rustpython_parser::ast::Expr::Attribute {
                                    value: obj,
                                    attr,
                                    ..
                                } = target.as_ref()
                                {
                                    if let rustpython_parser::ast::Expr::Name { id, .. } =
                                        obj.as_ref()
                                    {
                                        if id == "self" {
                                            let type_name = self.get_type_name(annotation);
                                            if !type_name.is_empty() {
                                                code.push(format!("    {} {};", type_name, attr));
                                            } else {
                                                code.push(format!("    int {};", attr));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        code.push("};".to_string());
        code
    }

    fn process_struct_member_type(&self, type_name: &str, _var_name: &str) -> String {
        if type_name == "struct *" || type_name == "*" {
            return "void *".to_string();
        }
        type_name.to_string()
    }

    // ── 方法定义 ──

    pub fn handle_method_def(
        &mut self,
        class_name: &str,
        name: &str,
        args: &rustpython_parser::ast::Arguments,
        body: &[rustpython_parser::ast::Stmt],
        returns: Option<&rustpython_parser::ast::Expr>,
    ) -> Vec<String> {
        let mut code = Vec::new();
        let return_type = returns
            .map(|r| self.get_type_name(r))
            .filter(|t| !t.is_empty())
            .unwrap_or_else(|| "void".to_string());

        let func_name = format!("{}__{}", class_name, name);

        let mut params = vec![format!("struct {}* self", class_name)];
        for arg in &args.args {
            if arg.node.arg != "self" {
                let param_type = arg
                    .node
                    .annotation
                    .as_ref()
                    .map(|a| self.get_type_name(a))
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "int".to_string());
                params.push(format!("{} {}", param_type, arg.node.arg));
            }
        }

        let params_str = params.join(", ");
        code.push(format!("{} {}({}) {{", return_type, func_name, params_str));

        self.var_scopes.push(HashMap::new());
        let body_code = self.handle_body(body, false);
        for line in body_code {
            code.push(format!("    {}", line));
        }
        self.var_scopes.pop();

        code.push("}".to_string());
        code
    }

    // ── 赋值 ──

    fn handle_assign(
        &mut self,
        targets: &[rustpython_parser::ast::Expr],
        value: &rustpython_parser::ast::Expr,
    ) -> Vec<String> {
        let mut code = Vec::new();

        // 处理多重赋值: a, b = b, a
        if targets.len() == 1 {
            if let rustpython_parser::ast::Expr::Tuple {
                elts: target_elts, ..
            } = &targets[0]
            {
                if let rustpython_parser::ast::Expr::Tuple {
                    elts: value_elts, ..
                } = value
                {
                    if target_elts.len() == 2 && value_elts.len() == 2 {
                        let t0 = self.handle_expr(&target_elts[0])[0].clone();
                        let t1 = self.handle_expr(&target_elts[1])[0].clone();
                        code.push("int temp;".to_string());
                        code.push(format!("temp = {};", t0));
                        code.push(format!("{} = {};", t0, t1));
                        code.push(format!("{} = temp;", t1));
                        return code;
                    }
                }
            }
        }

        for target in targets {
            match target {
                rustpython_parser::ast::Expr::Name { id, .. } => {
                    let value_code = self.handle_expr(value);
                    if value_code.is_empty() {
                        continue;
                    }
                    let val = &value_code[0];

                    if self.is_var_declared(id) {
                        code.push(format!("{} = {};", id, val));
                    } else {
                        // 未声明，自动推断为 int
                        code.push(format!("int {} = {};", id, val));
                        if let Some(scope) = self.var_scopes.last_mut() {
                            scope.insert(id.clone(), "int".to_string());
                        }
                    }
                }
                rustpython_parser::ast::Expr::Attribute {
                    value: obj, attr, ..
                } => {
                    let obj_code = self.handle_expr(obj);
                    if obj_code.is_empty() {
                        continue;
                    }
                    let obj_str = &obj_code[0];
                    let is_ptr = self.check_is_pointer_expr(obj_str, obj);
                    let value_code = self.handle_expr(value);
                    if value_code.is_empty() {
                        continue;
                    }
                    let val = &value_code[0];
                    if is_ptr {
                        code.push(format!("{}->{} = {};", obj_str, attr, val));
                    } else {
                        code.push(format!("{}.{} = {};", obj_str, attr, val));
                    }
                }
                rustpython_parser::ast::Expr::Subscript {
                    value: arr, slice, ..
                } => {
                    let arr_code = self.handle_expr(arr);
                    let idx_code = self.handle_expr(slice);
                    if !arr_code.is_empty() && !idx_code.is_empty() {
                        let val = &self.handle_expr(value)[0];
                        code.push(format!("{}[{}] = {};", arr_code[0], idx_code[0], val));
                    }
                }
                _ => {}
            }
        }

        code
    }

    // ── 带注解的赋值 ──

    fn handle_ann_assign(
        &mut self,
        target: &rustpython_parser::ast::Expr,
        annotation: &rustpython_parser::ast::Expr,
        value: Option<&rustpython_parser::ast::Expr>,
    ) -> Vec<String> {
        let mut code = Vec::new();

        let var_name = match target {
            rustpython_parser::ast::Expr::Name { id, .. } => id.clone(),
            _ => return code,
        };

        // 如果变量已在当前作用域，生成赋值
        if self
            .var_scopes
            .last()
            .map(|s| s.contains_key(&var_name))
            .unwrap_or(false)
        {
            if let Some(val) = value {
                let v = self.handle_expr(val);
                if !v.is_empty() {
                    code.push(format!("{} = {};", var_name, v[0]));
                }
            }
            return code;
        }

        let type_name = self.get_type_name(annotation);
        if type_name.is_empty() {
            // 默认 int
            if let Some(val) = value {
                let v = self.handle_expr(val);
                if !v.is_empty() {
                    code.push(format!("int {} = {};", var_name, v[0]));
                }
            } else {
                code.push(format!("int {};", var_name));
            }
            if let Some(scope) = self.var_scopes.last_mut() {
                scope.insert(var_name, "int".to_string());
            }
            return code;
        }

        // 处理 #define 类型
        if type_name == "#define" {
            if let Some(val) = value {
                let v = self.handle_expr(val);
                if !v.is_empty() {
                    code.push(format!("#define {} {}", var_name, v[0]));
                }
            }
            return code;
        }

        // 处理数组初始化
        if let Some(rustpython_parser::ast::Expr::List { elts, .. }) = value {
            let (base_type, array_str) = extract_array_size(&type_name);
            let base = if base_type.is_empty() {
                &type_name
            } else {
                &base_type
            };
            let elements: Vec<String> = elts
                .iter()
                .map(|e| self.handle_expr(e)[0].clone())
                .collect();
            code.push(format!(
                "{} {}{} = {{ {} }};",
                base,
                var_name,
                array_str,
                elements.join(", ")
            ));
            if let Some(scope) = self.var_scopes.last_mut() {
                scope.insert(var_name, type_name.clone());
            }
            return code;
        }

        // 处理 c.State (声明不定义)
        if let Some(rustpython_parser::ast::Expr::Attribute { .. }) = value {
            let v = self.handle_expr(value.unwrap());
            if !v.is_empty() && v[0] == "c.State" {
                let (base_type, array_str) = extract_array_size(&type_name);
                let (storage, type_part) = check_storage_class(&base_type);
                if !storage.is_empty() {
                    code.push(format!(
                        "{} {} {}{};",
                        storage, type_part, var_name, array_str
                    ));
                } else {
                    code.push(format!("{} {}{};", base_type, var_name, array_str));
                }
                if let Some(scope) = self.var_scopes.last_mut() {
                    scope.insert(var_name, type_name);
                }
                return code;
            }
        }

        // 普通带初值的声明
        if let Some(val) = value {
            let (base_type, array_str) = extract_array_size(&type_name);
            let (storage, type_part) = check_storage_class(&base_type);
            let v = self.handle_expr(val);
            let val_str = if v.is_empty() { "0" } else { &v[0] };

            if !storage.is_empty() {
                code.push(format!(
                    "{} {} {}{} = {};",
                    storage, type_part, var_name, array_str, val_str
                ));
            } else {
                code.push(format!(
                    "{} {}{} = {};",
                    base_type, var_name, array_str, val_str
                ));
            }
        } else {
            // 纯声明
            let (base_type, array_str) = extract_array_size(&type_name);
            let (storage, type_part) = check_storage_class(&base_type);
            if !storage.is_empty() {
                code.push(format!(
                    "{} {} {}{};",
                    storage, type_part, var_name, array_str
                ));
            } else {
                code.push(format!("{} {}{};", base_type, var_name, array_str));
            }
        }

        if let Some(scope) = self.var_scopes.last_mut() {
            scope.insert(var_name, type_name.clone());
        }
        code
    }

    // ── 语句体处理 ──

    pub fn handle_body(
        &mut self,
        body: &[rustpython_parser::ast::Stmt],
        in_block: bool,
    ) -> Vec<String> {
        let mut code = Vec::new();

        for stmt in body {
            match stmt {
                rustpython_parser::ast::Stmt::Expr { value } => {
                    if let rustpython_parser::ast::Expr::Call { func, .. } = value.as_ref() {
                        if let rustpython_parser::ast::Expr::Attribute {
                            value: obj, attr, ..
                        } = func.as_ref()
                        {
                            if let rustpython_parser::ast::Expr::Name { id, .. } = obj.as_ref() {
                                if id == "c" {
                                    // c.* 特殊调用在 handle_body 中作为表达式处理
                                }
                            }
                        }
                    }
                    let expr_code = self.handle_expr(value);
                    for e in expr_code {
                        if !e.is_empty() && e != "0" {
                            if e.ends_with(';') {
                                code.push(e);
                            } else {
                                code.push(format!("{};", e));
                            }
                        }
                    }
                }
                rustpython_parser::ast::Stmt::If {
                    test, body, orelse, ..
                } => {
                    code.extend(self.handle_if(test, body, orelse));
                }
                rustpython_parser::ast::Stmt::For {
                    target, iter, body, ..
                } => {
                    code.extend(self.handle_for(target, iter, body));
                }
                rustpython_parser::ast::Stmt::While { test, body, .. } => {
                    code.extend(self.handle_while(test, body));
                }
                rustpython_parser::ast::Stmt::Break { .. } => {
                    code.push("break;".to_string());
                }
                rustpython_parser::ast::Stmt::Continue { .. } => {
                    code.push("continue;".to_string());
                }
                rustpython_parser::ast::Stmt::Return { value: ret_val, .. } => {
                    if let Some(v) = ret_val {
                        let vc = self.handle_expr(v);
                        if !vc.is_empty() {
                            code.push(format!("return {};", vc[0]));
                        } else {
                            code.push("return;".to_string());
                        }
                    }
                }
                rustpython_parser::ast::Stmt::Assign { targets, value, .. } => {
                    code.extend(self.handle_assign(targets, value));
                }
                rustpython_parser::ast::Stmt::AugAssign {
                    target, op, value, ..
                } => {
                    code.extend(self.handle_aug_assign(target, op, value));
                }
                rustpython_parser::ast::Stmt::AnnAssign {
                    target,
                    annotation,
                    value,
                    ..
                } => {
                    let c = self.handle_ann_assign(target, annotation, value.as_deref());
                    if !c.is_empty() {
                        code.extend(c);
                    }
                }
                rustpython_parser::ast::Stmt::ClassDef { .. } => {
                    // 类定义在全局作用域处理，函数体内忽略
                }
                _ => {}
            }
        }

        code
    }

    // ── if 语句 ──

    fn handle_if(
        &mut self,
        test: &rustpython_parser::ast::Expr,
        body: &[rustpython_parser::ast::Stmt],
        orelse: &[rustpython_parser::ast::Stmt],
    ) -> Vec<String> {
        let mut code = Vec::new();
        let test_str = self.handle_expr(test);
        let test_code = if test_str.is_empty() {
            "0"
        } else {
            &test_str[0]
        };

        code.push(format!("if ({}) {{", test_code));
        for line in self.handle_body(body, true) {
            code.push(format!("    {}", line));
        }
        code.push("}".to_string());

        if !orelse.is_empty() {
            // 检查是否是 elif
            if orelse.len() == 1 && matches!(orelse[0], rustpython_parser::ast::Stmt::If { .. }) {
                if let rustpython_parser::ast::Stmt::If {
                    test: elif_test,
                    body: elif_body,
                    orelse: elif_orelse,
                    ..
                } = &orelse[0]
                {
                    code.push("else".to_string());
                    code.extend(self.handle_if(elif_test, elif_body, elif_orelse));
                }
            } else {
                code.push("else {".to_string());
                for line in self.handle_body(orelse, true) {
                    code.push(format!("    {}", line));
                }
                code.push("}".to_string());
            }
        }

        code
    }

    // ── for 语句 ──

    fn handle_for(
        &mut self,
        target: &rustpython_parser::ast::Expr,
        iter: &rustpython_parser::ast::Expr,
        body: &[rustpython_parser::ast::Stmt],
    ) -> Vec<String> {
        let mut code = Vec::new();

        // 处理 range 循环
        if let rustpython_parser::ast::Expr::Call { func, args, .. } = iter {
            if let rustpython_parser::ast::Expr::Name { id, .. } = func.as_ref() {
                if id == "range" {
                    let var_name = match target {
                        rustpython_parser::ast::Expr::Name { id, .. } => id.clone(),
                        _ => {
                            code.push("for (...) {".to_string());
                            code.extend(
                                self.handle_body(body, true)
                                    .into_iter()
                                    .map(|l| format!("    {}", l)),
                            );
                            code.push("}".to_string());
                            return code;
                        }
                    };

                    let (start, stop, step) = match args.len() {
                        1 => (
                            "0".to_string(),
                            self.handle_expr(&args[0])[0].clone(),
                            "1".to_string(),
                        ),
                        2 => (
                            self.handle_expr(&args[0])[0].clone(),
                            self.handle_expr(&args[1])[0].clone(),
                            "1".to_string(),
                        ),
                        3 => (
                            self.handle_expr(&args[0])[0].clone(),
                            self.handle_expr(&args[1])[0].clone(),
                            self.handle_expr(&args[2])[0].clone(),
                        ),
                        _ => ("0".to_string(), "0".to_string(), "1".to_string()),
                    };

                    let (cond_op, inc_op) = if step.starts_with('-') {
                        (">", &step[..])
                    } else {
                        ("<", &step[..])
                    };

                    let var_declared = self.is_var_declared(&var_name);
                    let init = if !var_declared {
                        format!("int {} = {}", var_name, start)
                    } else {
                        format!("{} = {}", var_name, start)
                    };

                    code.push(format!(
                        "for ({}; {} {} {}; {} += {}) {{",
                        init, var_name, cond_op, stop, var_name, inc_op
                    ));
                    code.extend(
                        self.handle_body(body, true)
                            .into_iter()
                            .map(|l| format!("    {}", l)),
                    );
                    code.push("}".to_string());
                    return code;
                }
            }
        }

        // 字符串切片遍历
        if let rustpython_parser::ast::Expr::Subscript {
            value: sub_val,
            slice,
            ..
        } = iter
        {
            if let rustpython_parser::ast::Expr::Slice { lower, .. } = slice.as_ref() {
                if let rustpython_parser::ast::Expr::Name { id: var_name, .. } = target {
                    if let rustpython_parser::ast::Expr::Name { id: base_var, .. } =
                        sub_val.as_ref()
                    {
                        let start_idx = lower
                            .as_ref()
                            .map(|l| self.handle_expr(l))
                            .filter(|v| !v.is_empty())
                            .map(|v| v[0].clone())
                            .unwrap_or_else(|| "0".to_string());
                        code.push(format!(
                            "for (int __for_i = {}; {}[__for_i] != '\\0'; __for_i++) {{",
                            start_idx, base_var
                        ));
                        code.push(format!("    char {} = {}[__for_i];", var_name, base_var));
                        code.extend(
                            self.handle_body(body, true)
                                .into_iter()
                                .map(|l| format!("    {}", l)),
                        );
                        code.push("}".to_string());
                        return code;
                    }
                }
            }
        }

        // 字符串直接遍历
        if let rustpython_parser::ast::Expr::Name { id: base_var, .. } = iter {
            if let rustpython_parser::ast::Expr::Name { id: var_name, .. } = target {
                code.push(format!(
                    "for (int __for_i = 0; {}[__for_i] != 0; __for_i++) {{",
                    base_var
                ));
                code.push(format!("    char {} = {}[__for_i];", var_name, base_var));
                code.extend(
                    self.handle_body(body, true)
                        .into_iter()
                        .map(|l| format!("    {}", l)),
                );
                code.push("}".to_string());
                return code;
            }
        }

        // fallback
        code.push("for (...) {".to_string());
        code.extend(
            self.handle_body(body, true)
                .into_iter()
                .map(|l| format!("    {}", l)),
        );
        code.push("}".to_string());
        code
    }

    // ── while 语句 ──

    fn handle_while(
        &mut self,
        test: &rustpython_parser::ast::Expr,
        body: &[rustpython_parser::ast::Stmt],
    ) -> Vec<String> {
        let mut code = Vec::new();

        // 检查 do-while 模式: while True: ... if not cond: break
        if let rustpython_parser::ast::Expr::Constant { value, .. } = test {
            use rustpython_parser::ast::Constant;
            if let Constant::Bool(true) = value {
                if let Some(last) = body.last() {
                    if let rustpython_parser::ast::Stmt::If {
                        test: cond,
                        body: if_body,
                        orelse,
                        ..
                    } = last
                    {
                        if orelse.is_empty() && if_body.len() == 1 {
                            if let rustpython_parser::ast::Stmt::Break { .. } = &if_body[0] {
                                let cond_str = self.handle_expr(cond);
                                let cond_code = if cond_str.is_empty() {
                                    "0"
                                } else {
                                    &cond_str[0]
                                };
                                code.push("do {".to_string());
                                for stmt in body.iter().take(body.len() - 1) {
                                    let c = self.handle_stmt_single(stmt);
                                    for l in c {
                                        code.push(format!("    {}", l));
                                    }
                                }
                                code.push(format!("}} while (!({}));", cond_code));
                                return code;
                            }
                        }
                    }
                }
            }
        }

        let test_str = self.handle_expr(test);
        let test_code = if test_str.is_empty() {
            "0"
        } else {
            &test_str[0]
        };
        code.push(format!("while ({}) {{", test_code));
        for line in self.handle_body(body, true) {
            code.push(format!("    {}", line));
        }
        code.push("}".to_string());
        code
    }

    fn handle_stmt_single(&mut self, stmt: &rustpython_parser::ast::Stmt) -> Vec<String> {
        // 用于 do-while 中单条语句的处理
        match stmt {
            rustpython_parser::ast::Stmt::Assign { targets, value, .. } => {
                self.handle_assign(targets, value)
            }
            rustpython_parser::ast::Stmt::AugAssign {
                target, op, value, ..
            } => self.handle_aug_assign(target, op, value),
            rustpython_parser::ast::Stmt::AnnAssign {
                target,
                annotation,
                value,
                ..
            } => self.handle_ann_assign(target, annotation, value.as_deref()),
            rustpython_parser::ast::Stmt::If {
                test, body, orelse, ..
            } => self.handle_if(test, body, orelse),
            rustpython_parser::ast::Stmt::For {
                target, iter, body, ..
            } => self.handle_for(target, iter, body),
            rustpython_parser::ast::Stmt::While { test, body, .. } => self.handle_while(test, body),
            rustpython_parser::ast::Stmt::Break { .. } => vec!["break;".to_string()],
            rustpython_parser::ast::Stmt::Continue { .. } => vec!["continue;".to_string()],
            rustpython_parser::ast::Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    let vc = self.handle_expr(v);
                    if !vc.is_empty() {
                        vec![format!("return {};", vc[0])]
                    } else {
                        vec!["return;".to_string()]
                    }
                } else {
                    vec!["return;".to_string()]
                }
            }
            rustpython_parser::ast::Stmt::Expr { value } => {
                let mut r = Vec::new();
                for e in self.handle_expr(value) {
                    if !e.is_empty() && e != "0" {
                        if e.ends_with(';') {
                            r.push(e);
                        } else {
                            r.push(format!("{};", e));
                        }
                    }
                }
                r
            }
            _ => Vec::new(),
        }
    }

    // ── 复合赋值 ──

    fn handle_aug_assign(
        &mut self,
        target: &rustpython_parser::ast::Expr,
        op: &rustpython_parser::ast::Operator,
        value: &rustpython_parser::ast::Expr,
    ) -> Vec<String> {
        let mut code = Vec::new();
        let op_name = std::mem::discriminant(op).to_string(); // simplified
        let op_sym = self.get_aug_op_symbol(op);

        let val = self.handle_expr(value);
        let val_str = if val.is_empty() { "0" } else { &val[0] };

        match target {
            rustpython_parser::ast::Expr::Name { id, .. } => {
                code.push(format!("{} {}= {};", id, op_sym, val_str));
            }
            rustpython_parser::ast::Expr::Subscript {
                value: arr, slice, ..
            } => {
                let arr_c = self.handle_expr(arr);
                let idx_c = self.handle_expr(slice);
                if !arr_c.is_empty() && !idx_c.is_empty() {
                    code.push(format!(
                        "{}[{}] {}= {};",
                        arr_c[0], idx_c[0], op_sym, val_str
                    ));
                }
            }
            rustpython_parser::ast::Expr::Attribute {
                value: obj, attr, ..
            } => {
                let obj_c = self.handle_expr(obj);
                if !obj_c.is_empty() {
                    let is_ptr = self.check_is_pointer_expr(&obj_c[0], obj);
                    if is_ptr {
                        code.push(format!("{}->{} {}= {};", obj_c[0], attr, op_sym, val_str));
                    } else {
                        code.push(format!("{}.{} {}= {};", obj_c[0], attr, op_sym, val_str));
                    }
                }
            }
            _ => {}
        }

        code
    }

    // ── 运算符映射 ──

    fn get_op_symbol(&self, op: &rustpython_parser::ast::Operator) -> &'static str {
        let name = operator_type_name(op);
        OPERATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("+")
    }

    fn get_aug_op_symbol(&self, op: &rustpython_parser::ast::Operator) -> &'static str {
        let name = operator_type_name(op);
        AUG_OPERATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("+")
    }

    fn get_comparator_symbol(&self, op: &rustpython_parser::ast::CmpOp) -> &'static str {
        let name = cmp_op_type_name(op);
        COMPARATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("==")
    }

    fn get_unary_op_symbol(&self, op: &rustpython_parser::ast::UnaryOp) -> &'static str {
        let name = unary_op_type_name(op);
        UNARY_OPERATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("")
    }

    // ── 辅助函数 ──

    fn is_var_declared(&self, name: &str) -> bool {
        for scope in self.var_scopes.iter().rev() {
            if scope.contains_key(name) {
                return true;
            }
        }
        self.symbol_table
            .get(name)
            .map(|s| matches!(s, SymbolKind::Variable { .. }))
            .unwrap_or(false)
    }

    fn check_is_pointer_expr(&self, expr_str: &str, expr: &rustpython_parser::ast::Expr) -> bool {
        // 从表达式推断是否是指针
        if expr_str.starts_with("((struct") && expr_str.contains("*)") {
            return true;
        }
        if let rustpython_parser::ast::Expr::Name { id, .. } = expr {
            if id == "self" {
                return true;
            }
            // 从作用域查找
            for scope in self.var_scopes.iter().rev() {
                if let Some(t) = scope.get(id) {
                    if t.contains('*') {
                        return true;
                    }
                }
            }
            // 从符号表查找
            if let Some(sym) = self.symbol_table.get(id) {
                if let SymbolKind::Variable { is_pointer, .. } = sym {
                    return *is_pointer;
                }
            }
        }
        false
    }
}

// ── 类型名称辅助 ──

fn operator_type_name(op: &rustpython_parser::ast::Operator) -> String {
    match op {
        rustpython_parser::ast::Operator::Add => "Add",
        rustpython_parser::ast::Operator::Sub => "Sub",
        rustpython_parser::ast::Operator::Mult => "Mult",
        rustpython_parser::ast::Operator::Div => "Div",
        rustpython_parser::ast::Operator::Mod => "Mod",
        rustpython_parser::ast::Operator::Pow => "Pow",
        rustpython_parser::ast::Operator::LShift => "LShift",
        rustpython_parser::ast::Operator::RShift => "RShift",
        rustpython_parser::ast::Operator::BitOr => "BitOr",
        rustpython_parser::ast::Operator::BitXor => "BitXor",
        rustpython_parser::ast::Operator::BitAnd => "BitAnd",
        rustpython_parser::ast::Operator::FloorDiv => "FloorDiv",
        rustpython_parser::ast::Operator::MatMult => "MatMult",
    }
    .to_string()
}

fn cmp_op_type_name(op: &rustpython_parser::ast::CmpOp) -> String {
    match op {
        rustpython_parser::ast::CmpOp::Gt => "Gt",
        rustpython_parser::ast::CmpOp::Lt => "Lt",
        rustpython_parser::ast::CmpOp::GtE => "GtE",
        rustpython_parser::ast::CmpOp::LtE => "LtE",
        rustpython_parser::ast::CmpOp::Eq => "Eq",
        rustpython_parser::ast::CmpOp::NotEq => "NotEq",
        rustpython_parser::ast::CmpOp::Is => "Is",
        rustpython_parser::ast::CmpOp::IsNot => "IsNot",
        rustpython_parser::ast::CmpOp::In => "In",
        rustpython_parser::ast::CmpOp::NotIn => "NotIn",
    }
    .to_string()
}

fn unary_op_type_name(op: &rustpython_parser::ast::UnaryOp) -> String {
    match op {
        rustpython_parser::ast::UnaryOp::Not => "Not",
        rustpython_parser::ast::UnaryOp::Invert => "Invert",
        rustpython_parser::ast::UnaryOp::UAdd => "UAdd",
        rustpython_parser::ast::UnaryOp::USub => "USub",
    }
    .to_string()
}
