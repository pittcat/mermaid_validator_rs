use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mermaid_validator::preview_validator::scan_markdown_for_mermaid;
use mermaid_validator::response_builder::invalid_result;

fn bench_string_parsing(c: &mut Criterion) {
    const MULTI_LINE_MARKDOWN: &str = r#"
## Section 1

```mermaid
graph TD
A --> B
```

## Section 2

Some text here.

```mermaid
graph LR
X --> Y
```
"#;

    c.bench_function("lines_iteration", |b| {
        b.iter(|| {
            let markdown = black_box(MULTI_LINE_MARKDOWN);
            let _line_count = markdown.lines().count();
        });
    });

    c.bench_function("trim_and_clone", |b| {
        b.iter(|| {
            let markdown = black_box(MULTI_LINE_MARKDOWN);
            let _ = markdown.trim().to_string();
        });
    });

    c.bench_function("string_join", |b| {
        b.iter(|| {
            let markdown = black_box(MULTI_LINE_MARKDOWN);
            let lines: Vec<&str> = markdown.lines().collect();
            let _ = lines.join("\n");
        });
    });

    c.bench_function("contains_check", |b| {
        b.iter(|| {
            let markdown = black_box(MULTI_LINE_MARKDOWN);
            let _ = markdown.contains("```mermaid");
        });
    });

    c.bench_function("split_whitespace", |b| {
        b.iter(|| {
            let markdown = black_box(MULTI_LINE_MARKDOWN);
            let _count = markdown.split_whitespace().count();
        });
    });
}

fn bench_fence_parsing(c: &mut Criterion) {
    const FENCE_LINE: &str = "```mermaid";
    const CLOSE_FENCE: &str = "```";

    c.bench_function("parse_fence_start", |b| {
        b.iter(|| {
            let line = black_box(FENCE_LINE);
            let mut chars = line.chars();
            let _marker = chars.next();
            let mut len = 1usize;
            for ch in chars.by_ref() {
                if ch == '`' {
                    len += 1;
                } else {
                    break;
                }
            }
        });
    });

    c.bench_function("is_fence_close", |b| {
        b.iter(|| {
            let line = black_box(CLOSE_FENCE);
            let mut len = 0usize;
            for ch in line.chars() {
                if ch == '`' {
                    len += 1;
                } else {
                    break;
                }
            }
            let _is_close = len >= 3 && line[len..].trim().is_empty();
        });
    });

    c.bench_function("is_mermaid_lang", |b| {
        b.iter(|| {
            let lang = black_box("mermaid");
            let first = lang.split_whitespace().next().unwrap_or("");
            let _ = first.eq_ignore_ascii_case("mermaid");
        });
    });

    c.bench_function("eq_ignore_ascii_case", |b| {
        b.iter(|| {
            let line = black_box("```mermaid");
            let _ = line.eq_ignore_ascii_case("```mermaid");
        });
    });
}

fn bench_error_context_parsing(c: &mut Criterion) {
    const ERROR_MESSAGE: &str = "mermaid error: Parse error on line 2:\ngraph m  A[main.rs] --> B[ser\n----------^\nExpecting 'SEMI', 'NEWLINE', 'EOF', got 'NODE_STRING'";

    c.bench_function("parse_error_context", |b| {
        b.iter(|| {
            let message = black_box(ERROR_MESSAGE);
            let lines: Vec<&str> = message.lines().collect();
            let mut _found_line_number = false;

            for line in &lines {
                let lower = line.to_ascii_lowercase();
                let marker = "parse error on line ";
                if lower.contains(marker) {
                    _found_line_number = true;
                    break;
                }
            }
        });
    });

    c.bench_function("find_reason", |b| {
        b.iter(|| {
            let message = black_box(ERROR_MESSAGE);
            let lines: Vec<&str> = message.lines().collect();
            let mut _found_reason = None;

            for line in &lines {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.contains("Expecting")
                    || trimmed.contains("UnknownDiagramError")
                    || trimmed.contains("got ")
                {
                    _found_reason = Some(trimmed);
                    break;
                }
            }
        });
    });

    c.bench_function("to_ascii_lowercase", |b| {
        b.iter(|| {
            let message = black_box(ERROR_MESSAGE);
            let _ = message.to_ascii_lowercase();
        });
    });
}

fn bench_string_normalization(c: &mut Criterion) {
    const FENCED_DIAGRAM: &str = "```mermaid\ngraph TD\nA-->B\n```";

    c.bench_function("normalize_fenced_diagram", |b| {
        b.iter(|| {
            let input = black_box(FENCED_DIAGRAM);
            let trimmed = input.trim();
            if trimmed.starts_with("```") {
                let mut lines: Vec<&str> = trimmed.lines().collect();
                if lines.len() > 2 {
                    lines.pop();
                    let _ = lines[1..].join("\n");
                }
            }
        });
    });

    c.bench_function("normalize_plain_diagram", |b| {
        b.iter(|| {
            let input = black_box("graph TD\nA-->B");
            let _ = input.trim();
        });
    });

    c.bench_function("is_mermaid_fence", |b| {
        b.iter(|| {
            let line = black_box("```mermaid");
            let _ = line.eq_ignore_ascii_case("```mermaid");
        });
    });

    c.bench_function("strip_prefix", |b| {
        b.iter(|| {
            let input = black_box("```mermaid\ngraph");
            let _ = input.strip_prefix("```mermaid");
        });
    });
}

