use async_std::fs;
use chrono::Utc;
use costaeres::common::*;
use costaeres::config::Config;
use costaeres::file_store::FileStore;
use costaeres::indexer::*;
use costaeres::manager::*;
use costaeres::scorer::{VisitEntry, VisitPriority};
use std::rc::Rc;

fn named_variant(name: &str, mime_type: &str) -> VariantMetadata {
    VariantMetadata::new(name, mime_type, 42)
}

fn default_variant() -> VariantMetadata {
    named_variant("default", "application/octet-stream")
}

async fn named_content(name: &str) -> Variant {
    let file = fs::File::open("./create_db.sh").await.unwrap();
    Variant::new(
        named_variant(name, "application/octet-stream"),
        Box::new(file),
    )
}

async fn default_content() -> Variant {
    named_content("default").await
}

// Prepare a test directory, and returns the matching config and file store.
async fn prepare_test(index: u32) -> (Config, FileStore) {
    let _ = env_logger::try_init();

    let path = format!("./test-content/{index}");

    let _ = fs::remove_dir_all(&path).await;
    let _ = fs::create_dir_all(&path).await;

    let store = FileStore::new(
        &path,
        Box::new(DefaultResourceNameProvider),
        Box::new(IdentityTransformer),
    )
    .await
    .unwrap();

    let config = Config {
        db_path: format!("{}/test_db.sqlite", &path),
        data_dir: ".".into(),
        metadata_cache_capacity: 100,
    };

    (config, store)
}

async fn create_hierarchy<T>(manager: &mut Manager<T>) {
    // Adding the root to the file store.
    manager.create_root().await.unwrap();

    // Add a sub-container.
    let mut container = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Container,
        "container",
        vec![],
        vec![],
    );
    manager
        .create(&mut container, Some(default_content().await))
        .await
        .unwrap();

    // Add a few children to the container.
    for i in 5..15 {
        let mut child = ResourceMetadata::new(
            &i.into(),
            &1.into(),
            if i == 10 {
                ResourceKind::Container
            } else {
                ResourceKind::Leaf
            },
            &format!("child #{i}"),
            vec![],
            vec![default_variant()],
        );
        manager
            .create(&mut child, Some(default_content().await))
            .await
            .unwrap();
    }

    // Add a few children to the sub-container #10.
    for i in 25..35 {
        let mut child = ResourceMetadata::new(
            &i.into(),
            &10.into(),
            ResourceKind::Leaf,
            &format!("child #{i}"),
            vec!["sub-child".into()],
            vec![default_variant()],
        );
        manager
            .create(&mut child, Some(default_content().await))
            .await
            .unwrap();
    }
}

#[async_std::test]
async fn basic_manager() {
    let (config, store) = prepare_test(1).await;

    let manager = Manager::<()>::new(config, Box::new(store)).await;
    assert!(manager.is_ok(), "Failed to create a manager");
    let mut manager = manager.unwrap();

    // Adding an object.
    let mut meta = ResourceMetadata::new(
        &ROOT_ID,
        &ROOT_ID,
        ResourceKind::Leaf,
        "object 0",
        vec!["one".into(), "two".into()],
        vec![default_variant()],
    );

    manager
        .create(&mut meta, Some(default_content().await))
        .await
        .unwrap();
    // assert_eq!(res, Ok(()));

    let res = manager.get_metadata(&meta.id()).await.unwrap();
    assert_eq!(res, meta);

    // Delete a non-existent object.
    let res = manager.delete(&42.into()).await;
    assert!(res.is_err());

    // Update the root object.
    let res = manager
        .update_variant(&ROOT_ID, default_content().await)
        .await;
    assert_eq!(res, Ok(()));

    // Delete the root object
    let res = manager.delete(&ROOT_ID).await;
    assert!(res.is_ok());

    // Expected failure
    let res = manager.get_metadata(&meta.id()).await;
    assert!(res.is_err());
}

#[async_std::test]
async fn rehydrate_single() {
    let (config, store) = prepare_test(2).await;

    // Adding an object to the file store
    let meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "object 0",
        vec!["one".into(), "two".into()],
        vec![default_variant()],
    );
    store
        .create(&meta, Some(default_content().await))
        .await
        .unwrap();

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    assert!(!manager.has_object(&meta.id()).await.unwrap());

    let res = manager.get_metadata(&meta.id()).await.unwrap();
    assert_eq!(res, meta);

    assert!(manager.has_object(&meta.id()).await.unwrap());
}

