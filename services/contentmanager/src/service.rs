use crate::config::Config;
use crate::cursor::MetadataCursorImpl;
use crate::generated::{common::*, service::*};
use async_std::io::ReadExt;
use async_std::path::Path;
use async_std::task;
use common::core::BaseMessage;
use common::object_tracker::ObjectTracker;
use common::observers::{ObserverTracker, ServiceObserverTracker};
use common::traits::{
    DispatcherId, ObjectTrackerMethods, OriginAttributes, Service, SessionSupport, Shared,
    SharedServiceState, SharedSessionContext, StateLogger, TrackerId,
};
use common::Blob;
use common::JsonValue;
use costaeres::array::Array;
use costaeres::common::{
    DefaultResourceNameProvider, IdFrec, IdentityTransformer, ResourceId, ResourceMetadata,
    ResourceStoreError, Variant as VariantC, VariantContent, ROOT_ID,
};
use costaeres::config::Config as CoConfig;
use costaeres::file_store::FileStore;
use costaeres::indexer::*;
use costaeres::manager::{
    Manager, ModificationKind as ModificationKindC, ModificationObserver,
    ResourceModification as ResourceModificationC,
};
use costaeres::scorer::VisitEntry;
use log::{debug, error, info};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

pub(crate) struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    pub fn start(name: &str) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        log::info!("{} : {}ms", self.name, self.start.elapsed().as_millis());
    }
}

type ObservedManager = Manager<Rc<ObserverItems>>;

pub struct State {
    // The holder of content.
    manager: ObservedManager,
    active_keys: Arc<Mutex<HashSet<String>>>,
    store_path: String,
}

impl StateLogger for State {}

impl Into<State> for &Config {
    fn into(self) -> State {
        let config_path = self.storage_path();
        let config_path = Path::new(&config_path);
        let store_path = config_path.join("data");
        let _ = std::fs::create_dir_all(&store_path);
        task::block_on(async {
            let store = FileStore::new(
                &store_path,
                Box::new(DefaultResourceNameProvider),
                Box::new(IdentityTransformer),
            )
            .await
            .unwrap();

            let sqlite_path = config_path.to_path_buf().join("manager.sqlite");
            let config = CoConfig {
                db_path: sqlite_path.to_string_lossy().into(),
                data_dir: store_path.to_string_lossy().into(),
                metadata_cache_capacity: self.metadata_cache_capacity(),
            };

            let mut manager = match Manager::new(config, Box::new(store)).await {
                Ok(manager) => manager,
                Err(err) => panic!("Failed to get manager: {:?}", err),
            };

            manager.add_indexer(Box::new(create_places_indexer()));
            manager.add_indexer(Box::new(create_contacts_indexer()));
            manager.add_indexer(Box::new(create_media_indexer()));

            if manager.get_root().await.is_err() {
                if let Err(err) = manager.create_root().await {
                    error!("Failed to create root content object: {}", err);
                }
            }

            State {
                manager,
                active_keys: Default::default(),
                store_path: store_path.display().to_string(),
            }
        })
    }
}

pub struct ContentManagerService {
    id: TrackerId,
    state: Shared<State>,
    dispatcher_id: DispatcherId,
    tracker: Arc<Mutex<ContentManagerTrackerType>>,
    proxy_tracker: ContentManagerProxyTracker,
    observers: ServiceObserverTracker<ResourceId>,
    _origin_attributes: OriginAttributes,
    http_key: String,
}

impl ContentManager for ContentManagerService {
    fn get_tracker(&mut self) -> Arc<Mutex<ContentManagerTrackerType>> {
        self.tracker.clone()
    }

    fn get_proxy_tracker(&mut self) -> &mut ContentManagerProxyTracker {
        &mut self.proxy_tracker
    }
}

impl From<Payload> for VariantContent {
    fn from(val: Payload) -> Self {
        let variant = VariantC::new(&val.variant, &val.blob.mime_type(), val.blob.len() as _);
        VariantContent::new(variant, Box::new(Array::new(val.blob.take_data())))
    }
}

