// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Reproducible public-API layout cost baseline.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use binlayout::{Layout, StructLayout, TargetLayout, Type, UnionLayout};

const SAMPLES: usize = 5;
const SAMPLE_TARGET: Duration = Duration::from_millis(12);
const MAX_ITERATIONS: u32 = 1 << 22;
const FIELD_WIDTH: u64 = 8;

fn elapsed(iterations: u32, operation: &mut impl FnMut()) -> Duration {
    let start = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    start.elapsed()
}

fn measure(operation: &str, workload: &str, mut run: impl FnMut()) {
    let mut iterations = 1;
    while elapsed(iterations, &mut run) < SAMPLE_TARGET && iterations < MAX_ITERATIONS {
        iterations *= 2;
    }
    let mut times = [Duration::ZERO; SAMPLES];
    for time in &mut times {
        *time = elapsed(iterations, &mut run);
    }
    times.sort_unstable();
    let ns = |time: Duration| time.as_secs_f64() * 1e9 / f64::from(iterations);
    println!(
        "{operation},{workload},{iterations},{:.2},{:.2},{:.2}",
        ns(times[0]),
        ns(times[SAMPLES / 2]),
        ns(times[SAMPLES - 1]),
    );
}

fn field_name(index: usize, length: usize) -> String {
    format!("{}{index:04}", "x".repeat(length - 4))
}

fn build_structure(names: &[String], field: Layout) -> StructLayout {
    let mut builder = StructLayout::builder();
    for name in names {
        builder = builder.field(name.as_str(), field);
    }
    builder.build().expect("valid structure fixture")
}

fn build_union(names: &[String], field: Layout) -> UnionLayout {
    let mut builder = UnionLayout::builder();
    for name in names {
        builder = builder.field(name.as_str(), field);
    }
    builder.build().expect("valid union fixture")
}

fn named_layouts(count: usize, length: usize) {
    let names: Vec<_> = (0..count).map(|index| field_name(index, length)).collect();
    let field = Layout::new(FIELD_WIDTH, FIELD_WIDTH).expect("valid field fixture");
    let structure = build_structure(&names, field);
    let union = build_union(&names, field);
    assert_eq!(structure.fields().len(), count);
    assert_eq!(union.fields().len(), count);
    assert_eq!(structure.size(), count as u64 * FIELD_WIDTH);
    assert_eq!(union.size(), FIELD_WIDTH);

    let workload = format!("{count}fields_{length}bytes");
    measure("struct_build", &workload, || {
        black_box(build_structure(black_box(&names), black_box(field)));
    });
    measure("union_build", &workload, || {
        black_box(build_union(black_box(&names), black_box(field)));
    });

    let missing = field_name(count, length);
    for (position, name, index) in [
        ("first", names[0].as_str(), Some(0)),
        ("middle", names[count / 2].as_str(), Some(count / 2)),
        ("last", names[count - 1].as_str(), Some(count - 1)),
        ("missing", missing.as_str(), None),
    ] {
        assert_eq!(
            structure.offset(name),
            index.map(|value| value as u64 * FIELD_WIDTH)
        );
        assert_eq!(union.offset(name), index.map(|_| 0));
        assert_eq!(
            union.field(name).map(|field| field.name()),
            index.map(|_| name)
        );
        let workload = format!("{workload}_{position}");
        measure("struct_lookup", &workload, || {
            black_box(black_box(&structure).field(black_box(name)));
        });
        measure("union_lookup", &workload, || {
            black_box(black_box(&union).field(black_box(name)));
        });
    }
}

fn target_calculations() {
    let target = TargetLayout::x86_64();
    for depth in [0, 1, 4, 12, 32] {
        let mut ty = Type::U64;
        for _ in 0..depth {
            ty = Type::Struct(vec![Type::U8, ty]);
        }
        let expected = Layout::new((depth + 1) * FIELD_WIDTH, FIELD_WIDTH).unwrap();
        assert_eq!(target.layout_of(&ty), Ok(expected));
        measure("target_nested", &format!("depth_{depth}"), || {
            black_box(black_box(&target).layout_of(black_box(&ty)).unwrap());
        });
    }

    for count in [1, 1024, 1_048_576] {
        let record = Type::Struct(vec![Type::U8, Type::U64, Type::U16]);
        let ty = Type::array(record, count);
        let expected = Layout::new(24 * count, FIELD_WIDTH).unwrap();
        assert_eq!(target.layout_of(&ty), Ok(expected));
        measure("target_array", &format!("count_{count}"), || {
            black_box(black_box(&target).layout_of(black_box(&ty)).unwrap());
        });
    }
}

fn main() {
    println!("operation,workload,iterations,min_ns,median_ns,max_ns");
    for (count, length) in [(4, 8), (16, 8), (64, 8), (256, 8), (64, 32), (64, 128)] {
        named_layouts(count, length);
    }
    target_calculations();
}
