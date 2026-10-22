use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
    measurement::WallTime,
};
use edb2_avl_rbtree::tree::{avl::AvlTree, ordered::OrderedTree, rbtree::RBTree};
use std::hint::black_box;

const SIZES: [usize; 3] = [100, 1_000, 10_000];

fn keys(size: usize, order: &str) -> Vec<usize> {
    let mut keys: Vec<_> = (0..size).collect();
    match order {
        "descending" => keys.reverse(),
        "alternating" => {
            keys = (0..size)
                .step_by(2)
                .chain((1..size).step_by(2).rev())
                .collect();
        }
        _ => {}
    }
    keys
}

fn insert_benchmarks<T: OrderedTree<usize, usize>>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    tree_name: &str,
) {
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        for order in ["ascending", "alternating", "descending"] {
            let input = keys(size, order);
            group.bench_with_input(
                BenchmarkId::new(format!("{tree_name}/{order}"), size),
                &input,
                |b, input| {
                    b.iter(|| {
                        let mut tree = T::new();
                        for &key in input {
                            tree.insert(black_box(key), key);
                        }
                        black_box(tree);
                    });
                },
            );
        }
    }
}

fn remove_benchmarks<T: OrderedTree<usize, usize>>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    tree_name: &str,
) {
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        for order in ["ascending", "alternating", "descending"] {
            let insert_order = keys(size, order);
            let remove_order = keys(size, order);
            group.bench_with_input(
                BenchmarkId::new(format!("{tree_name}/{order}"), size),
                &insert_order,
                |b, input| {
                    b.iter_batched(
                        || {
                            let mut tree = T::new();
                            for &key in input {
                                tree.insert(key, key);
                            }
                            tree
                        },
                        |mut tree| {
                            for key in &remove_order {
                                black_box(tree.remove(black_box(key)));
                            }
                        },
                        BatchSize::LargeInput,
                    );
                },
            );
        }
    }
}

fn search_benchmark<T: OrderedTree<usize, usize>>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    tree_name: &str,
    size: usize,
    order: &str,
    query_kind: &str,
) {
    let insert_order = keys(size, order);
    let mut tree = T::new();
    for &key in &insert_order {
        tree.insert(key, key);
    }
    let queries = match query_kind {
        "hits" => insert_order,
        "misses" => (size..size * 2).collect(),
        _ => unreachable!("search query kind is either hits or misses"),
    };

    group.throughput(Throughput::Elements(queries.len() as u64));
    group.bench_with_input(
        BenchmarkId::new(format!("{tree_name}/{query_kind}"), size),
        &queries,
        |b, queries| {
            b.iter(|| {
                for key in queries {
                    black_box(tree.search(black_box(key)));
                }
            });
        },
    );
}

fn search_benchmarks(c: &mut Criterion) {
    for order in ["ascending", "descending", "alternating"] {
        let mut group = c.benchmark_group(format!("search/{order}"));
        for size in SIZES {
            for query_kind in ["hits", "misses"] {
                search_benchmark::<AvlTree<usize, usize>>(
                    &mut group, "avl", size, order, query_kind,
                );
                search_benchmark::<RBTree<usize, usize>>(
                    &mut group, "rbtree", size, order, query_kind,
                );
            }
        }
        group.finish();
    }
}

fn benchmarks(c: &mut Criterion) {
    let mut insert_group = c.benchmark_group("insert");
    insert_benchmarks::<AvlTree<usize, usize>>(&mut insert_group, "avl");
    insert_benchmarks::<RBTree<usize, usize>>(&mut insert_group, "rbtree");
    insert_group.finish();

    let mut remove_group = c.benchmark_group("remove");
    remove_benchmarks::<AvlTree<usize, usize>>(&mut remove_group, "avl");
    remove_benchmarks::<RBTree<usize, usize>>(&mut remove_group, "rbtree");
    remove_group.finish();

    search_benchmarks(c);
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
