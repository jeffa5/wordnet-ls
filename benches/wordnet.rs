use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use wordnet_ls::wordnet::SemanticRelation;
use wordnet_ls::wordnet::WordNet;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("all_words", |b| {
        b.iter(|| {
            let wndir = std::env::var("WNSEARCHDIR").unwrap();
            let wn = WordNet::new(&PathBuf::from(wndir)).unwrap();
            let len = wn.all_words().len();
            black_box(len)
        })
    });
    c.bench_function("all_words_cause", |b| {
        b.iter(|| {
            let wndir = std::env::var("WNSEARCHDIR").unwrap();
            let wn = WordNet::new(&PathBuf::from(wndir)).unwrap();
            let words = wn.all_words().into_iter().map(move |w| {
                let synsets = wn.synsets(&w);
                synsets
                    .iter()
                    .map(|synsets| {
                        synsets
                            .iter()
                            .map(|ss| ss.with_relationship(SemanticRelation::Cause).len())
                            .sum::<usize>()
                    })
                    .sum::<usize>()
            });
            black_box(words)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
