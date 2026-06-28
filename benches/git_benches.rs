use divan::{Bencher, black_box};
use gitkit_cli::git::{kit::KitRepo, metrics::silo::SiloData};

#[divan::bench]
fn bench_get_all_commits() {
    let repo = KitRepo::open(black_box("../ghexample")).unwrap();
    let _commits = black_box(repo.get_all_commits().unwrap());
}

#[divan::bench(sample_count = 5, sample_size = 1)]
fn bench_accumulate_churn(bencher: divan::Bencher) {
    let repo = KitRepo::open("../ghexample").unwrap();

    bencher.bench_local(|| {
        let churn_map = SiloData::accumulate_churn(black_box(&repo)).unwrap();
        black_box(churn_map);
    });
}

fn main() {
    divan::main();
}