#[async_std::test]
async fn check_constraints() {
    let (config, store) = prepare_test(3).await;

    let mut meta = ResourceMetadata::new(
        &1.into(),
        &1.into(),
        ResourceKind::Leaf,
        "object 0",
        vec![],
        vec![],
    );

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    // Fail to store an object where both id and parent are 1
    let res = manager
        .create(&mut meta, Some(default_content().await))
        .await;
    assert_eq!(res, Err(ResourceStoreError::InvalidContainerId));

    // Fail to store an object if the parent doesn't exist.
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "leaf 1",
        vec![],
        vec![default_variant()],
    );
    let res = manager
        .create(&mut leaf_meta, Some(default_content().await))
        .await;
    assert_eq!(res, Err(ResourceStoreError::InvalidContainerId));

    // Create the root
    let mut root_meta = ResourceMetadata::new(
        &ROOT_ID,
        &ROOT_ID,
        ResourceKind::Container,
        "root",
        vec![],
        vec![default_variant()],
    );
    manager
        .create(&mut root_meta, Some(default_content().await))
        .await
        .unwrap();

    // And now add the leaf.
    manager
        .create(&mut leaf_meta, Some(default_content().await))
        .await
        .unwrap();

    // Try to update the leaf to a non-existent parent.
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &2.into(),
        ResourceKind::Leaf,
        "leaf 1",
        vec![],
        vec![default_variant()],
    );
    let res = manager
        .create(&mut leaf_meta, Some(default_content().await))
        .await;
    assert_eq!(res, Err(ResourceStoreError::InvalidContainerId));
}

#[async_std::test]
async fn delete_hierarchy() {
    let (config, store) = prepare_test(4).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    // Delete a single child.
    manager.delete(&12.into()).await.unwrap();
    // Child 12 disappears
    assert!(!manager.has_object(&12.into()).await.unwrap());

    // Child 10 exists now.
    assert!(manager.has_object(&10.into()).await.unwrap());

    // Delete the container.
    manager.delete(&1.into()).await.unwrap();
    // Child 10 disappears, but not the root.
    assert!(!manager.has_object(&10.into()).await.unwrap());
    assert!(manager.has_object(&ROOT_ID).await.unwrap());
}

#[async_std::test]
async fn rehydrate_full() {
    let (config, store) = prepare_test(5).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    assert_eq!(manager.resource_count().await.unwrap(), 22);

    // Clear the local index.
    manager.clear().await.unwrap();
    assert_eq!(manager.resource_count().await.unwrap(), 0);

    let (root_meta, children) = manager.get_root().await.unwrap();
    assert!(root_meta.id().is_root());
    assert_eq!(children.len(), 1);

    let (sub_meta, children) = manager.get_container(&children[0].id()).await.unwrap();
    assert_eq!(sub_meta.id(), 1.into());
    assert_eq!(children.len(), 10);
}

#[async_std::test]
async fn get_full_path() {
    let (config, store) = prepare_test(6).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let root_path = manager.get_full_path(&ROOT_ID).await.unwrap();
    assert_eq!(root_path.len(), 1);
    assert!(root_path[0].id().is_root());

    let obj_path = manager.get_full_path(&30.into()).await.unwrap();
    assert_eq!(obj_path.len(), 4);
    assert!(obj_path[0].id().is_root());
    assert_eq!(obj_path[1].id(), 1.into());
    assert_eq!(obj_path[2].id(), 10.into());
    assert_eq!(obj_path[3].id(), 30.into());
}

#[async_std::test]
async fn search_by_name() {
    let (config, store) = prepare_test(7).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let results = manager.by_name("unknown", Some("image/png")).await.unwrap();
    assert_eq!(results.len(), 0);

    let results = manager.by_name("child #12", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], 12.into());

    let results = manager.by_name("child #12", None).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], 12.into());

    let results = manager
        .by_name("child #12", Some("image/png"))
        .await
        .unwrap();
    assert_eq!(results.len(), 0);
}

#[async_std::test]
async fn search_by_tag() {
    let (config, store) = prepare_test(8).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let results = manager.by_tag("no-such-tag").await.unwrap();
    assert_eq!(results.len(), 0);

    let results = manager.by_tag("sub-child").await.unwrap();
    assert_eq!(results.len(), 10);
    assert_eq!(results[0], 25.into());
}

