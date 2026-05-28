//! TransPyC —— Python 子集 → C 源码翻译器
//!
//! # 示例
//!
//! ```rust
//! use trans_py_c::Translator;
//!
//! let source = r#"
//! def add(a: int, b: int) -> int:
//!     return a + b
//! "#;
//!
//! let mut t = Translator::new();
//! let c_code = t.generate_c_code(source).expect("翻译失败");
//! assert!(c_code.contains("int add(int a, int b)"));
//!
//! // 翻译后可读取符号表和调试日志
//! println!("{:?}", t.symbol_table);
//! ```
//!
//! # 模块
//!
//! - `Translator` — 翻译入口
//! - `error::Error` — 错误类型
//! - `core::types` — 符号表、作用域等数据结构
//! - `includes::gramma` / `includes::types` — C 语法/类型映射

pub mod constants;
pub mod core;
pub mod error;
pub mod includes;

pub use core::translator::Translator;
pub use core::types::{FunctionReturnTypes, MemberInfo, SymbolKind, SymbolTable, VarScopes};
pub use error::Error;
