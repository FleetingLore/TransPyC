//! C语法

pub struct Asm {
    pub code: String,
    pub args: Vec<Box<dyn std::any::Any>>,
}

impl Asm {
    pub fn new(code: String, args: Vec<Box<dyn std::any::Any>>) -> Self {
        Self { code, args }
    }
}

pub struct State;

impl State {
    pub fn new() -> Self {
        Self
    }
}

pub struct ClassPoint;

impl ClassPoint {
    pub fn new() -> Self {
        Self
    }
}

/// 类列表
/// 用于存储项目的列表容器
pub struct ClassList {
    /// 存储项目的向量
    pub items: Vec<Box<dyn std::any::Any>>,
}

impl ClassList {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加项目到列表
    pub fn append(&mut self, item: Box<dyn std::any::Any>) {
        self.items.push(item);
    }
}

pub struct Memory {
    pub addr: usize,
}

impl Memory {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }
}

/// 解引用
/// 用于解引用指针
pub struct Dereference {
    /// 被解引用的指针
    pub ptr: Box<dyn std::any::Any>,
}

impl Dereference {
    pub fn new(ptr: Box<dyn std::any::Any>) -> Self {
        Self { ptr }
    }
}

/// 引用
/// 用于创建变量的引用
pub struct Reference {
    /// 被引用的变量
    pub var: Box<dyn std::any::Any>,
}

impl Reference {
    pub fn new(var: Box<dyn std::any::Any>) -> Self {
        Self { var }
    }
}

/// 宏
/// 用于定义宏名称和值
pub struct Macro {
    /// 宏名称
    pub name: String,
    /// 宏值
    pub value: String,
}

impl Macro {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

pub struct Ast;

impl Ast {
    pub fn new() -> Self {
        Self
    }
}

/// 类型转换
/// 用于将值转换为指定类型
pub struct TypeCast {
    /// 目标类型名称
    pub type_name: String,
    /// 被转换的值
    pub value: Box<dyn std::any::Any>,
}

impl TypeCast {
    pub fn new(type_name: String, value: Box<dyn std::any::Any>) -> Self {
        Self { type_name, value }
    }
}

pub struct Esp;

impl Esp {
    pub fn new() -> Self {
        Self
    }
}

pub struct Ebp;

impl Ebp {
    pub fn new() -> Self {
        Self
    }
}

pub struct Addr {
    pub addr: usize,
}

impl Addr {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }
}

/// 指针
/// 包含地址、值和类型信息的指针结构
pub struct Ptr {
    /// 指针地址
    pub addr: usize,
    /// 指针指向的值
    pub value: Option<Box<dyn std::any::Any>>,
    /// 指针类型
    pub ptr_type: Option<String>,
}

impl Ptr {
    pub fn new(addr: usize) -> Self {
        Self {
            addr,
            value: None,
            ptr_type: None,
        }
    }

    pub fn with_value(addr: usize, value: Box<dyn std::any::Any>) -> Self {
        Self {
            addr,
            value: Some(value),
            ptr_type: None,
        }
    }

    pub fn with_type(addr: usize, ptr_type: String) -> Self {
        Self {
            addr,
            value: None,
            ptr_type: Some(ptr_type),
        }
    }
}

/// 转换
/// 用于指针转换
pub struct Cast {
    /// 被转换的指针
    pub ptr: Box<dyn std::any::Any>,
}

impl Cast {
    pub fn new(ptr: Box<dyn std::any::Any>) -> Self {
        Self { ptr }
    }
}

/// 设置
/// 用于键值对存储
pub struct Set {
    /// 键
    pub key: String,
    /// 值
    pub value: Box<dyn std::any::Any>,
}

impl Set {
    pub fn new(key: String, value: Box<dyn std::any::Any>) -> Self {
        Self { key, value }
    }
}