#[async_std::test]
async fn search_by_text() {
    let (config, store) = prepare_test(9).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let results = manager.by_text("no-match", None).await.unwrap();
    assert_eq!(results.len(), 0);

    let results = manager.by_text("cont", None).await.unwrap();
    assert_eq!(results.len(), 1);

    let results = manager.by_text("child", None).await.unwrap();
    assert_eq!(results.len(), 20);

    let results = manager.by_text("child #27", None).await.unwrap();
    assert_eq!(results.len(), 1);

    let results = manager.by_text("child #17", None).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[async_std::test]
async fn score() {
    let (config, store) = prepare_test(10).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();
    manager.create_root().await.unwrap();

    let root_meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(root_meta.scorer().frecency(), 0);

    // Update the score
    manager
        .visit(
            &ROOT_ID,
            &VisitEntry::new(&Utc::now(), VisitPriority::Normal),
        )
        .await
        .unwrap();
    let root_meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    let initial_score = root_meta.scorer().frecency();
    assert_eq!(initial_score, 100);

    // Clear the database to force re-hydration.
    manager.clear().await.unwrap();

    // Load the root again.
    let root_meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(initial_score, root_meta.scorer().frecency());
}

#[async_std::test]
async fn top_frecency() {
    let (config, store) = prepare_test(11).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let root_meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(root_meta.scorer().frecency(), 0);

    // Update the score
    manager
        .visit(
            &ROOT_ID,
            &VisitEntry::new(&Utc::now(), VisitPriority::Normal),
        )
        .await
        .unwrap();
    let root_meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(root_meta.scorer().frecency(), 100);

    let results = manager.top_by_frecency(None, 10).await.unwrap();
    assert_eq!(results.len(), 10);
    assert_eq!(results[0], IdFrec::new(&ROOT_ID, 100));
}

#[async_std::test]
async fn index_places() {
    let (config, store) = prepare_test(12).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();
    manager.add_indexer(Box::new(create_places_indexer()));

    manager.create_root().await.unwrap();
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "ecdf525a-e5d6-11eb-9c9b-d3fd1d0ea335",
        vec!["places".into()],
        vec![],
    );

    let places1 = fs::File::open("./test-fixtures/places-1.json")
        .await
        .unwrap();

    manager
        .create(
            &mut leaf_meta,
            Some(Variant::new(
                named_variant("default", "application/x-places+json"),
                Box::new(places1),
            )),
        )
        .await
        .unwrap();

    // Found in the url.
    let results = manager
        .by_text("example", Some("places".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Found in the title.
    let results = manager.by_text("web", Some("places".into())).await.unwrap();
    assert_eq!(results.len(), 1);

    // Update the object with new content.
    let places2 = fs::File::open("./test-fixtures/places-2.json")
        .await
        .unwrap();
    manager
        .update_variant(
            &leaf_meta.id(),
            Variant::new(
                named_variant("default", "application/x-places+json"),
                Box::new(places2),
            ),
        )
        .await
        .unwrap();

    // Found in the url.
    let results = manager
        .by_text("example", Some("places".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Not found in the title anymore.
    let results = manager.by_text("web", Some("places".into())).await.unwrap();
    assert_eq!(results.len(), 0);

    // Found in the new title.
    let results = manager.by_text("new", Some("places".into())).await.unwrap();
    assert_eq!(results.len(), 1);

    // Delete the object, removing the associated text index.
    manager.delete(&leaf_meta.id()).await.unwrap();

    // Used to be found in the url.
    let results = manager
        .by_text("example", Some("places".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 0);

    // Used to be found in the title.
    let results = manager.by_text("new", Some("places".into())).await.unwrap();
    assert_eq!(results.len(), 0);
}

#[async_std::test]
async fn index_contacts() {
    let (config, store) = prepare_test(13).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();
    manager.add_indexer(Box::new(create_contacts_indexer()));

    manager.create_root().await.unwrap();
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "ecdf525a-e5d6-11eb-9c9b-d3fd1d0ea335",
        vec!["contact".into()],
        vec![default_variant()],
    );

    let contacts = fs::File::open("./test-fixtures/contacts-1.json")
        .await
        .unwrap();

    manager
        .create(
            &mut leaf_meta,
            Some(Variant::new(
                named_variant("default", "application/x-contact+json"),
                Box::new(contacts),
            )),
        )
        .await
        .unwrap();

    // Found in the name.
    let results = manager
        .by_text("jean", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Found in the phone number.
    let results = manager
        .by_text("4567", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Found in the name and email.
    let results = manager
        .by_text("dupont", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Found in the email.
    let results = manager
        .by_text("secret", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Starts with "j"
    let results = manager
        .by_text("^^^^j", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 1);

    // Doesn't start with "a"
    let results = manager
        .by_text("^^^^a", Some("contact".into()))
        .await
        .unwrap();
    assert_eq!(results.len(), 0);
}

#[async_std::test]

async fn get_root_children() {
    let (config, store) = prepare_test(15).await;

    let manager = Manager::<()>::new(config, Box::new(store)).await;
    assert!(manager.is_ok(), "Failed to create a manager");
    let mut manager = manager.unwrap();

    manager.create_root().await.unwrap();

    let (root, children) = manager.get_container(&ROOT_ID).await.unwrap();

    assert!(root.id().is_root());
    assert_eq!(children.len(), 0);
}

#[async_std::test]

async fn unique_children_names() {
    let (config, store) = prepare_test(16).await;

    let manager = Manager::<()>::new(config, Box::new(store)).await;
    assert!(manager.is_ok(), "Failed to create a manager");
    let mut manager = manager.unwrap();

    manager.create_root().await.unwrap();

    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "file.txt",
        vec![],
        vec![],
    );

    manager.create(&mut leaf_meta, None).await.unwrap();

    let res = manager.create(&mut leaf_meta, None).await;
    assert!(res.is_err());
}

#[async_std::test]

async fn child_by_name() {
    let (config, store) = prepare_test(17).await;

    let manager = Manager::<()>::new(config, Box::new(store)).await;
    assert!(manager.is_ok(), "Failed to create a manager");
    let mut manager = manager.unwrap();

    manager.create_root().await.unwrap();

    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "file.txt",
        vec![],
        vec![],
    );

    manager.create(&mut leaf_meta, None).await.unwrap();

    let mut leaf_meta = ResourceMetadata::new(
        &2.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "photo.png",
        vec![],
        vec![],
    );

    manager.create(&mut leaf_meta, None).await.unwrap();

    let file = manager.child_by_name(&ROOT_ID, "file.txt").await.unwrap();
    assert_eq!(file.name(), "file.txt");

    let image = manager.child_by_name(&ROOT_ID, "photo.png").await.unwrap();
    assert_eq!(image.name(), "photo.png");
}

#[async_std::test]

async fn migration_check() {
    let (config, store) = prepare_test(18).await;
    let (_config, store2) = prepare_test(18).await;

    {
        let manager = Manager::<()>::new(config.clone(), Box::new(store)).await;
        assert!(manager.is_ok(), "Failed to create first manager");
        let mut manager = manager.unwrap();

        manager.create_root().await.unwrap();

        manager.close().await;
    }

    {
        let manager = Manager::<()>::new(config, Box::new(store2)).await;
        assert!(manager.is_ok(), "Failed to create second manager");
        let manager = manager.unwrap();

        let has_root = manager.has_object(&ROOT_ID).await.unwrap();
        assert!(has_root);

        manager.close().await;
    }
}

#[async_std::test]
async fn frecency_update() {
    let (config, store) = prepare_test(19).await;

    let manager = Manager::<()>::new(config.clone(), Box::new(store)).await;
    assert!(manager.is_ok(), "Failed to create first manager");
    let mut manager = manager.unwrap();

    manager.create_root().await.unwrap();

    let meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(meta.scorer().frecency(), 0);

    manager
        .visit(&meta.id(), &VisitEntry::now(VisitPriority::Normal))
        .await
        .unwrap();
    let meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(meta.scorer().frecency(), 100);
}

#[async_std::test]
async fn index_places_mdn() {
    let (config, store) = prepare_test(20).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();
    manager.add_indexer(Box::new(create_places_indexer()));

    manager.create_root().await.unwrap();
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "ecdf525a-e5d6-11eb-9c9b-d3fd1d0ea335",
        vec!["places".into()],
        vec![],
    );

    let places1 = fs::File::open("./test-fixtures/places-mdn.json")
        .await
        .unwrap();

    manager
        .create(
            &mut leaf_meta,
            Some(Variant::new(
                named_variant("default", "application/x-places+json"),
                Box::new(places1),
            )),
        )
        .await
        .unwrap();

    // Found in the url.
    let results = manager.by_text("mdn", Some("places".into())).await.unwrap();
    assert_eq!(results.len(), 1);
}

#[async_std::test]
async fn import_from_path() {
    let (config, store) = prepare_test(21).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    manager.create_root().await.unwrap();

    // Wrong file.
    let meta = manager
        .import_from_path(&ROOT_ID, "./test-fixtures/unknown.txt", false)
        .await;
    assert_eq!(
        meta,
        Err(ResourceStoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No such file or directory"
        )))
    );

    // Correct file
    let meta = manager
        .import_from_path(&ROOT_ID, "./test-fixtures/import.txt", false)
        .await
        .unwrap();
    assert_eq!(meta.name(), "import.txt".to_owned());

    // Wrong parent.
    let meta = manager
        .import_from_path(&meta.id(), "./test-fixtures/import.txt", false)
        .await;
    assert_eq!(meta, Err(ResourceStoreError::InvalidContainerId));

    // Duplicate name -> renaming resource.
    let meta = manager
        .import_from_path(&ROOT_ID, "./test-fixtures/import.txt", false)
        .await
        .unwrap();
    assert_eq!(meta.name(), "import(1).txt".to_owned());
}

#[async_std::test]
async fn container_size() {
    let (config, store) = prepare_test(22).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

    create_hierarchy(&mut manager).await;

    let size = manager.container_size(&ROOT_ID).await.unwrap();
    assert_eq!(size, 798);
}

#[derive(Default)]
struct Observer {
    tracker: Rc<Tracker>,
}

#[derive(Default)]
struct Tracker {
    created: usize,
    modified: usize,
    deleted: usize,
    child_created: usize,
    child_modified: usize,
    child_deleted: usize,
}

impl Tracker {
    fn assert(
        &self,
        created: usize,
        modified: usize,
        deleted: usize,
        child_created: usize,
        child_modified: usize,
        child_deleted: usize,
    ) {
        assert_eq!(self.created, created);
        assert_eq!(self.modified, modified);
        assert_eq!(self.deleted, deleted);
        assert_eq!(self.child_created, child_created);
        assert_eq!(self.child_modified, child_modified);
        assert_eq!(self.child_deleted, child_deleted);
    }
}

impl ModificationObserver for Observer {
    type Inner = Rc<Tracker>;

    fn modified(&mut self, modification: &ResourceModification) {
        // println!("{:?}", modification);
        let tracker = Rc::get_mut(&mut self.tracker).unwrap();
        match modification {
            ResourceModification::Created(_) => tracker.created += 1,
            ResourceModification::Modified(_) => tracker.modified += 1,
            ResourceModification::Deleted(_) => tracker.deleted += 1,
            ResourceModification::ChildCreated(_) => tracker.child_created += 1,
            ResourceModification::ChildModified(_) => tracker.child_modified += 1,
            ResourceModification::ChildDeleted(_) => tracker.child_deleted += 1,
        }
    }

    fn get_inner(&mut self) -> &mut Self::Inner {
        &mut self.tracker
    }
}

#[async_std::test]
async fn observers() {
    let (config, store) = prepare_test(23).await;

    let mut manager = Manager::new(config, Box::new(store)).await.unwrap();

    let observer_id = manager.add_observer(Box::<Observer>::default());

    manager.create_root().await.unwrap();

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 1);
        assert_eq!(tracker.modified, 0);
        assert_eq!(tracker.deleted, 0);
    });

    // Add a leaf node.
    // println!("Adding leaf");
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "A leaf",
        vec![],
        vec![],
    );

    manager.create(&mut leaf_meta, None).await.unwrap();
    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 2);
        assert_eq!(tracker.modified, 1);
        assert_eq!(tracker.deleted, 0);
    });

    // Remove the leaf node.
    // println!("Removing leaf");
    manager.delete(&leaf_meta.id()).await.unwrap();
    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 2);
        assert_eq!(tracker.modified, 2);
        assert_eq!(tracker.deleted, 1);
    });

    // Add a new container
    // println!("Adding container");
    let mut container_meta = ResourceMetadata::new(
        &2.into(),
        &ROOT_ID,
        ResourceKind::Container,
        "A container",
        vec![],
        vec![],
    );
    manager.create(&mut container_meta, None).await.unwrap();
    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 3);
        assert_eq!(tracker.modified, 3);
        assert_eq!(tracker.deleted, 1);
    });
    // Add the leaf to this container.
    // println!("Adding leaf to container");
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &container_meta.id(),
        ResourceKind::Leaf,
        "A leaf",
        vec![],
        vec![],
    );
    manager.create(&mut leaf_meta, None).await.unwrap();
    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 4);
        assert_eq!(tracker.modified, 4);
        assert_eq!(tracker.deleted, 1);
    });

    // Remove the sub container.
    // println!("Removing sub container");
    manager.delete(&container_meta.id()).await.unwrap();
    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 4);
        assert_eq!(tracker.modified, 5);
        assert_eq!(tracker.deleted, 3);
    });

    manager.remove_observer(observer_id);
    manager.with_observer(observer_id, &mut |_observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        panic!("This observer should have been removed!");
    });
}

