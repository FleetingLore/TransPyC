// src/utils/helpers.rs

use crate::constants::ERROR_MESSAGES;
use std::fs;
use std::path::Path;
use std::process::Command;

/// 检测文件类型
pub fn detect_file_type(file_path: &str) -> String {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .map(|ext| format!(".{}", ext))
        .unwrap_or_default()
}

/// 执行命令
pub fn execute_command(
    command: &str,
    shell: bool,
    capture_output: bool,
    text: bool,
) -> Option<String> {
    let mut cmd = if shell {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        cmd
    } else {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let mut cmd = Command::new(parts[0]);
        cmd.args(&parts[1..]);
        cmd
    };

    if capture_output {
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
    }

    match cmd.output() {
        Ok(output) => {
            if text {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Some(format!("{:?}", output.stdout))
            }
        }
        Err(e) => {
            let error_msg = ERROR_MESSAGES
                .iter()
                .find(|(key, _)| *key == "COMPILE_FAILED")
                .map(|(_, msg)| *msg)
                .unwrap_or("Compilation failed");
            eprintln!("{}: {}", error_msg, e);
            None
        }
    }
}

/// 获取文件内容
pub fn get_file_content(file_path: &str) -> Option<String> {
    match fs::read_to_string(file_path) {
        Ok(content) => Some(content),
        Err(e) => {
            let error_msg = ERROR_MESSAGES
                .iter()
                .find(|(key, _)| *key == "FAILED_PARSE")
                .map(|(_, msg)| *msg)
                .unwrap_or("Failed to parse file");
            eprintln!("{} {}: {}", error_msg, file_path, e);
            None
        }
    }
}

/// 写入文件内容
pub fn write_file_content(file_path: &str, content: &str) -> bool {
    match fs::write(file_path, content) {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Failed to write file {}: {}", file_path, e);
            false
        }
    }
}

/// 追加文件内容
pub fn append_file_content(file_path: &str, content: &str) -> bool {
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
    {
        Ok(file) => {
            use std::io::Write;
            let mut file = file;
            match writeln!(file, "{}", content) {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Failed to append to file {}: {}", file_path, e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to append to file {}: {}", file_path, e);
            false
        }
    }
}

/// 检查是否是标准库
pub fn is_standard_library(module_name: &str) -> bool {
    let standard_libs = ["stdio", "stdlib", "string", "math", "time", "sys", "os"];
    standard_libs.contains(&module_name)
}

/// 提取数组大小
pub fn extract_array_size(type_name: &str) -> (String, String) {
    let mut array_sizes = Vec::new();
    let mut base_type = type_name.to_string();

    while let Some(start) = base_type.rfind('[') {
        if let Some(end) = base_type.rfind(']') {
            let size = &base_type[start + 1..end];
            array_sizes.push(size.to_string());
            base_type = base_type[..start].to_string();
        } else {
            break;
        }
    }

    let array_size_str: String = array_sizes
        .iter()
        .rev()
        .map(|size| format!("[{}]", size))
        .collect();

    (base_type, array_size_str)
}

/// 检查存储类修饰符
pub fn check_storage_class(type_name: &str) -> (String, String) {
    let type_name = type_name.trim();

    if type_name.starts_with("static ") || type_name.starts_with("extern ") {
        let parts: Vec<&str> = type_name.splitn(2, ' ').collect();
        let storage_class = parts[0].to_string();
        let type_part = parts.get(1).unwrap_or(&"").to_string();
        (storage_class, type_part)
    } else {
        (String::new(), type_name.to_string())
    }
}

/// 构建数组初始化代码
pub fn build_array_initialization(elements: &[String], _use_single_quote: bool) -> String {
    let elements_str = elements.join(", ");
    format!("{{ {} }}", elements_str)
}

/// 格式化错误消息
pub fn format_error_message(error_type: &str, args: &[String]) -> String {
    if let Some((_, message)) = ERROR_MESSAGES.iter().find(|(key, _)| *key == error_type) {
        if args.is_empty() {
            message.to_string()
        } else {
            // 简单的格式化替换
            let mut result = message.to_string();
            for (i, arg) in args.iter().enumerate() {
                result = result.replace(&format!("{{{}}}", i), arg);
            }
            result
        }
    } else {
        format!("Unknown error: {}", error_type)
    }
}

/// 验证命令行参数
pub fn validate_args(has_input: bool, has_output: bool) -> (bool, Option<&'static str>) {
    if !has_input || !has_output {
        let msg = ERROR_MESSAGES
            .iter()
            .find(|(key, _)| *key == "MISSING_ARGS")
            .map(|(_, msg)| *msg);
        (false, msg)
    } else {
        (true, None)
    }
}

/// 获取缩进
pub fn get_indentation(level: usize) -> String {
    "    ".repeat(level)
}
