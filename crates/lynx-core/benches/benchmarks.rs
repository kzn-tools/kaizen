use std::hint::black_box;
use std::time::Instant;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use lynx_core::analysis::AnalysisEngine;
use lynx_core::parser::ParsedFile;

const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures");

fn generate_500_loc_typescript() -> String {
    let mut code = String::with_capacity(20000);
    code.push_str("// Generated 500 LOC TypeScript file for benchmarking\n\n");

    for i in 0..25 {
        code.push_str(&format!(
            r#"interface Entity{i} {{
    id: number;
    name: string;
    createdAt: Date;
    updatedAt: Date;
    metadata?: Record<string, unknown>;
}}

function processEntity{i}(entity: Entity{i}): Entity{i} {{
    const result = {{
        ...entity,
        updatedAt: new Date(),
    }};
    if (entity.metadata) {{
        result.metadata = {{ ...entity.metadata, processed: true }};
    }}
    return result;
}}

async function fetchEntity{i}(id: number): Promise<Entity{i} | null> {{
    const response = await fetch(`/api/entities/{i}/${{id}}`);
    if (!response.ok) {{
        return null;
    }}
    return response.json();
}}

"#,
            i = i
        ));
    }

    code
}

fn generate_100_files() -> Vec<(String, String)> {
    (0..100)
        .map(|i| {
            let filename = format!("file_{}.ts", i);
            let content = format!(
                r#"interface Item{i} {{
    id: number;
    value: string;
}}

function process{i}(item: Item{i}): Item{i} {{
    return {{ ...item, value: item.value.toUpperCase() }};
}}

export {{ Item{i}, process{i} }};
"#,
                i = i
            );
            (filename, content)
        })
        .collect()
}

fn read_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{}", FIXTURES_DIR, path))
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path))
}

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");

    let code_500 = generate_500_loc_typescript();
    let lines_500 = code_500.lines().count();

    group.throughput(Throughput::Elements(lines_500 as u64));
    group.bench_function("parse_500_loc", |b| {
        b.iter(|| ParsedFile::from_source(black_box("benchmark.ts"), black_box(&code_500)))
    });

    let tsx_code = read_fixture("typescript/tsx-component.tsx");
    let tsx_lines = tsx_code.lines().count();

    group.throughput(Throughput::Elements(tsx_lines as u64));
    group.bench_function("parse_tsx_component", |b| {
        b.iter(|| ParsedFile::from_source(black_box("component.tsx"), black_box(&tsx_code)))
    });

    let interfaces_code = read_fixture("typescript/interfaces.ts");
    let interfaces_lines = interfaces_code.lines().count();

    group.throughput(Throughput::Elements(interfaces_lines as u64));
    group.bench_function("parse_typescript_interfaces", |b| {
        b.iter(|| ParsedFile::from_source(black_box("interfaces.ts"), black_box(&interfaces_code)))
    });

    group.finish();
}

fn bench_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("rules");

    let engine = AnalysisEngine::new();

    let quality_code = r#"
var x = 1;
var y = 2;
if (x == y) {
    console.log("equal");
}
const unused = 42;
let reassignable = "hello";
reassignable = "world";
eval("console.log('dangerous')");
"#;

    let quality_file = ParsedFile::from_source("quality.ts", quality_code);
    group.bench_function("quality_rules", |b| {
        b.iter(|| engine.analyze(black_box(&quality_file)))
    });

    let security_code = r#"
import { query } from 'database';

function getUserById(id: string) {
    return query("SELECT * FROM users WHERE id = " + id);
}

function setHtml(elem: Element, content: string) {
    elem.innerHTML = content;
}

const secret = "sk_live_abc123xyz789";
const password = "password123";
"#;

    let security_file = ParsedFile::from_source("security.ts", security_code);
    group.bench_function("security_rules", |b| {
        b.iter(|| engine.analyze(black_box(&security_file)))
    });

    let clean_code = r#"
const PI = 3.14159;

function calculateArea(radius: number): number {
    return PI * radius * radius;
}

function formatResult(value: number, decimals = 2): string {
    return value.toFixed(decimals);
}

export { calculateArea, formatResult };
"#;

    let clean_file = ParsedFile::from_source("clean.ts", clean_code);
    group.bench_function("clean_code", |b| {
        b.iter(|| engine.analyze(black_box(&clean_file)))
    });

    group.finish();
}

fn bench_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis");

    let engine = AnalysisEngine::new();
    let code_500 = generate_500_loc_typescript();
    let file_500 = ParsedFile::from_source("large.ts", &code_500);

    group.bench_function("analyze_500_loc", |b| {
        b.iter(|| engine.analyze(black_box(&file_500)))
    });

    let files_100 = generate_100_files();
    let parsed_files: Vec<ParsedFile> = files_100
        .iter()
        .map(|(name, content)| ParsedFile::from_source(name, content))
        .collect();

    group.bench_function("analyze_100_files", |b| {
        b.iter(|| {
            for file in &parsed_files {
                let _ = engine.analyze(black_box(file));
            }
        })
    });

    for size in [10, 25, 50, 100] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("project_size", size), &size, |b, &size| {
            let subset: Vec<_> = parsed_files.iter().take(size).collect();
            b.iter(|| {
                for file in &subset {
                    let _ = engine.analyze(black_box(file));
                }
            })
        });
    }

    group.finish();
}

fn bench_latency_percentiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency");

    let engine = AnalysisEngine::new();
    let code_500 = generate_500_loc_typescript();

    group.bench_function("p95_500_loc_parse_analyze", |b| {
        b.iter_custom(|iters| {
            let mut durations: Vec<_> = (0..iters)
                .map(|_| {
                    let start = Instant::now();
                    let file =
                        ParsedFile::from_source(black_box("benchmark.ts"), black_box(&code_500));
                    let _ = engine.analyze(black_box(&file));
                    start.elapsed()
                })
                .collect();
            durations.sort();
            let p95_idx = ((iters as f64) * 0.95) as usize;
            let p95_idx = p95_idx.min(durations.len().saturating_sub(1));
            durations[p95_idx]
        })
    });

    let files_100 = generate_100_files();
    group.bench_function("p95_per_file_100_files", |b| {
        b.iter_custom(|iters| {
            let mut all_durations = Vec::with_capacity((iters as usize) * 100);
            for _ in 0..iters {
                for (name, content) in &files_100 {
                    let start = Instant::now();
                    let file = ParsedFile::from_source(black_box(name), black_box(content));
                    let _ = engine.analyze(black_box(&file));
                    all_durations.push(start.elapsed());
                }
            }
            all_durations.sort();
            let p95_idx = ((all_durations.len() as f64) * 0.95) as usize;
            let p95_idx = p95_idx.min(all_durations.len().saturating_sub(1));
            all_durations[p95_idx]
        })
    });

    group.finish();
}

fn bench_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    group.sample_size(10);

    let files_100 = generate_100_files();
    let parsed_files: Vec<ParsedFile> = files_100
        .iter()
        .map(|(name, content)| ParsedFile::from_source(name, content))
        .collect();

    group.bench_function("100_files_retained", |b| {
        b.iter(|| {
            let retained: Vec<_> = parsed_files.iter().map(|f| f.source().len()).collect();
            black_box(retained)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing,
    bench_rules,
    bench_analysis,
    bench_latency_percentiles,
    bench_memory
);
criterion_main!(benches);