fn bench_memory_allocation(c: &mut Criterion) {
    c.bench_function("vec_push_100", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..100 {
                vec.push(i);
            }
        });
    });

    c.bench_function("vec_push_1000", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(i);
            }
        });
    });

    c.bench_function("string_concat_100", |b| {
        b.iter(|| {
            let mut s = String::new();
            for i in 0..100 {
                s.push_str(&format!("line {}\n", i));
            }
        });
    });

    c.bench_function("hashmap_insert_100", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for i in 0..100 {
                map.insert(i, format!("value {}", i));
            }
        });
    });

    c.bench_function("string_from_str", |b| {
        b.iter(|| {
            let _ = String::from("test string with some content");
        });
    });

    c.bench_function("string_to_owned", |b| {
        b.iter(|| {
            let s = black_box("test string");
            let _ = s.to_owned();
        });
    });
}

fn bench_iteration_patterns(c: &mut Criterion) {
    const DATA: &[i32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    c.bench_function("for_loop_sum", |b| {
        b.iter(|| {
            let mut sum = 0;
            for &item in black_box(DATA) {
                sum += item;
            }
        });
    });

    c.bench_function("iterator_sum", |b| {
        b.iter(|| {
            let sum: i32 = black_box(DATA).iter().sum();
            sum
        });
    });

    c.bench_function("for_loop_find", |b| {
        b.iter(|| {
            let mut found = None;
            for &item in black_box(DATA) {
                if item == 5 {
                    found = Some(item);
                    break;
                }
            }
        });
    });

    c.bench_function("iterator_find", |b| {
        b.iter(|| {
            let _found = black_box(DATA).iter().find(|&&x| x == 5);
        });
    });
}

fn bench_regex_like_operations(c: &mut Criterion) {
    const TEST_STRING: &str = "Parse error on line 42: unexpected token";

    c.bench_function("starts_with_check", |b| {
        b.iter(|| {
            let s = black_box(TEST_STRING);
            let _ = s.starts_with("Parse error");
        });
    });

    c.bench_function("contains_check", |b| {
        b.iter(|| {
            let s = black_box(TEST_STRING);
            let _ = s.contains("line");
        });
    });

    c.bench_function("find_pattern", |b| {
        b.iter(|| {
            let s = black_box(TEST_STRING);
            let _ = s.find("line");
        });
    });

    c.bench_function("split_once", |b| {
        b.iter(|| {
            let s = black_box("line 42");
            let _ = s.split_once(' ').map(|(a, b)| (a, b));
        });
    });
}

fn bench_json_like_parsing(c: &mut Criterion) {
    const JSON_DATA: &str = r#"{"name": "test", "value": 123, "items": ["a", "b", "c"]}"#;

    c.bench_function("string_parsing_lines", |b| {
        b.iter(|| {
            let data = black_box(JSON_DATA);
            let _lines: Vec<&str> = data.lines().collect();
        });
    });

    c.bench_function("string_parsing_split", |b| {
        b.iter(|| {
            let data = black_box(JSON_DATA);
            let _parts: Vec<&str> = data.split(',').collect();
        });
    });

    c.bench_function("string_parsing_chars", |b| {
        b.iter(|| {
            let data = black_box(JSON_DATA);
            let _count = data.chars().count();
        });
    });
}

fn bench_real_world_paths(c: &mut Criterion) {
    const MARKDOWN_SMALL: &str = r#"
# Title

```mermaid
graph TD
A-->B
```

Text.
"#;

    const MARKDOWN_LARGE: &str = r#"
# Section 1
```mermaid
graph TD
A1-->B1
```
Some text here.

# Section 2
```mermaid
graph TD
A2-->B2
```
Some text here.

# Section 3
```mermaid
graph TD
A3-->B3
```
Some text here.

# Section 4
```mermaid
graph TD
A4-->B4
```
Some text here.

# Section 5
```mermaid
graph TD
A5-->B5
```
Some text here.

# Section 6
```mermaid
graph TD
A6-->B6
```
Some text here.
"#;

    const PARSE_ERROR: &str = "mermaid-cli process exited with code 1\n\nError details:\nmermaid error: Parse error on line 2:\ngraph m  A[main.rs] --> B[ser\n----------^\nExpecting 'SEMI', 'NEWLINE', 'EOF', got 'NODE_STRING'";
    const UNKNOWN_DIAGRAM: &str =
        "UnknownDiagramError: No diagram type detected matching given configuration";

    c.bench_function("scan_markdown_small", |b| {
        b.iter(|| {
            let markdown = black_box(MARKDOWN_SMALL);
            let _ = scan_markdown_for_mermaid(markdown);
        });
    });

    c.bench_function("scan_markdown_large", |b| {
        b.iter(|| {
            let markdown = black_box(MARKDOWN_LARGE);
            let _ = scan_markdown_for_mermaid(markdown);
        });
    });

    c.bench_function("invalid_result_parse_error", |b| {
        b.iter(|| {
            let err = black_box(PARSE_ERROR);
            let _ = invalid_result(err);
        });
    });

    c.bench_function("invalid_result_unknown_diagram", |b| {
        b.iter(|| {
            let err = black_box(UNKNOWN_DIAGRAM);
            let _ = invalid_result(err);
        });
    });
}

criterion_group!(
    benches,
    bench_string_parsing,
    bench_fence_parsing,
    bench_error_context_parsing,
    bench_string_normalization,
    bench_memory_allocation,
    bench_iteration_patterns,
    bench_regex_like_operations,
    bench_json_like_parsing,
    bench_real_world_paths
);
criterion_main!(benches);