#[async_std::test]
async fn add_remove_tags() {
    let (config, store) = prepare_test(24).await;

    let mut manager = Manager::new(config, Box::new(store)).await.unwrap();

    manager.create_root().await.unwrap();

    let observer_id = manager.add_observer(Box::<Observer>::default());

    let meta = manager.get_metadata(&ROOT_ID).await.unwrap();

    // Start with no tags
    assert_eq!(meta.tags().len(), 0);

    // Add a tag.
    let meta = manager.add_tag(&ROOT_ID, "tag1").await.unwrap();
    assert_eq!(meta.tags().len(), 1);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 1);
    });

    // Add the same tag again. Not an error!
    let meta = manager.add_tag(&ROOT_ID, "tag1").await.unwrap();
    assert_eq!(meta.tags().len(), 1);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 1);
    });

    // Add a new tag.
    let meta = manager.add_tag(&ROOT_ID, "tag2").await.unwrap();
    assert_eq!(meta.tags().len(), 2);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 2);
    });

    // Remove an unknown tag. Not an error!
    let meta = manager.remove_tag(&ROOT_ID, "tag3").await.unwrap();
    assert_eq!(meta.tags().len(), 2);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 2);
    });

    // Remove an existing tag.
    let meta = manager.remove_tag(&ROOT_ID, "tag1").await.unwrap();
    assert_eq!(meta.tags().len(), 1);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 3);
    });

    // Only tag2 should remain.
    let meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(meta.tags(), &vec!["tag2".to_owned()]);

    // Clears the local cache of the manager to force redhydratation,
    // and check that tag2 is still present.
    manager.clear().await.unwrap();
    assert_eq!(manager.resource_count().await.unwrap(), 0);

    let meta = manager.get_metadata(&ROOT_ID).await.unwrap();
    assert_eq!(meta.tags(), &vec!["tag2".to_owned()]);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 3);
    });

    // Create a leaf resource.
    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "A leaf",
        vec![],
        vec![],
    );
    manager.create(&mut leaf_meta, None).await.unwrap();

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 4);
    });

    // Add a tag to the leaf, and check this triggers a modification in the root observer.
    let meta = manager.add_tag(&1.into(), "left_tag").await.unwrap();
    assert_eq!(meta.tags().len(), 1);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 5);
        assert_eq!(tracker.child_modified, 1);
    });

    // Remove a tag from the leaf, and check this triggers a modification in the root observer.
    let meta = manager.remove_tag(&1.into(), "left_tag").await.unwrap();
    assert_eq!(meta.tags().len(), 0);

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.modified, 6);
        assert_eq!(tracker.child_modified, 2);
    });
}

