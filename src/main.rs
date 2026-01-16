use std::{
    io::{self, BufRead, Write}, // 导入标准输入输出相关模块
};

use anyhow::{Context, Result}; // 错误处理库
use clap::{Arg, Command}; // 命令行参数解析库
use regex::Regex; // 正则表达式库

fn main() -> Result<()> {
    // 构建命令行参数解析器
    let matches = Command::new("mrustgrep")
        .version("0.1.0")
        .author("Your Name")
        .about("A simple Rust implementation of grep")
        .arg(
            Arg::new("pattern")
                .required(true)
                .index(1)
                .help("The pattern to search for"), // 需要查找的模式
        )
        .get_matches();

    // 获取命令行参数中的 pattern
    let pattern = matches
        .get_one::<String>("pattern")
        .context("Failed to get pattern")?;

    // 执行主逻辑，处理错误
    match run(pattern) {
        Ok(count) => {
            eprintln!("Total matched lines: {}", count);
            Ok(())
        }
        Err(e) => {
            eprintln!("Application error: {e}");
            std::process::exit(1);
        }
    }
}

// 主运行逻辑，接收正则模式，返回匹配的行数
fn run(pattern: &str) -> Result<usize> {
    // 编译正则表达式
    let re = Regex::new(pattern).context("regex invalid")?;

    let _stdin = io::stdin(); // 获取标准输入

    // 创建带缓冲的读取器和写入器
    let reader = io::BufReader::new(_stdin.lock());
    let mut writer = io::BufWriter::new(io::stdout());

    let mut count = 0; // 匹配行计数
    let mut line_number = 0; // 当前行号

    // 循环读取每一行进行匹配
    for line_result in reader.lines() {
        let line = line_result.context("Failed to read line")?;
        line_number += 1;

        if re.is_match(&line) {
            count += 1;
            writeln!(writer, "{}: {}", line_number, line.trim_end())
                .context("Failed to write output")?;
        }
    }

    Ok(count)
}
