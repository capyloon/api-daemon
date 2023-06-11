use criterion::async_executor::AsyncStdExecutor;
use criterion::*;

use costaeres::common::*;
use costaeres::config::Config;
use costaeres::file_store::FileStore;
use costaeres::manager::Manager;

async fn prepare_bench() -> (Config, FileStore) {
    let _ = env_logger::try_init();

    let path = format!("./bench-fixtures/");

    let store = FileStore::new(
        &path,
        Box::new(DefaultResourceNameProvider),
        Box::new(IdentityTransformer),
    )
    .await
    .unwrap();

    let config = Config {
        db_path: format!("{}/manager.sqlite", &path),
        data_dir: ".".into(),
        metadata_cache_capacity: 100,
    };

    (config, store)
}

async fn search(manager: &Manager<()>, input: &str, tag: Option<String>) {
    let _results = manager.by_text(input, tag).await.unwrap();
}

fn places_search(c: &mut Criterion) {
    c.bench_function("places search", move |b| {
        let manager = async_std::task::block_on(async {
            let (config, store) = prepare_bench().await;
            Manager::<()>::new(config, Box::new(store)).await.unwrap()
        });

        b.to_async(AsyncStdExecutor)
            .iter(|| async { search(&manager, "wiki", Some("places".into())).await })
    });
}

fn no_tag_search(c: &mut Criterion) {
    c.bench_function("no tag search", move |b| {
        let manager = async_std::task::block_on(async {
            let (config, store) = prepare_bench().await;
            Manager::<()>::new(config, Box::new(store)).await.unwrap()
        });

        b.to_async(AsyncStdExecutor)
            .iter(|| async { search(&manager, "wiki", None).await })
    });
}

criterion_group!(benches, places_search);
criterion_group!(benches2, no_tag_search);
criterion_main!(benches, benches2);