fn variant_content_for_blob(variant: &str, blob: Blob) -> VariantContent {
    let variant = VariantC::new(variant, &blob.mime_type(), blob.len() as _);
    VariantContent::new(variant, Box::new(Array::new(blob.take_data())))
}

impl ContentManagerService {
    pub fn get_http_state() -> costaeres::http::HttpData {
        let shared_data = Self::shared_state().clone();
        let shared_data = shared_data.lock();
        let keys = shared_data.active_keys.clone();
        let store_path = shared_data.store_path.clone();

        let store = task::block_on(async move {
            FileStore::new(
                &store_path,
                Box::new(DefaultResourceNameProvider),
                Box::new(IdentityTransformer),
            )
            .await
            .unwrap()
        });

        costaeres::http::HttpData { store, keys }
    }

    fn get_cursor(
        tracker: Arc<Mutex<ContentManagerTrackerType>>,
        data: Vec<ResourceMetadata>,
    ) -> Rc<MetadataCursorImpl> {
        let mut tracker = tracker.lock();
        let cursor = Rc::new(MetadataCursorImpl::new(tracker.next_id(), data));
        tracker.track(ContentManagerTrackedObject::MetadataCursor(cursor.clone()));
        cursor
    }

    async fn create_task(
        state: Shared<State>,
        data: CreationData,
        variant: &str,
        blob: Option<Blob>,
    ) -> Result<Metadata, ResourceStoreError> {
        let mut lock = state.lock();
        let manager = &mut lock.manager;
        // 1. get a new id.
        let id = ResourceId::new();
        debug!("Will use id {} for new object", id);
        // 2. Create a full meta-data object
        let mut meta = ResourceMetadata::new(
            &id,
            &data.parent.into(),
            data.kind.into(),
            &data.name,
            data.tags,
            vec![],
        );
        // 3. Create a new object.
        if let Some(blob) = blob {
            manager
                .create(&mut meta, Some(variant_content_for_blob(variant, blob)))
                .await?;
        } else {
            manager.create(&mut meta, None).await?;
        }

        Ok(meta.into())
    }

    async fn update_variant_task(
        state: Shared<State>,
        id: &str,
        variant: &str,
        blob: Blob,
    ) -> Result<(), ResourceStoreError> {
        let mut lock = state.lock();
        let manager = &mut lock.manager;

        let id: ResourceId = id.to_owned().into();

        manager
            .update_variant(&id, variant_content_for_blob(variant, blob))
            .await?;

        Ok(())
    }

    async fn meta_from_ids(
        manager: &mut ObservedManager,
        ids: &[IdFrec],
        max_count: usize,
    ) -> Vec<ResourceMetadata> {
        let _timer = Timer::start("meta_from_ids");
        let mut all_meta: Vec<ResourceMetadata> = Vec::with_capacity(max_count);
        for id_frec in ids {
            if let Ok(meta) = manager.get_metadata(&id_frec.id).await {
                all_meta.push(meta);
            }
            if all_meta.len() == max_count {
                break;
            }
        }

        all_meta
    }

    async fn search_task(
        state: Shared<State>,
        query: &str,
        max_count: usize,
        tag: Option<String>,
    ) -> Result<Vec<ResourceMetadata>, ResourceStoreError> {
        let _timer = Timer::start("search_task");
        let mut lock = state.lock();
        let manager = &mut lock.manager;

        let all_ids = manager.by_text(query, tag).await?;
        Ok(Self::meta_from_ids(manager, &all_ids, max_count).await)
    }

    async fn top_by_frecency_task(
        state: Shared<State>,
        max_count: usize,
    ) -> Result<Vec<ResourceMetadata>, ResourceStoreError> {
        let _timer = Timer::start("top_by_frecency_task");
        let mut lock = state.lock();
        let manager = &mut lock.manager;

        let all_ids = manager.top_by_frecency(max_count as _).await?;
        Ok(Self::meta_from_ids(manager, &all_ids, max_count).await)
    }

    async fn last_modified_task(
        state: Shared<State>,
        max_count: usize,
    ) -> Result<Vec<ResourceMetadata>, ResourceStoreError> {
        let _timer = Timer::start("last_modified_task");
        let mut lock = state.lock();
        let manager = &mut lock.manager;

        let all_ids = manager.last_modified(max_count as _).await?;
        Ok(Self::meta_from_ids(manager, &all_ids, max_count).await)
    }