#[async_std::test]
async fn copy_resource() {
    use async_std::io::ReadExt;

    let (config, store) = prepare_test(25).await;

    let mut manager = Manager::new(config, Box::new(store)).await.unwrap();

    manager.create_root().await.unwrap();

    let mut source_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "Source",
        vec![],
        vec![],
    );
    manager
        .create(&mut source_meta, Some(default_content().await))
        .await
        .unwrap();

    // meta1: container with a resource of the same name.
    let mut target_meta1 = ResourceMetadata::new(
        &2.into(),
        &ROOT_ID,
        ResourceKind::Container,
        "Target 1 Container",
        vec![],
        vec![],
    );
    manager.create(&mut target_meta1, None).await.unwrap();

    let mut leaf = ResourceMetadata::new(
        &3.into(),
        &target_meta1.id(),
        ResourceKind::Leaf,
        "Source",
        vec![],
        vec![],
    );
    manager
        .create(&mut leaf, Some(default_content().await))
        .await
        .unwrap();

    // meta2: empty container.
    let mut target_meta2 = ResourceMetadata::new(
        &4.into(),
        &ROOT_ID,
        ResourceKind::Container,
        "Target 2 Container",
        vec![],
        vec![],
    );
    manager.create(&mut target_meta2, None).await.unwrap();

    // Copying containers is not supported yet.
    assert_eq!(
        manager.copy_resource(&target_meta1.id(), &10.into()).await,
        Err(ResourceStoreError::Custom(
            "Copying containers is not supported yet.".into(),
        ))
    );

    // Copying to an unknown container will fail.
    assert_eq!(
        manager.copy_resource(&source_meta.id(), &10.into()).await,
        Err(ResourceStoreError::InvalidContainerId)
    );

    // Copying to a container where a similarly named resource exists will fail.
    assert_eq!(
        manager
            .copy_resource(&source_meta.id(), &target_meta1.id())
            .await,
        Err(ResourceStoreError::ResourceAlreadyExists)
    );

    let observer_id = manager.add_observer(Box::<Observer>::default());

    let new_meta = manager
        .copy_resource(&source_meta.id(), &target_meta2.id())
        .await
        .unwrap();

    // Read the content of the default variant.
    let mut variant = manager.get_leaf(&new_meta.id(), "default").await.unwrap();
    let mut content = String::new();
    let _ = variant.1.read_to_string(&mut content).await.unwrap();
    assert_eq!(content.len(), 93);
    assert_eq!(&content[0..32], "#!/bin/bash\n\nset -x -e\n\nrm build");

    manager.with_observer(observer_id, &mut |observer: &mut Box<
        dyn ModificationObserver<Inner = Rc<Tracker>>,
    >| {
        let tracker = observer.get_inner();
        assert_eq!(tracker.created, 1);
    });
}

