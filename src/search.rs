use anyhow::{Context, Result};
use regex::Regex;
use std::{
    io::{BufRead, Write},
    marker,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    // 只计数，不输出具体行
    CountOnly,
    // 只输出匹配的文本片段（类似 grep -o）
    MatchOnly,
    // 输出完整行，带行号（默认）
    LineNumbered,
    // 输出完整行，不带行号
    FullLine,
}

// 从用户选项转换为格式化策略
// 优先级：count_only > match_only > show_line_number > full_line
impl From<&Options> for OutputFormat {
    fn from(opts: &Options) -> Self {
        if opts.count_only {
            OutputFormat::CountOnly
        } else if opts.match_only {
            OutputFormat::MatchOnly
        } else if opts.show_line_number {
            OutputFormat::LineNumbered
        } else {
            OutputFormat::FullLine
        }
    }
}

// 单次搜索的结果，包含行号、行内容和所有匹配位置
pub struct SearchResult {
    pub line_number: usize,
    pub line: String,
    pub matches: Vec<(usize, usize)>,
}

impl SearchResult {
    // 获取所有匹配的文本片段
    pub fn match_texts(&self) -> Vec<&str> {
        self.matches
            .iter()
            .map(|&(start, end)| &self.line[start..end])
            .collect()
    }

    // 根据输出格式格式化到writer
    // 使用 match 表达式替代 if-else，更清晰且易扩展
    pub fn format_to<W: Write>(&self, writer: &mut W, format: &OutputFormat) -> Result<()> {
        match format {
            OutputFormat::CountOnly => {}
            OutputFormat::MatchOnly => {
                for match_text in self.match_texts() {
                    writeln!(writer, "{}", match_text)?;
                }
            }
            OutputFormat::LineNumbered => {
                writeln!(writer, "{}: {}", self.line_number, self.line.trim_end())?;
            }
            OutputFormat::FullLine => {
                writeln!(writer, "{}", self.line.trim_end())?;
            }
        }
        Ok(())
    }
}

// 用户配置选项（从命令行参数来）
// 保留这个结构体用于配置管理，然后转换为 OutputFormat 使用
#[derive(Clone, Debug, Default)]
pub struct Options {
    // 是否显示行号
    pub show_line_number: bool,
    // 是否仅显示匹配数量（不输出具体行）
    pub count_only: bool,
    // 是否大小写不敏感
    pub case_ignore: bool,
    // 是否只输出匹配的部分
    pub match_only: bool,
}

impl Options {
    // 获取对应的输出格式
    pub fn output_format(&self) -> OutputFormat {
        OutputFormat::from(self)
    }
}

// 搜索器，持有正则和配置选项，负责创建搜索迭代器
pub struct Searcher {
    regex: Regex,
    opts: Options,
}

impl Searcher {
    pub fn new(pattern: &str, opts: Options) -> Result<Searcher> {
        let pattern = match opts.case_ignore {
            true => format!("(?i){}", pattern),
            false => pattern.to_string(),
        };

        let regex = Regex::new(&pattern).context("Failed to compile regex pattern")?;

        Ok(Searcher { regex, opts })
    }

    // 创建一个搜索迭代器，从给定的reader中逐行搜索
    pub fn search<'a, R: BufRead + 'a>(&'a self, reader: R) -> SearchIter<'a, R> {
        SearchIter::new(self, reader)
    }

    // 搜索单行（内部使用）
    fn search_line(&self, line_number: usize, line: String) -> Option<SearchResult> {
        let matches: Vec<(usize, usize)> = self
            .regex
            .find_iter(&line)
            .map(|m| (m.start(), m.end()))
            .collect();

        if matches.is_empty() {
            return None;
        }

        Some(SearchResult {
            line_number,
            line,
            matches,
        })
    }

    // 获取输出格式
    pub fn output_format(&self) -> OutputFormat {
        self.opts.output_format()
    }
}

// 搜索迭代器，实现Iterator trait
// 每次迭代返回一个匹配的行
// 使用迭代器链实现，而不是手动loop，更符合Rust习惯
pub struct SearchIter<'a, R> {
    inner: Box<dyn Iterator<Item = Result<SearchResult>> + 'a>,
    _phantom: marker::PhantomData<R>,
}

impl<'a, R: BufRead + 'a> SearchIter<'a, R> {
    fn new(searcher: &'a Searcher, reader: R) -> Self {
        // 使用迭代器链：lines() -> enumerate() -> filter_map()
        let inner = Box::new(
            reader
                .lines()
                .enumerate()
                .filter_map(move |(idx, line_result)| {
                    let line_number = idx + 1;
                    match line_result {
                        Ok(line) => searcher.search_line(line_number, line).map(Ok),
                        Err(e) => Some(Err(e.into())),
                    }
                }),
        );

        SearchIter {
            inner,
            _phantom: marker::PhantomData,
        }
    }
}

impl<'a, R> Iterator for SearchIter<'a, R> {
    type Item = Result<SearchResult>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
