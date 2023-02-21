use async_std::fs;
use costaeres::common::*;
use costaeres::file_store::*;

fn named_variant(name: &str) -> VariantMetadata {
    VariantMetadata::new(name, "application/octet-stream", 42)
}

fn default_variant() -> VariantMetadata {
    named_variant("default")
}

async fn named_content(name: &str) -> Variant {
    let file = fs::File::open("./create_db.sh").await.unwrap();
    Variant::new(named_variant(name), Box::new(file))
}

async fn default_content() -> Variant {
    named_content("default").await
}

#[async_std::test]
async fn file_store() {
    let _ = fs::remove_dir_all("./test-content/0").await;
    let _ = fs::create_dir_all("./test-content/0").await;

    let store = FileStore::new(
        "./test-content/0",
        Box::new(DefaultResourceNameProvider),
        Box::new(IdentityTransformer),
    )
    .await
    .unwrap();

    // Starting with no content.
    let res = store.get_full(&ROOT_ID, "default").await.err();
    assert_eq!(res, Some(ResourceStoreError::NoSuchResource));

    // Adding an object.
    let meta = ResourceMetadata::new(
        &ROOT_ID,
        &ROOT_ID,
        ResourceKind::Leaf,
        "object 0",
        vec!["one".into(), "two".into()],
        vec![default_variant()],
    );

    let res = store
        .create(&meta, Some(default_content().await))
        .await
        .ok();
    assert_eq!(res, Some(()));

    // Now check that we can get it.
    let res = store.get_full(&ROOT_ID, "default").await.ok().unwrap().0;
    assert!(res.id().is_root());
    assert_eq!(*res.tags(), vec!["one".to_owned(), "two".to_owned()]);
    assert_eq!(&res.name(), "object 0");

    // Check we can't add another object with the same id.
    let res = store
        .create(&meta, Some(default_content().await))
        .await
        .err();
    assert_eq!(res, Some(ResourceStoreError::ResourceAlreadyExists));

    // Update the object.
    let mut meta = ResourceMetadata::new(
        &ROOT_ID,
        &ROOT_ID,
        ResourceKind::Leaf,
        "object 0 updated",
        vec!["one".into(), "two".into()],
        vec![default_variant()],
    );

    store
        .update(&meta, Some(default_content().await))
        .await
        .unwrap();

    let res = store.get_full(&ROOT_ID, "default").await.ok().unwrap().0;
    assert!(res.id().is_root());
    assert_eq!(&res.name(), "object 0 updated");

    // Get the default variant.
    store.get_variant(&ROOT_ID, "default").await.unwrap();

    // Check that we don't have another variant.
    assert!(store.get_variant(&ROOT_ID, "not-default").await.is_err());

    // Add a variant.
    meta.add_variant(named_variant("new-variant"));
    store
        .update(&meta, Some(named_content("new-variant").await))
        .await
        .unwrap();
    // Get the new variant.
    store.get_variant(&ROOT_ID, "new-variant").await.unwrap();

    // Update with an invalid variant.
    let res = store
        .update(&meta, Some(named_content("invalid-variant").await))
        .await;
    assert_eq!(
        res,
        Err(ResourceStoreError::InvalidVariant("invalid-variant".into()))
    );

    // Now delete this object.
    store.delete(&ROOT_ID).await.ok().unwrap();

    // And check we can't get it anymore.
    let res = store.get_full(&ROOT_ID, "default").await.err();
    assert_eq!(res, Some(ResourceStoreError::NoSuchResource));
}