#[async_std::test]
async fn tags_persistence() {
    let (config, store) = prepare_test(26).await;

    {
        let mut manager = Manager::<()>::new(config.clone(), Box::new(store))
            .await
            .unwrap();

        manager.create_root().await.unwrap();

        let meta = manager.get_metadata(&ROOT_ID).await.unwrap();

        // Start with no tags
        assert_eq!(meta.tags().len(), 0);

        // Add a tag.
        let meta = manager.add_tag(&ROOT_ID, "tag1").await.unwrap();
        assert_eq!(meta.tags().len(), 1);
    }

    {
        let path = format!("./test-content/{}", 25);
        let store = FileStore::new(
            &path,
            Box::new(DefaultResourceNameProvider),
            Box::new(IdentityTransformer),
        )
        .await
        .unwrap();

        let mut manager = Manager::<()>::new(config.clone(), Box::new(store))
            .await
            .unwrap();

        // Reload the root and check the persisted state.
        let (root_meta, _) = manager.get_root().await.unwrap();
        assert!(root_meta.id().is_root());
        assert_eq!(root_meta.tags().len(), 1);

        // Remove the tag.
        let meta = manager.remove_tag(&ROOT_ID, "tag1").await.unwrap();
        assert_eq!(meta.tags().len(), 0);
    }

    {
        let path = format!("./test-content/{}", 25);
        let store = FileStore::new(
            &path,
            Box::new(DefaultResourceNameProvider),
            Box::new(IdentityTransformer),
        )
        .await
        .unwrap();

        let mut manager = Manager::<()>::new(config.clone(), Box::new(store))
            .await
            .unwrap();

        // Reload the root and check the persisted state.
        let (root_meta, _) = manager.get_root().await.unwrap();
        assert!(root_meta.id().is_root());
        assert_eq!(root_meta.tags().len(), 0);
    }
}

