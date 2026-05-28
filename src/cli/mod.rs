//! 命令行接口
//!
//! ```text
//! trans_py_c [OPTIONS] [FILES]     翻译 Python → C
//! trans_py_c init <PATH>           初始化新项目
//! ```

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::Deserialize;

// ── CLI ──

/// TransPyC —— Python 子集 → C 源码翻译器
#[derive(Parser)]
#[command(name = "trans_py_c", version, about)]
pub struct Args {
    /// 输入 Python 文件 (支持 glob)
    #[arg()]
    pub files: Vec<String>,

    /// 输出目录 (默认: 当前目录, 有配置时为项目 target/)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// TOML 配置文件或项目目录
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    /// 详细输出
    #[arg(short, long)]
    pub verbose: bool,

    /// 输出翻译器调试信息
    #[arg(long)]
    pub debug: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// 初始化一个新项目 (在当前目录或指定路径)
    Init {
        /// 项目路径 (默认: 当前目录)
        path: Option<PathBuf>,
    },
}

// ── 配置 (TOML) ──

/// TransPyC.toml 顶层结构
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    /// 项目名称
    pub _name: Option<String>,
    /// 输出目录 (相对于配置文件所在目录)
    pub output: Option<String>,
    /// 文件组
    #[serde(default)]
    pub files: Vec<FileGroup>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FileGroup {
    /// 输入: Python 文件或 glob
    pub input: String,
    /// 覆盖输出目录
    pub output: Option<String>,
}

// ── 任务 ──

#[derive(Debug)]
pub struct Job {
    pub input: PathBuf,
    pub output: PathBuf,
}

impl Job {
    fn from_path(input: PathBuf, out_dir: &PathBuf) -> Self {
        let stem = input
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let output = out_dir.join(format!("{}.c", stem));
        Self { input, output }
    }
}

// ── Config 发现 ──

/// 查找 TransPyC.toml：先看显式路径，再看当前目录
fn find_config_toml(explicit: Option<&PathBuf>) -> Option<(PathBuf, Config)> {
    let candidates: Vec<PathBuf> = match explicit {
        Some(p) if p.is_dir() => vec![p.join("TransPyC.toml")],
        Some(p) => vec![p.clone()],
        None => vec![PathBuf::from("TransPyC.toml")],
    };

    for path in candidates {
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(cfg) => return Some((path, cfg)),
                    Err(e) => eprintln!("警告: {} 解析失败: {}", path.display(), e),
                },
                Err(e) => eprintln!("警告: 无法读取 {}: {}", path.display(), e),
            }
        }
    }
    None
}

// ── Glob ──

fn expand_glob(pattern: &str) -> Vec<PathBuf> {
    match glob::glob(pattern) {
        Ok(paths) => paths
            .filter_map(|r| r.ok())
            .filter(|p| p.extension().map_or(false, |e| e == "py"))
            .collect(),
        Err(e) => {
            eprintln!("警告: glob 无效 '{}': {}", pattern, e);
            Vec::new()
        }
    }
}

// ── 入口 ──

/// 处理 `init` 子命令
pub fn handle_init(path: &Option<PathBuf>) {
    let target = path.clone().unwrap_or_else(|| PathBuf::from("."));
    if target.join("TransPyC.toml").exists() {
        eprintln!("TransPyC.toml 已存在于 {}", target.display());
        return;
    }
    let _ = std::fs::create_dir_all(&target);

    let name = target
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "project".to_string());

    let toml = format!(
        r#"# TransPyC 项目配置
name = "{}"
output = "target"

[[files]]
input = "*.py"
"#,
        name
    );
    let main_py = "# TransPyC 入口\n\ndef main() -> int:\n    return 0\n";

    std::fs::write(target.join("TransPyC.toml"), toml).ok();
    std::fs::write(target.join("main.py"), main_py).ok();
    println!("初始化项目: {} (name={})", target.display(), name);
}

/// 合并 CLI + TOML → 任务列表
pub fn collect_jobs(args: &Args) -> Vec<Job> {
    let mut jobs = Vec::new();
    let default_output = PathBuf::from(".");

    // 有配置文件时
    if let Some((config_path, cfg)) = find_config_toml(args.config.as_ref()) {
        let base = config_path.parent().unwrap_or(&default_output);
        let global_out = base.join(cfg.output.as_deref().unwrap_or("target"));

        for group in &cfg.files {
            let out_dir = group
                .output
                .as_ref()
                .map(|o| base.join(o))
                .unwrap_or_else(|| global_out.clone());

            for input in expand_glob(&base.join(&group.input).to_string_lossy()) {
                jobs.push(Job::from_path(input, &out_dir));
            }
        }

        if args.verbose {
            println!("配置: {}", config_path.display());
            println!("输出目录: {}", global_out.display());
            println!("共 {} 个翻译任务", jobs.len());
        }
    } else {
        // 无配置文件: 直接翻译命令行指定的文件
        let out_dir = args
            .output
            .clone()
            .unwrap_or_else(|| default_output.clone());

        for pattern in &args.files {
            for input in expand_glob(pattern) {
                jobs.push(Job::from_path(input, &out_dir));
            }
        }

        if args.verbose && !args.files.is_empty() {
            println!("输出目录: {}", out_dir.display());
            println!("共 {} 个翻译任务", jobs.len());
        }
    }

    if jobs.is_empty() && args.command.is_none() {
        println!("用法: trans_py_c [文件...] [-c 配置]");
        println!("      trans_py_c init <项目路径>");
        println!("      trans_py_c --help");
    }

    jobs
}
