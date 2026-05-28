//! TransPyC 错误类型

use std::fmt;

/// 翻译过程中的错误
#[derive(Debug)]
pub enum Error {
    /// Python 源码解析失败
    Parse(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(msg) => write!(f, "解析错误: {}", msg),
        }
    }
}

impl std::error::Error for Error {}