#[async_std::test]
async fn move_resource() {
    let (config, store) = prepare_test(27).await;

    {
        let mut manager = Manager::new(config.clone(), Box::new(store)).await.unwrap();

        let observer_id = manager.add_observer(Box::<Observer>::default());

        manager.create_root().await.unwrap();

        manager.with_observer(observer_id, &mut |observer: &mut Box<
            dyn ModificationObserver<Inner = Rc<Tracker>>,
        >| {
            let tracker = observer.get_inner();
            tracker.assert(1, 0, 0, 0, 0, 0);
        });

        // Create two sub containers.
        let mut container1 = ResourceMetadata::new(
            &1.into(),
            &ROOT_ID,
            ResourceKind::Container,
            "container_1",
            vec![],
            vec![],
        );
        manager.create(&mut container1, None).await.unwrap();

        let mut container2 = ResourceMetadata::new(
            &2.into(),
            &ROOT_ID,
            ResourceKind::Container,
            "container_2",
            vec![],
            vec![],
        );
        manager.create(&mut container2, None).await.unwrap();

        // Add a leaf to container_1
        let mut leaf = ResourceMetadata::new(
            &3.into(),
            &container1.id(),
            ResourceKind::Leaf,
            "leaf",
            vec![],
            vec![],
        );
        manager
            .create(&mut leaf, Some(default_content().await))
            .await
            .unwrap();

        let (_, children) = manager.get_container(&container1.id()).await.unwrap();
        assert_eq!(children.len(), 1);

        let (_, children) = manager.get_container(&container2.id()).await.unwrap();
        assert_eq!(children.len(), 0);

        manager
            .move_resource(&leaf.id(), &container2.id())
            .await
            .unwrap();

        // 4 resources created: root, 2 containers, 1 leaf
        // 3 resource modified: root x 2, container 1
        // 4 children created: container 1, container 2, leaf, leaf when moved.
        // 1 children deleted when moving the leaf.
        manager.with_observer(observer_id, &mut |observer: &mut Box<
            dyn ModificationObserver<Inner = Rc<Tracker>>,
        >| {
            let tracker = observer.get_inner();
            tracker.assert(4, 3, 0, 4, 0, 1);
        });

        let (_, children) = manager.get_container(&container1.id()).await.unwrap();
        assert_eq!(children.len(), 0);

        let (_, children) = manager.get_container(&container2.id()).await.unwrap();
        assert_eq!(children.len(), 1);
    }

    // Verify persistence
    {
        let path = format!("./test-content/{}", 27);
        let store = FileStore::new(
            &path,
            Box::new(DefaultResourceNameProvider),
            Box::new(IdentityTransformer),
        )
        .await
        .unwrap();

        let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();

        let (_, children) = manager.get_container(&1.into()).await.unwrap();
        assert_eq!(children.len(), 0);

        let (_, children) = manager.get_container(&2.into()).await.unwrap();
        assert_eq!(children.len(), 1);
    }
}