    async fn get_full_path_task(
        state: Shared<State>,
        id: &ResourceId,
    ) -> Result<Vec<ResourceMetadata>, ResourceStoreError> {
        let _timer = Timer::start("get_full_path_task");
        let mut lock = state.lock();
        let manager = &mut lock.manager;

        Ok(manager.get_full_path(id).await?)
    }
}

impl ContentStoreMethods for ContentManagerService {
    fn add_observer(
        &mut self,
        responder: ContentStoreAddObserverResponder,
        resource: String,
        observer: ObjectRef,
    ) {
        debug!("Adding observer for {}", resource);

        match self.proxy_tracker.get(&observer) {
            Some(ContentManagerProxy::ModificationObserver(proxy)) => {
                // let id = self.shared_obj.lock().add_observer(reason, proxy);
                let state = &mut self.state.lock();

                let mut id = 0;
                let resource_id: ResourceId = resource.into();
                let resource_id2 = resource_id.clone();
                state.manager.with_observer(1, &mut |observer: &mut Box<
                    dyn ModificationObserver<Inner = Rc<ObserverItems>>,
                >| {
                    let mut inner = observer.get_inner();
                    let items = Rc::get_mut(&mut inner).unwrap();
                    id = items
                        .resource_observers
                        .add(resource_id2.clone(), proxy.clone());
                });

                self.observers.add(observer.into(), resource_id.clone(), id);
                responder.resolve();
            }
            _ => {
                error!("Failed to get tracked observer");
                responder.reject();
            }
        }
    }

    fn remove_observer(
        &mut self,
        responder: ContentStoreRemoveObserverResponder,
        resource: String,
        observer: ObjectRef,
    ) {
        debug!("Removing observer for {}", resource);

        if self.proxy_tracker.contains_key(&observer) {
            let state = &mut self.state.lock();

            let resource_id: ResourceId = resource.into();

            let mut obt = Default::default();
            state
                .manager
                .with_observer(1, &mut |content_observer: &mut Box<
                    dyn ModificationObserver<Inner = Rc<ObserverItems>>,
                >| {
                    let mut inner = content_observer.get_inner();
                    let items = Rc::get_mut(&mut inner).unwrap();
                    obt = items.resource_observers.clone();
                });

            let removed = self
                .observers
                .remove(observer.into(), resource_id.clone(), &mut obt);

            // Put back the modified `obt`in items...
            state
                .manager
                .with_observer(1, &mut |content_observer: &mut Box<
                    dyn ModificationObserver<Inner = Rc<ObserverItems>>,
                >| {
                    let mut inner = content_observer.get_inner();
                    let items = Rc::get_mut(&mut inner).unwrap();
                    items.resource_observers = obt.clone();
                });

            if removed {
                responder.resolve();
            } else {
                error!("Failed to find observer in list");
                responder.reject();
            }
        } else {
            error!("Failed to find proxy for this observer");
            responder.reject();
        }
    }

    fn by_ids(&mut self, responder: ContentStoreByIdsResponder, _ids: Vec<String>) {
        responder.reject();
    }

