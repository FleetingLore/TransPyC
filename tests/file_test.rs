//! 文件级集成测试: examples/<项目>/main.py → expected.c 对比

use std::fs;
use std::path::PathBuf;

use trans_py_c::core::translator::Translator;

fn translate(input: &PathBuf) -> String {
    let source = fs::read_to_string(input).expect("无法读取输入文件");
    let mut t = Translator::new();
    t.generate_c_code(&source).expect("翻译失败")
}

fn assert_match(name: &str, got: &str, expected: &str) {
    let got_norm = got
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let exp_norm = expected
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    if got_norm != exp_norm {
        let gl: Vec<&str> = got_norm.lines().collect();
        let el: Vec<&str> = exp_norm.lines().collect();
        let mut diff = vec![format!("{}: 输出不匹配", name)];
        for i in 0..gl.len().max(el.len()) {
            let g = gl.get(i).copied().unwrap_or("<缺失>");
            let e = el.get(i).copied().unwrap_or("<缺失>");
            if g != e {
                diff.push(format!("L{}: 期望=`{}` 实际=`{}`", i + 1, e, g));
            }
        }
        panic!("{}", diff.join("\n"));
    }
}

fn file_test(name: &str) {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples")).join(name);
    let input = dir.join("main.py");
    let expected = dir.join("expected.c");

    let got = translate(&input);
    let exp = fs::read_to_string(&expected).unwrap_or_else(|e| {
        panic!(
            "缺少期望输出 {}: {}\n提示: cargo run -- -c examples/{}/TransPyC.toml",
            expected.display(),
            e,
            name
        )
    });
    assert_match(name, &got, &exp);
}

#[test]
fn test_example1() {
    file_test("example1");
}
#[test]
fn test_simple() {
    file_test("test_simple");
}
#[test]
fn test_hello() {
    file_test("hello");
}
#[test]
fn test_variables() {
    file_test("variables");
}
#[test]
fn test_struct() {
    file_test("struct");
}
#[test]
fn test_control_flow() {
    file_test("control_flow");
}
#[test]
fn test_pointer() {
    file_test("pointer");
}
#[test]
fn test_asm_macro() {
    file_test("asm_macro");
}
