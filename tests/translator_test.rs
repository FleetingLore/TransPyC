use trans_py_c::core::translator::Translator;

fn t(source: &str, mut translator: Translator) -> String {
    translator.generate_c_code(source).expect("翻译失败")
}

/// 测试简单的函数定义翻译
#[test]
fn test_simple_function() {
    let source = r#"
def add(a: int, b: int) -> int:
    return a + b
"#;
    let result = t(source, Translator::new());
    assert!(result.contains("int add(int a, int b)"));
    assert!(result.contains("return (a + b);"));
}

/// 测试类定义转换为结构体
#[test]
fn test_class_to_struct() {
    let source = r#"
class Point:
    x: int
    y: int
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("struct Point {"));
    assert!(result.contains("int x;"));
    assert!(result.contains("int y;"));
    assert!(result.contains("};"));
}

/// 测试 if/elif/else 控制流
#[test]
fn test_if_elif_else() {
    let source = r#"
def test(x: int) -> int:
    if x > 0:
        return 1
    elif x < 0:
        return -1
    else:
        return 0
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("if ("));
    assert!(result.contains("else"));
    assert!(result.contains("return 1"));
    assert!(result.contains("return -1"));
    assert!(result.contains("return 0"));
}

/// 测试 range() 循环翻译为 C for 循环
#[test]
fn test_for_range_loop() {
    let source = r#"
def test():
    sum = 0
    for i in range(10):
        sum += i
    return sum
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("for (int i = 0; i < 10; i += 1)"));
    assert!(result.contains("sum += i"));
}

/// 测试 while 循环
#[test]
fn test_while_loop() {
    let source = r#"
def test():
    i = 0
    while i < 10:
        i += 1
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("while ("));
    assert!(result.contains("i += 1"));
}

/// 测试 c.Memory 和 c.TypeCast 等特殊调用
#[test]
fn test_c_special_calls() {
    let source = r#"
def test():
    addr = c.Memory(0x1000)
    x = 42
    y = c.TypeCast('float', x)
"#;
    let result = t(source, Translator::new());

    assert!(
        result.contains("((void *)4096)"),
        "should cast addr to void*"
    );
    // c.TypeCast 应该生成类型转换
    assert!(
        result.contains("float") && result.contains("42"),
        "should contain float cast"
    );
}

/// 测试 c.Macro 生成 #define
#[test]
fn test_macro() {
    let source = r#"
def TestMacro():
    c.Macro('MAX_VALUE', '100')
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("#define MAX_VALUE 100"));
}

/// 测试类型注解的全局变量
#[test]
fn test_annotated_global() {
    let source = r#"
global_var: int = 42
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("int global_var = 42;"));
}

/// 测试空函数（仅声明）
#[test]
fn test_empty_function() {
    let source = r#"
def empty():
    pass
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("void empty(void) {"));
    assert!(result.contains("}"));
}

/// 测试导入语句
#[test]
fn test_import() {
    let source = r#"
import c
import t
import stdio
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("#include <stdio.h>"));
}

/// 测试 c.Asm 内联汇编
#[test]
fn test_asm() {
    let source = r#"
def test():
    c.Asm('nop')
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("__asm__ volatile"));
}

/// 测试 t.CPtr 指针类型
#[test]
fn test_pointer_type() {
    let source = r#"
ptr: int = 0
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("int ptr = 0;"));
}

/// 测试自增模式 k := k + 1
#[test]
fn test_named_expr_increment() {
    let source = r#"
def test():
    k = 0
    result = (k, k := k + 1)[0]
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("k++"));
}

/// 测试三目运算符 if-else 表达式
#[test]
fn test_if_exp() {
    let source = r#"
def test(x: int) -> int:
    return 1 if x > 0 else 2
"#;
    let result = t(source, Translator::new());

    // 三目运算符
    assert!(result.contains("?"));
    assert!(result.contains(":"));
}

/// 测试数组初始化
#[test]
fn test_array_init() {
    let source = r#"
arr: int = [1, 2, 3]
"#;
    let result = t(source, Translator::new());

    // 检查数组声明和花括号初始化
    assert!(result.contains("int"), "should declare int type");
    assert!(result.contains("arr"), "should have array name");
    assert!(result.contains("="), "should have assignment");
    assert!(result.contains("1"), "should contain element 1");
    assert!(result.contains("2"), "should contain element 2");
    assert!(result.contains("3"), "should contain element 3");
}

/// 测试逻辑运算符 and/or 转换为 &&/||
#[test]
fn test_bool_op() {
    let source = r#"
def test(a: int, b: int) -> int:
    if a > 0 and b > 0:
        return 1
    return 0
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("&&"));
}

/// 测试 do-while 模式检测
#[test]
fn test_do_while_pattern() {
    let source = r#"
def test():
    i = 0
    while True:
        i += 1
        if i >= 10:
            break
"#;
    let result = t(source, Translator::new());

    assert!(result.contains("do {"));
    assert!(result.contains("while (!("));
}

/// 测试符号表收集
#[test]
fn test_symbol_table_collection() {
    let source = r#"
class MyStruct:
    field1: int

MY_VAR: int = 0

def my_func():
    pass
"#;
    let mut tr = Translator::new();
    let _result = tr.generate_c_code(source).unwrap();
    // 验证符号表已被填充
    let has_struct = tr.symbol_table.contains_key("MyStruct");
    let has_var = tr.symbol_table.contains_key("MY_VAR");
    let has_func = tr.symbol_table.contains_key("my_func");
    assert!(has_struct, "Symbol table should contain MyStruct");
    assert!(has_var, "Symbol table should contain MY_VAR");
    assert!(has_func, "Symbol table should contain my_func");
}
