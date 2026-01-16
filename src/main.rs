use std::{
    io::{self, Write}, // 导入标准输入输出相关模块
};

use anyhow::{Context, Result}; // 错误处理库
use clap::{Arg, Command}; // 命令行参数解析库

use crate::search::Searcher;

mod search;

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
    // 创建搜索器
    let searcher = Searcher::new(
        pattern,
        search::Options {
            show_line_number: true,
            count_only: false,
            case_ignore: false,
            match_only: false,
        },
    )?;

    // 获取输出格式的枚举类型
    let format = searcher.output_format();

    // 从标准输入读取数据
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin.lock());
    let mut writer = io::BufWriter::new(io::stdout());

    let mut count = 0;

    // 使用迭代器模式，逐行搜索
    for result in searcher.search(reader) {
        let search_result = result.context("Failed to read or search line")?;
        count += 1;

        // 使用枚举 match
        search_result.format_to(&mut writer, &format)?;
    }

    writer.flush()?;
    Ok(count)
}