#[async_std::test]
async fn update_variant() {
    let (config, store) = prepare_test(28).await;

    let mut manager = Manager::<()>::new(config, Box::new(store)).await.unwrap();
    manager.add_indexer(Box::new(create_places_indexer()));

    manager.create_root().await.unwrap();

    let places1 = fs::File::open("./test-fixtures/places-1.json")
        .await
        .unwrap();

    let mut leaf_meta = ResourceMetadata::new(
        &1.into(),
        &ROOT_ID,
        ResourceKind::Leaf,
        "default-wallpaper",
        vec![],
        vec![],
    );

    manager
        .create(
            &mut leaf_meta,
            Some(Variant::new(
                VariantMetadata::new("default", "application/x-places+json", 110),
                Box::new(places1),
            )),
        )
        .await
        .unwrap();

    let meta = manager.get_metadata(&leaf_meta.id()).await.unwrap();
    let variant = &meta.variants()[0];
    assert_eq!(variant.mime_type(), "application/x-places+json");
    assert_eq!(variant.size(), 110);

    // Update the object with new content.
    let text = fs::File::open("./test-fixtures/import.txt").await.unwrap();
    manager
        .update_variant(
            &leaf_meta.id(),
            Variant::new(VariantMetadata::new("default", "text/plain", 13), Box::new(text)),
        )
        .await
        .unwrap();
    let meta = manager.get_metadata(&leaf_meta.id()).await.unwrap();
    let variant = &meta.variants()[0];
    assert_eq!(variant.mime_type(), "text/plain");
    assert_eq!(variant.size(), 13);
}
