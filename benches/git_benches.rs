use divan::black_box;
use gitkit_cli::git::{
    kit::KitRepo,
    metrics::{cadence::CadenceData, home::HomeData, silo::SiloData},
};

const REPO: &str = "../ghexample";

#[divan::bench]
fn bench_get_all_commits() {
    let repo = KitRepo::open(black_box(REPO)).unwrap();
    let _commits = black_box(repo.get_all_commits().unwrap());
}

#[divan::bench(sample_count = 5, sample_size = 1)]
fn bench_accumulate_churn(bencher: divan::Bencher) {
    let repo = KitRepo::open(REPO).unwrap();

    bencher.bench_local(|| {
        let churn_map = SiloData::accumulate_churn(black_box(&repo)).unwrap();
        black_box(churn_map);
    });
}

#[divan::bench(sample_count = 20)]
fn bench_home_data(bencher: divan::Bencher) {
    let repo = KitRepo::open(REPO).unwrap();
    bencher.bench_local(|| black_box(HomeData::new(black_box(&repo))));
}

#[divan::bench(sample_count = 20)]
fn bench_cadence_data(bencher: divan::Bencher) {
    let repo = KitRepo::open(REPO).unwrap();
    bencher.bench_local(|| black_box(CadenceData::new(black_box(&repo))));
}

#[divan::bench(sample_count = 5, sample_size = 1)]
fn bench_silo_data(bencher: divan::Bencher) {
    let repo = KitRepo::open(REPO).unwrap();
    bencher.bench_local(|| black_box(SiloData::new(black_box(&repo))));
}

fn main() {
    divan::main();
}