    fn children_of(&mut self, responder: ContentStoreChildrenOfResponder, id: String) {
        debug!("children_of {}", id);
        let state = self.state.clone();
        let tracker = self.get_tracker();
        task::block_on(async move {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.get_container(&id.clone().into()).await
            };
            match res {
                Ok((_obj, children)) => {
                    debug!("Got {} children for {}", children.len(), id);
                    let cursor = Self::get_cursor(tracker, children);
                    responder.resolve(cursor);
                }
                Err(err) => {
                    error!("Failed to get children of {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn createobj(
        &mut self,
        responder: ContentStoreCreateobjResponder,
        data: CreationData,
        variant: String,
        blob: Option<Blob>,
    ) {
        debug!("createobj {:?} {}", data.kind, data.name);
        let state = self.state.clone();
        task::block_on(async move {
            match Self::create_task(state, data, &variant, blob).await {
                Ok(meta) => responder.resolve(meta),
                Err(err) => {
                    error!("Failed to create object: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn get_root(&mut self, responder: ContentStoreGetRootResponder) {
        debug!("get_root");
        let state = self.state.clone();
        task::block_on(async move {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.get_metadata(&ROOT_ID).await
            };
            match res {
                Ok(value) => responder.resolve(value.into()),
                Err(err) => {
                    error!("Failed to get root object: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn get_variant(
        &mut self,
        responder: ContentStoreGetVariantResponder,
        id: String,
        variant_name: Option<String>,
    ) {
        debug!("get_variant {} {:?}", id, variant_name);
        let state = self.state.clone();
        let variant_name = variant_name.unwrap_or_else(|| "default".into());
        task::block_on(async move {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.get_leaf(&id.clone().into(), &variant_name).await
            };
            match res {
                Ok((meta, mut reader)) => {
                    let mut buffer = vec![];
                    if reader.read_to_end(&mut buffer).await.is_ok() {
                        // Get the mime type for this variant.
                        let mime_type = meta
                            .mime_type_for_variant(&variant_name)
                            .unwrap_or_else(|| "application/octet-stream".into());
                        let blob = Blob::new(&mime_type, buffer);
                        responder.resolve(blob);
                    } else {
                        responder.reject();
                    }
                }
                Err(err) => {
                    error!("Failed to get leaf content for resource {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn get_variant_json(
        &mut self,
        responder: ContentStoreGetVariantJsonResponder,
        id: String,
        variant_name: Option<String>,
    ) {
        debug!("get_variant_as_json {} {:?}", id, variant_name);
        let state = self.state.clone();
        let variant_name = variant_name.unwrap_or_else(|| "default".into());
        task::block_on(async move {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.get_leaf(&id.clone().into(), &variant_name).await
            };
            match res {
                Ok((meta, mut reader)) => {
                    // Check the variant mime type.
                    let mime_type = meta
                        .mime_type_for_variant(&variant_name)
                        .unwrap_or_else(|| "application/octet-stream".into());
                    if !mime_type.contains("json") {
                        error!(
                            "Expected json mime type for resource {} but got {}",
                            id, mime_type
                        );
                        responder.reject();
                        return;
                    }

                    // Read the content as Json.
                    let mut buffer = vec![];
                    if reader.read_to_end(&mut buffer).await.is_ok() {
                        let maybe_json: serde_json::Result<serde_json::Value> =
                            serde_json::from_slice(&buffer);
                        match maybe_json {
                            Ok(json) => {
                                responder.resolve(JsonValue::from(json));
                            }
                            Err(err) => {
                                error!(
                                    "Failed to parse json for {}/{} : {}",
                                    id, variant_name, err
                                );
                                responder.reject();
                            }
                        }
                    } else {
                        responder.reject();
                    }
                }
                Err(err) => {
                    error!("Failed to get leaf content for resource {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn get_metadata(&mut self, responder: ContentStoreGetMetadataResponder, id: String) {
        debug!("get_metadata {}", id);
        let state = self.state.clone();
        task::block_on(async move {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.get_metadata(&id.clone().into()).await
            };
            match res {
                Ok(value) => responder.resolve(value.into()),
                Err(err) => {
                    error!("Failed to get metadata for resource {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn update_variant(
        &mut self,
        responder: ContentStoreUpdateVariantResponder,
        id: String,
        variant: String,
        blob: Blob,
    ) {
        debug!("update {}", &id);
        let state = self.state.clone();
        task::block_on(async move {
            match Self::update_variant_task(state, &id, &variant, blob).await {
                Ok(()) => responder.resolve(),
                Err(err) => {
                    error!("Failed to update resource {}: {}", &id, err);
                    responder.reject();
                }
            }
        });
    }

    fn delete(&mut self, responder: ContentStoreDeleteResponder, id: String) {
        let state = self.state.clone();
        task::block_on(async {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.delete(&id.clone().into()).await
            };
            match res {
                Ok(()) => responder.resolve(),
                Err(err) => {
                    error!("Failed to delete resource {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn delete_variant(
        &mut self,
        responder: ContentStoreDeleteVariantResponder,
        id: String,
        variant_name: String,
    ) {
        let state = self.state.clone();
        task::block_on(async {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager
                    .delete_variant(&id.clone().into(), &variant_name)
                    .await
            };
            match res {
                Ok(()) => responder.resolve(),
                Err(err) => {
                    error!("Failed to delete object {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn child_by_name(
        &mut self,
        responder: ContentStoreChildByNameResponder,
        parent: String,
        name: String,
    ) {
        let state = self.state.clone();
        task::block_on(async {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager.child_by_name(&parent.clone().into(), &name).await
            };
            match res {
                Ok(meta) => responder.resolve(meta.into()),
                Err(err) => {
                    error!("Failed to child_by_name {} {}: {}", parent, name, err);
                    responder.reject();
                }
            }
        });
    }

    fn search(
        &mut self,
        responder: ContentStoreSearchResponder,
        query: String,
        max_count: i64,
        tag: Option<String>,
    ) {
        let state = self.state.clone();
        let max_count = max_count as usize;
        let tracker = self.get_tracker();

        task::block_on(async {
            match Self::search_task(state, &query, max_count, tag).await {
                Ok(meta) => {
                    let cursor = Self::get_cursor(tracker, meta);
                    responder.resolve(cursor);
                }
                Err(err) => {
                    error!("Failed to get searh results: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn top_by_frecency(&mut self, responder: ContentStoreTopByFrecencyResponder, max_count: i64) {
        let state = self.state.clone();
        let max_count = max_count as usize;
        let tracker = self.get_tracker();

        task::block_on(async {
            match Self::top_by_frecency_task(state, max_count).await {
                Ok(meta) => {
                    let cursor = Self::get_cursor(tracker, meta);
                    responder.resolve(cursor);
                }
                Err(err) => {
                    error!("Failed to get top_by_frecency results: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn last_modified(&mut self, responder: ContentStoreLastModifiedResponder, max_count: i64) {
        let state = self.state.clone();
        let max_count = max_count as usize;
        let tracker = self.get_tracker();

        task::block_on(async {
            match Self::last_modified_task(state, max_count).await {
                Ok(meta) => {
                    let cursor = Self::get_cursor(tracker, meta);
                    responder.resolve(cursor);
                }
                Err(err) => {
                    error!("Failed to get last_modified results: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn get_full_path(&mut self, responder: ContentStoreGetFullPathResponder, id: String) {
        let state = self.state.clone();

        task::block_on(async {
            match Self::get_full_path_task(state, &id.into()).await {
                Ok(meta) => {
                    responder.resolve(meta.iter().map(|item| item.clone().into()).collect());
                }
                Err(err) => {
                    error!("Failed to get last_modified results: {}", err);
                    responder.reject();
                }
            }
        });
    }

    fn visit(&mut self, responder: ContentStoreVisitResponder, id: String, visit: VisitPriority) {
        let state = self.state.clone();
        task::block_on(async {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                manager
                    .visit(&id.clone().into(), &VisitEntry::now(visit.into()))
                    .await
            };
            match res {
                Ok(_) => responder.resolve(),
                Err(err) => {
                    error!("Failed to visit {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn visit_by_name(
        &mut self,
        responder: ContentStoreVisitByNameResponder,
        parent: String,
        name: String,
        visit: VisitPriority,
    ) {
        let state = self.state.clone();
        task::block_on(async {
            let res = {
                let mut lock = state.lock();
                let manager = &mut lock.manager;
                match manager.child_by_name(&parent.clone().into(), &name).await {
                    Ok(meta) => {
                        manager
                            .visit(&meta.id(), &VisitEntry::now(visit.into()))
                            .await
                    }
                    Err(err) => Err(err),
                }
            };
            match res {
                Ok(_) => responder.resolve(),
                Err(err) => {
                    error!("Failed to visit_by_name {} '{}': {}", parent, name, err);
                    responder.reject();
                }
            }
        });
    }

    fn import_from_path(
        &mut self,
        responder: ContentStoreImportFromPathResponder,
        parent: String,
        path: String,
        remove: bool,
    ) {
        let state = self.state.clone();
        task::block_on(async {
            let mut lock = state.lock();
            let manager = &mut lock.manager;
            match manager
                .import_from_path(&parent.clone().into(), &path, remove)
                .await
            {
                Ok(meta) => responder.resolve(meta.into()),
                Err(err) => {
                    error!("Failed to import from {}: {}", path, err);
                    responder.reject();
                }
            }
        });
    }

    fn container_size(&mut self, responder: ContentStoreContainerSizeResponder, id: String) {
        let state = self.state.clone();
        task::block_on(async {
            let mut lock = state.lock();
            let manager = &mut lock.manager;
            match manager.container_size(&id.clone().into()).await {
                Ok(size) => responder.resolve(size as _),
                Err(err) => {
                    error!("Failed to get container size for {}: {}", id, err);
                    responder.reject();
                }
            }
        });
    }

    fn http_key(&mut self, responder: ContentStoreHttpKeyResponder) {
        // Add the key to the set of active keys.
        let _ = self
            .state
            .lock()
            .active_keys
            .lock()
            .insert(self.http_key.clone());

        responder.resolve(self.http_key.clone());
    }

    fn add_tag(&mut self, responder: ContentStoreAddTagResponder, id: String, tag: String) {
        let state = self.state.clone();
        task::block_on(async {
            let mut lock = state.lock();
            let manager = &mut lock.manager;
            match manager.add_tag(&id.clone().into(), &tag.clone()).await {
                Ok(metadata) => responder.resolve(metadata.into()),
                Err(err) => {
                    error!("Failed to get add tag {} to {} : {}", tag, id, err);
                    responder.reject();
                }
            }
        });
    }

    fn remove_tag(&mut self, responder: ContentStoreRemoveTagResponder, id: String, tag: String) {
        let state = self.state.clone();
        task::block_on(async {
            let mut lock = state.lock();
            let manager = &mut lock.manager;
            match manager.remove_tag(&id.clone().into(), &tag.clone()).await {
                Ok(metadata) => responder.resolve(metadata.into()),
                Err(err) => {
                    error!("Failed to get add tag {} to {} : {}", tag, id, err);
                    responder.reject();
                }
            }
        });
    }

    fn copy_resource(
        &mut self,
        responder: ContentStoreCopyResourceResponder,
        source: String,
        target: String,
    ) {
        let state = self.state.clone();
        task::block_on(async {
            let mut lock = state.lock();
            let manager = &mut lock.manager;
            match manager
                .copy_resource(&source.clone().into(), &target.clone().into())
                .await
            {
                Ok(metadata) => responder.resolve(metadata.into()),
                Err(err) => {
                    error!("Failed to copy resource {} to {} : {}", source, target, err);
                    responder.reject();
                }
            }
        });
    }
}

common::impl_shared_state!(ContentManagerService, State, Config);

impl Into<ModificationKind> for ModificationKindC {
    fn into(self) -> ModificationKind {
        match self {
            Self::Created => ModificationKind::Created,
            Self::Deleted => ModificationKind::Deleted,
            Self::Modified => ModificationKind::Modified,
        }
    }
}

impl Into<ResourceModification> for &ResourceModificationC {
    fn into(self) -> ResourceModification {
        ResourceModification {
            id: self.id.clone().into(),
            kind: self.kind.clone().into(),
        }
    }
}

#[derive(Default)]
struct ObserverItems {
    event_broadcaster: ContentStoreEventBroadcaster,
    resource_observers: ObserverTracker<ResourceId, ModificationObserverProxy>,
}

struct Observer {
    // Handle to the event broadcaster to fire events when changes happen.
    inner: Rc<ObserverItems>,
}

// impl From<ResourceModification> for
impl ModificationObserver for Observer {
    type Inner = Rc<ObserverItems>;

    fn modified(&mut self, modification: &ResourceModificationC) {
        info!("Resource modification: {:?}", modification);

        self.inner
            .event_broadcaster
            .broadcast_onresourcemodified(modification.into());
        info!("Done broadcasting event {:?}", modification);

        let inner = Rc::get_mut(&mut self.inner).unwrap();
        inner
            .resource_observers
            .for_each(&modification.id, |proxy, id| {
                info!("Notifiying observer {}", id);
                proxy.modified(modification.into());
            });
    }

    fn get_inner<'a>(&'a mut self) -> &'a mut Self::Inner {
        &mut self.inner
    }
}

impl Service<ContentManagerService> for ContentManagerService {
    fn create(
        attrs: &OriginAttributes,
        _context: SharedSessionContext,
        helper: SessionSupport,
    ) -> Result<ContentManagerService, String> {
        info!("ContentManagerService::create");
        let service_id = helper.session_tracker_id().service();
        let event_dispatcher = ContentStoreEventDispatcher::from(helper, 0 /* object id */);
        let state = Self::shared_state();
        let state2 = state.clone();

        // Check if we already added an observer for content changes.
        let manager = &mut state2.lock().manager;
        if manager.observer_count() == 0 {
            let observer = Observer {
                inner: Default::default(),
            };
            manager.add_observer(Box::new(observer));
        }
        // Add the dispatcher to the broadcaster.
        let mut dispatcher_id = 0;
        // Since we have a single observer, its id is always 1.
        manager.with_observer(1, &mut |observer: &mut Box<
            dyn ModificationObserver<Inner = Rc<ObserverItems>>,
        >| {
            let mut inner = observer.get_inner();
            let items = Rc::get_mut(&mut inner).unwrap();
            dispatcher_id = items.event_broadcaster.add(&event_dispatcher);
        });

        Ok(ContentManagerService {
            id: service_id,
            state,
            dispatcher_id,
            tracker: Arc::new(Mutex::new(ObjectTracker::default())),
            proxy_tracker: ContentManagerProxyTracker::default(),
            _origin_attributes: attrs.clone(),
            observers: ServiceObserverTracker::default(),
            http_key: uuid::Uuid::new_v4().to_string(),
        })
    }

    // Returns a human readable version of the request.
    fn format_request(&mut self, _transport: &SessionSupport, message: &BaseMessage) -> String {
        let req: Result<ContentManagerFromClient, common::BincodeError> =
            common::deserialize_bincode(&message.content);
        match req {
            Ok(req) => {
                let full = format!("ContentManagerService request: {:?}", req);
                let len = std::cmp::min(256, full.len());
                (&full[..len]).into()
            }
            Err(err) => format!("Unable to format ContentManagerService request: {:?}", err),
        }
    }

    // Processes a request coming from the Session.
    fn on_request(&mut self, transport: &SessionSupport, message: &BaseMessage) {
        self.dispatch_request(transport, message);
    }

    fn release_object(&mut self, object_id: u32) -> bool {
        debug!("releasing object {}", object_id);
        self.tracker.lock().untrack(object_id)
    }
}

impl Drop for ContentManagerService {
    fn drop(&mut self) {
        debug!("Dropping Content Service #{}", self.id);
        let state = &mut self.state.lock();

        let dispatcher_id = self.dispatcher_id;
        state.manager.with_observer(1, &mut |observer: &mut Box<
            dyn ModificationObserver<Inner = Rc<ObserverItems>>,
        >| {
            let mut inner = observer.get_inner();
            let items = Rc::get_mut(&mut inner).unwrap();
            items.event_broadcaster.remove(dispatcher_id);
        });

        // TODO:
        // self.observers.clear(...);

        // Remove this instance http key from the valid key set.
        let _ = state.active_keys.lock().remove(&self.http_key);

        self.tracker.lock().clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common::traits::*;
    use std::collections::HashSet;

    use crate::config::Config;
    use crate::service::ContentManagerService;

    #[test]
    fn service_creation() {
        let session_context = SessionContext::default();
        let (sender, _receiver) = std::sync::mpsc::channel();
        let shared_sender = MessageSender::new(Box::new(StdSender::new(&sender)));

        let helper = SessionSupport::new(
            SessionTrackerId::from(0, 0),
            shared_sender,
            Shared::adopt(IdFactory::new(0)),
            Shared::default(),
        );

        ContentManagerService::init_shared_state(&Config::new("./test-content", 250));

        let _service: ContentManagerService = ContentManagerService::create(
            &OriginAttributes::new("test", HashSet::new()),
            Shared::adopt(session_context),
            helper,
        )
        .unwrap();
    }
}
