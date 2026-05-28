use std::fs;

use clap::Parser;
use trans_py_c::core::translator::Translator;

mod cli;

fn main() {
    let args = cli::Args::parse();

    if let Some(cli::Command::Init { path }) = &args.command {
        cli::handle_init(path);
        return;
    }

    let jobs = cli::collect_jobs(&args);
    if jobs.is_empty() {
        return;
    }

    for job in &jobs {
        let source = match fs::read_to_string(&job.input) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("错误: 无法读取 {}: {}", job.input.display(), e);
                continue;
            }
        };

        let mut translator = Translator::new();
        let c_code = translator.generate_c_code(&source);

        if let Some(parent) = job.output.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match fs::write(&job.output, &c_code) {
            Ok(_) => {
                if args.verbose {
                    println!("{} → {}", job.input.display(), job.output.display());
                } else {
                    println!("{}", job.output.display());
                }
            }
            Err(e) => eprintln!("错误: 写入 {} 失败: {}", job.output.display(), e),
        }

        if args.debug && !translator.debug_logs.is_empty() {
            eprintln!("--- 调试: {} ---", job.input.display());
            for log in &translator.debug_logs {
                eprintln!("  {}", log);
            }
        }
    }

    if args.verbose {
        println!("完成: {} 个文件", jobs.len());
    }
}
