//! TransPyC 核心翻译器
//!
//! `Translator` 是翻译流程的总控制器。它持有符号表、作用域栈
//! 等状态，提供各个 AST 节点的处理方法。
//!
//! # 翻译流程
//!
//! `generate_c_code()` 是唯一对外入口：
//!
//! 1. 用 `rustpython_parser` 将 Python 源码解析为 AST
//! 2. `collect_symbols()` 第一遍扫描: 收集 class/function/全局变量
//! 3. 按代码出现顺序生成 C 代码:
//!    - 导入语句 → `#include`
//!    - 宏定义 → `#define`
//!    - 全局变量 / 结构体
//!    - 函数定义
//!
//! # 方法命名约定
//!
//! 所有 `handle_*` 方法接收 AST 节点引用，返回 `Vec<String>`
//! （C 代码行）。大多数方法需要 `&mut self` 以修改作用域/
//! 符号表，但表达式处理方法 `handle_expr()` 只需要 `&self`
//! （表达式求值不改变翻译状态）。

use std::collections::HashMap;

use rustpython_parser::Parse;

use super::types::*;
use crate::Error;
use crate::constants::{AUG_OPERATOR_MAP, COMPARATOR_MAP, OPERATOR_MAP, UNARY_OPERATOR_MAP};

/// TransPyC 主翻译器
pub struct Translator {
    /// 变量作用域栈
    pub var_scopes: VarScopes,
    /// 函数返回类型记录
    pub function_return_types: FunctionReturnTypes,
    /// 符号表
    pub symbol_table: SymbolTable,
    /// 调试日志
    pub debug_logs: Vec<String>,
    // 内部状态
    original_lines: Vec<String>,
    content: String,
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

    // ── 入口 ──

    /// 解析 Python 源码并生成 C 代码
    pub fn generate_c_code(&mut self, source: &str) -> Result<String, Error> {
        self.content = source.to_string();
        self.original_lines = source.lines().map(|l| l.to_string()).collect();

        let stmts = rustpython_parser::ast::Suite::parse(source, "<embedded>")
            .map_err(|e| Error::Parse(format!("{:?}", e)))?;

        // 第一遍: 收集符号
        self.collect_symbols(&stmts);

        // 第二遍: 单次遍历，按序生成。分类收集后统一输出以保证顺序正确。
        let mut imports = Vec::new();
        let mut macros = Vec::new();
        let mut globals = Vec::new();
        let mut structs = Vec::new();
        let mut funcs = Vec::new();

        for stmt in &stmts {
            use rustpython_parser::ast;
            match stmt {
                ast::Stmt::Import(import) => imports.extend(self.handle_import(&import.names)),
                ast::Stmt::ImportFrom(ifrom) => imports.extend(self.handle_import_from(
                    &ifrom.module,
                    &ifrom.names,
                    &ifrom.level,
                )),
                ast::Stmt::Expr(_) => self.extract_macros(stmt, &mut macros),
                ast::Stmt::ClassDef(class_def) => {
                    structs.extend(self.handle_class_def(&class_def.name, &class_def.body));
                }
                ast::Stmt::Assign(assign) => {
                    globals.extend(self.handle_assign(&assign.targets, &assign.value));
                }
                ast::Stmt::AnnAssign(ann) => {
                    globals.extend(self.handle_ann_assign(
                        &ann.target,
                        &ann.annotation,
                        ann.value.as_deref(),
                    ));
                }
                ast::Stmt::FunctionDef(func_def) => {
                    funcs.extend(self.handle_function_def(
                        &func_def.name,
                        &func_def.args,
                        &func_def.body,
                        func_def.returns.as_deref(),
                    ));
                }
                _ => {}
            }
        }

        let mut code = Vec::new();
        code.extend(imports);
        code.extend(macros);
        code.extend(structs);
        code.extend(globals);
        code.extend(funcs);
        Ok(code.join("\n"))
    }

    // ── 第一遍: 收集符号 ──

    fn collect_symbols(&mut self, body: &[rustpython_parser::ast::Stmt]) {
        use rustpython_parser::ast;
        for stmt in body {
            match stmt {
                ast::Stmt::ClassDef(class_def) => {
                    let mut members = HashMap::new();
                    for item in &class_def.body {
                        if let ast::Stmt::AnnAssign(ann_assign) = item {
                            if let ast::Expr::Name(name) = ann_assign.target.as_ref() {
                                let type_name = self.get_type_name(&ann_assign.annotation);
                                let is_ptr = type_name.contains('*') || type_name.contains("CPtr");
                                members.insert(
                                    name.id.to_string(),
                                    MemberInfo {
                                        type_name,
                                        is_pointer: is_ptr,
                                    },
                                );
                            }
                        }
                    }
                    self.symbol_table
                        .insert(class_def.name.to_string(), SymbolKind::Struct { members });
                }
                ast::Stmt::FunctionDef(func_def) => {
                    self.symbol_table
                        .insert(func_def.name.to_string(), SymbolKind::Function);
                }
                ast::Stmt::AnnAssign(ann_assign) => {
                    if let ast::Expr::Name(name) = ann_assign.target.as_ref() {
                        let type_name = self.get_type_name(&ann_assign.annotation);
                        let is_ptr = type_name.contains('*') || type_name.contains("CPtr");
                        self.symbol_table.insert(
                            name.id.to_string(),
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
            let name = &alias.name;
            if name.as_str() != "c" && name.as_str() != "t" {
                if name.as_str() == "stdio" {
                    code.push(format!("#include <{}.h>", name.as_str()));
                } else {
                    code.push(format!("#include \"{}.h\"", name.as_str()));
                }
            }
        }
        code
    }

    fn handle_import_from(
        &self,
        module: &Option<rustpython_parser::ast::Identifier>,
        names: &[rustpython_parser::ast::Alias],
        _level: &Option<rustpython_parser::ast::Int>,
    ) -> Vec<String> {
        let mut code = Vec::new();
        if let Some(mod_name) = module {
            let s = mod_name.as_str();
            if s != "c" && s != "t" {
                let path = s.replace('.', "/");
                let comment = if names.len() == 1 && names[0].name.as_str() == "*" {
                    format!("from {} import *", s)
                } else {
                    let ns: Vec<&str> = names.iter().map(|a| a.name.as_str()).collect();
                    format!("from {} import {}", s, ns.join(", "))
                };
                code.push(format!("#include \"{}.h\" // {}", path, comment));
            }
        }
        code
    }

    // ── 宏提取 ──

    fn extract_macros(&self, stmt: &rustpython_parser::ast::Stmt, code: &mut Vec<String>) {
        use rustpython_parser::ast;
        if let ast::Stmt::Expr(expr_stmt) = stmt {
            if let ast::Expr::Call(call) = expr_stmt.value.as_ref() {
                if let ast::Expr::Attribute(attr) = call.func.as_ref() {
                    if let ast::Expr::Name(name) = attr.value.as_ref() {
                        if name.id.as_str() == "c"
                            && attr.attr.as_str() == "Macro"
                            && call.args.len() >= 2
                        {
                            if let (ast::Expr::Constant(c0), ast::Expr::Constant(c1)) =
                                (&call.args[0], &call.args[1])
                            {
                                if let (ast::Constant::Str(n), ast::Constant::Str(v)) =
                                    (&c0.value, &c1.value)
                                {
                                    code.push(format!("#define {} {}", n, v));
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
                    .def
                    .annotation
                    .as_ref()
                    .map(|a| self.get_type_name(a))
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "int".to_string());
                if let Some(scope) = self.var_scopes.last_mut() {
                    scope.insert(arg.def.arg.to_string(), param_type.clone());
                }
                format!("{} {}", param_type, arg.def.arg)
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

    fn extract_return_type_from_state(&self, ret: &rustpython_parser::ast::Expr) -> String {
        use rustpython_parser::ast;
        if let ast::Expr::BinOp(binop) = ret {
            if matches!(&binop.op, ast::Operator::BitOr) {
                let left_t = self.get_type_name(&binop.left);
                if left_t == "c.State" || left_t.contains("c.State") {
                    self.get_type_name(&binop.right)
                } else {
                    left_t
                }
            } else {
                "void".to_string()
            }
        } else {
            "void".to_string()
        }
    }

    // ── 类定义 ──

    pub fn handle_class_def(
        &mut self,
        name: &rustpython_parser::ast::Identifier,
        body: &[rustpython_parser::ast::Stmt],
    ) -> Vec<String> {
        use rustpython_parser::ast;
        let mut code = Vec::new();
        let name_str = name.to_string();
        code.push(format!("struct {} {{", name_str));

        for item in body {
            match item {
                ast::Stmt::AnnAssign(ann_assign) => {
                    if let ast::Expr::Name(var_name) = ann_assign.target.as_ref() {
                        let type_name = self.get_type_name(&ann_assign.annotation);
                        if !type_name.is_empty() {
                            code.push(format!(
                                "    {} {};",
                                self.process_struct_member_type(
                                    &type_name,
                                    &var_name.id.to_string()
                                ),
                                var_name.id
                            ));
                        } else {
                            code.push(format!("    int {};", var_name.id));
                        }
                    }
                }
                ast::Stmt::FunctionDef(func_def) => {
                    if func_def.name.as_str() == "__init__" {
                        for stmt in &func_def.body {
                            if let ast::Stmt::AnnAssign(ann_assign) = stmt {
                                if let ast::Expr::Attribute(attr) = ann_assign.target.as_ref() {
                                    if let ast::Expr::Name(obj) = attr.value.as_ref() {
                                        if obj.id.as_str() == "self" {
                                            let type_name =
                                                self.get_type_name(&ann_assign.annotation);
                                            if !type_name.is_empty() {
                                                code.push(format!(
                                                    "    {} {};",
                                                    type_name, attr.attr
                                                ));
                                            } else {
                                                code.push(format!("    int {};", attr.attr));
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
            if arg.def.arg.as_str() != "self" {
                let param_type = arg
                    .def
                    .annotation
                    .as_ref()
                    .map(|a| self.get_type_name(a))
                    .filter(|t| !t.is_empty())
                    .unwrap_or_else(|| "int".to_string());
                params.push(format!("{} {}", param_type, arg.def.arg));
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
        use rustpython_parser::ast;
        let mut code = Vec::new();

        // 处理多重赋值: a, b = b, a
        if targets.len() == 1 {
            if let ast::Expr::Tuple(target_tuple) = &targets[0] {
                if let ast::Expr::Tuple(value_tuple) = value {
                    if target_tuple.elts.len() == 2 && value_tuple.elts.len() == 2 {
                        let t0 = self.handle_expr(&target_tuple.elts[0])[0].clone();
                        let t1 = self.handle_expr(&target_tuple.elts[1])[0].clone();
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
                ast::Expr::Name(name) => {
                    let value_code = self.handle_expr(value);
                    if value_code.is_empty() {
                        continue;
                    }
                    let val = &value_code[0];
                    let id_str = name.id.to_string();

                    if self.is_var_declared(&id_str) {
                        code.push(format!("{} = {};", id_str, val));
                    } else {
                        code.push(format!("int {} = {};", id_str, val));
                        if let Some(scope) = self.var_scopes.last_mut() {
                            scope.insert(id_str, "int".to_string());
                        }
                    }
                }
                ast::Expr::Attribute(attr) => {
                    let obj_code = self.handle_expr(&attr.value);
                    if obj_code.is_empty() {
                        continue;
                    }
                    let obj_str = &obj_code[0];
                    let is_ptr = self.check_is_pointer_expr(obj_str, &attr.value);
                    let value_code = self.handle_expr(value);
                    if value_code.is_empty() {
                        continue;
                    }
                    let val = &value_code[0];
                    if is_ptr {
                        code.push(format!("{}->{} = {};", obj_str, attr.attr, val));
                    } else {
                        code.push(format!("{}.{} = {};", obj_str, attr.attr, val));
                    }
                }
                ast::Expr::Subscript(sub) => {
                    let arr_code = self.handle_expr(&sub.value);
                    let idx_code = self.handle_expr(&sub.slice);
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
            rustpython_parser::ast::Expr::Name(name) => name.id.to_string(),
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
        if let Some(rustpython_parser::ast::Expr::List(list)) = value {
            let (base_type, array_str) = extract_array_size(&type_name);
            let base = if base_type.is_empty() {
                &type_name
            } else {
                &base_type
            };
            let elements: Vec<String> = list
                .elts
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
        _in_block: bool,
    ) -> Vec<String> {
        let mut code = Vec::new();

        use rustpython_parser::ast;
        for stmt in body {
            match stmt {
                ast::Stmt::Expr(expr_stmt) => {
                    let expr_code = self.handle_expr(&expr_stmt.value);
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
                ast::Stmt::If(if_stmt) => {
                    code.extend(self.handle_if(&if_stmt.test, &if_stmt.body, &if_stmt.orelse));
                }
                ast::Stmt::For(for_stmt) => {
                    code.extend(self.handle_for(&for_stmt.target, &for_stmt.iter, &for_stmt.body));
                }
                ast::Stmt::While(while_stmt) => {
                    code.extend(self.handle_while(&while_stmt.test, &while_stmt.body));
                }
                ast::Stmt::Break(_) => {
                    code.push("break;".to_string());
                }
                ast::Stmt::Continue(_) => {
                    code.push("continue;".to_string());
                }
                ast::Stmt::Return(return_stmt) => {
                    if let Some(v) = &return_stmt.value {
                        let vc = self.handle_expr(v);
                        if !vc.is_empty() {
                            code.push(format!("return {};", vc[0]));
                        } else {
                            code.push("return;".to_string());
                        }
                    } else {
                        code.push("return;".to_string());
                    }
                }
                ast::Stmt::Assign(assign) => {
                    code.extend(self.handle_assign(&assign.targets, &assign.value));
                }
                ast::Stmt::AugAssign(aug_assign) => {
                    code.extend(self.handle_aug_assign(
                        &aug_assign.target,
                        &aug_assign.op,
                        &aug_assign.value,
                    ));
                }
                ast::Stmt::AnnAssign(ann_assign) => {
                    let c = self.handle_ann_assign(
                        &ann_assign.target,
                        &ann_assign.annotation,
                        ann_assign.value.as_deref(),
                    );
                    if !c.is_empty() {
                        code.extend(c);
                    }
                }
                ast::Stmt::ClassDef(_) => {}
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
            if orelse.len() == 1 && matches!(&orelse[0], rustpython_parser::ast::Stmt::If(_)) {
                if let rustpython_parser::ast::Stmt::If(elif_stmt) = &orelse[0] {
                    code.push("else".to_string());
                    code.extend(self.handle_if(
                        &elif_stmt.test,
                        &elif_stmt.body,
                        &elif_stmt.orelse,
                    ));
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

        use rustpython_parser::ast;
        // 处理 range 循环
        if let ast::Expr::Call(call) = iter {
            if let ast::Expr::Name(name) = call.func.as_ref() {
                if name.id.as_str() == "range" {
                    let var_name = match target {
                        ast::Expr::Name(n) => n.id.to_string(),
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

                    let (start, stop, step) = match call.args.len() {
                        1 => (
                            "0".to_string(),
                            self.handle_expr(&call.args[0])[0].clone(),
                            "1".to_string(),
                        ),
                        2 => (
                            self.handle_expr(&call.args[0])[0].clone(),
                            self.handle_expr(&call.args[1])[0].clone(),
                            "1".to_string(),
                        ),
                        3 => (
                            self.handle_expr(&call.args[0])[0].clone(),
                            self.handle_expr(&call.args[1])[0].clone(),
                            self.handle_expr(&call.args[2])[0].clone(),
                        ),
                        _ => ("0".to_string(), "0".to_string(), "1".to_string()),
                    };

                    let cond_op = if step.starts_with('-') { ">" } else { "<" };
                    let inc_op = &step;

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
        if let rustpython_parser::ast::Expr::Subscript(sub) = iter {
            if let rustpython_parser::ast::Expr::Slice(slice) = sub.slice.as_ref() {
                if let rustpython_parser::ast::Expr::Name(var_name) = target {
                    if let rustpython_parser::ast::Expr::Name(base_var) = sub.value.as_ref() {
                        let start_idx = slice
                            .lower
                            .as_ref()
                            .map(|l| self.handle_expr(l))
                            .filter(|v| !v.is_empty())
                            .map(|v| v[0].clone())
                            .unwrap_or_else(|| "0".to_string());
                        code.push(format!(
                            "for (int __for_i = {}; {}[__for_i] != '\\0'; __for_i++) {{",
                            start_idx, base_var.id
                        ));
                        code.push(format!(
                            "    char {} = {}[__for_i];",
                            var_name.id, base_var.id
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
        }

        // 字符串直接遍历
        if let rustpython_parser::ast::Expr::Name(base_var) = iter {
            if let rustpython_parser::ast::Expr::Name(var_name) = target {
                code.push(format!(
                    "for (int __for_i = 0; {}[__for_i] != 0; __for_i++) {{",
                    base_var.id
                ));
                code.push(format!(
                    "    char {} = {}[__for_i];",
                    var_name.id, base_var.id
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
        use rustpython_parser::ast;
        let mut code = Vec::new();

        // 检查 do-while 模式
        if let ast::Expr::Constant(constant) = test {
            if let ast::Constant::Bool(true) = &constant.value {
                if let Some(last) = body.last() {
                    if let ast::Stmt::If(if_stmt) = last {
                        if if_stmt.orelse.is_empty() && if_stmt.body.len() == 1 {
                            if let ast::Stmt::Break(_) = &if_stmt.body[0] {
                                let cond_str = self.handle_expr(&if_stmt.test);
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
        use rustpython_parser::ast;
        match stmt {
            ast::Stmt::Assign(assign) => self.handle_assign(&assign.targets, &assign.value),
            ast::Stmt::AugAssign(aug) => self.handle_aug_assign(&aug.target, &aug.op, &aug.value),
            ast::Stmt::AnnAssign(ann) => {
                self.handle_ann_assign(&ann.target, &ann.annotation, ann.value.as_deref())
            }
            ast::Stmt::If(if_stmt) => self.handle_if(&if_stmt.test, &if_stmt.body, &if_stmt.orelse),
            ast::Stmt::For(for_stmt) => {
                self.handle_for(&for_stmt.target, &for_stmt.iter, &for_stmt.body)
            }
            ast::Stmt::While(while_stmt) => self.handle_while(&while_stmt.test, &while_stmt.body),
            ast::Stmt::Break(_) => vec!["break;".to_string()],
            ast::Stmt::Continue(_) => vec!["continue;".to_string()],
            ast::Stmt::Return(ret) => {
                if let Some(v) = &ret.value {
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
            ast::Stmt::Expr(expr) => {
                let mut r = Vec::new();
                for e in self.handle_expr(&expr.value) {
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
        use rustpython_parser::ast;
        let mut code = Vec::new();
        let op_sym = self.get_aug_op_symbol(op);
        let val = self.handle_expr(value);
        let val_str = if val.is_empty() { "0" } else { &val[0] };

        match target {
            ast::Expr::Name(name) => {
                code.push(format!("{} {}= {};", name.id, op_sym, val_str));
            }
            ast::Expr::Subscript(sub) => {
                let arr_c = self.handle_expr(&sub.value);
                let idx_c = self.handle_expr(&sub.slice);
                if !arr_c.is_empty() && !idx_c.is_empty() {
                    code.push(format!(
                        "{}[{}] {}= {};",
                        arr_c[0], idx_c[0], op_sym, val_str
                    ));
                }
            }
            ast::Expr::Attribute(attr) => {
                let obj_c = self.handle_expr(&attr.value);
                if !obj_c.is_empty() {
                    let is_ptr = self.check_is_pointer_expr(&obj_c[0], &attr.value);
                    if is_ptr {
                        code.push(format!(
                            "{}->{} {}= {};",
                            obj_c[0], attr.attr, op_sym, val_str
                        ));
                    } else {
                        code.push(format!(
                            "{}.{} {}= {};",
                            obj_c[0], attr.attr, op_sym, val_str
                        ));
                    }
                }
            }
            _ => {}
        }
        code
    }

    // ── 运算符映射 ──

    pub(super) fn get_op_symbol(&self, op: &rustpython_parser::ast::Operator) -> &'static str {
        let name = operator_type_name(op);
        OPERATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("+")
    }

    pub(super) fn get_aug_op_symbol(&self, op: &rustpython_parser::ast::Operator) -> &'static str {
        let name = operator_type_name(op);
        AUG_OPERATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("+")
    }

    pub(super) fn get_comparator_symbol(&self, op: &rustpython_parser::ast::CmpOp) -> &'static str {
        let name = cmp_op_type_name(op);
        COMPARATOR_MAP
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, val)| *val)
            .unwrap_or("==")
    }

    pub(super) fn get_unary_op_symbol(&self, op: &rustpython_parser::ast::UnaryOp) -> &'static str {
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
        if let rustpython_parser::ast::Expr::Name(name) = expr {
            let id_str = name.id.as_str();
            if id_str == "self" {
                return true;
            }
            for scope in self.var_scopes.iter().rev() {
                if let Some(t) = scope.get(id_str) {
                    if t.contains('*') {
                        return true;
                    }
                }
            }
            if let Some(sym) = self.symbol_table.get(id_str) {
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
