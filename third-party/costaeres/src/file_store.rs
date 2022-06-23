/// A file based storage engine.
/// Each object is stored in 2 files:
/// ${object.id}.meta for the metadata serialized as Json.
/// ${object.id}.content for the opaque content.
use crate::common::{
    BoxedReader, ResourceId, ResourceKind, ResourceMetadata, ResourceNameProvider, ResourceStore,
    ResourceStoreError, ResourceTransformer, Variant,
};
use async_std::{
    fs,
    fs::File,
    io::prelude::WriteExt,
    io::BufReader,
    path::{Path, PathBuf},
};
use async_trait::async_trait;
use log::error;
use speedy::{Readable, Writable};

macro_rules! custom_error {
    ($error:expr) => {
        Err(ResourceStoreError::Custom($error.into()))
    };
}

pub struct FileStore {
    root: PathBuf, // The root path of the storage.
    name_provider: Box<dyn ResourceNameProvider>,
    transformer: Box<dyn ResourceTransformer>,
}

impl FileStore {
    pub async fn new<P>(
        path: P,
        name_provider: Box<dyn ResourceNameProvider>,
        transformer: Box<dyn ResourceTransformer>,
    ) -> Result<Self, ResourceStoreError>
    where
        P: AsRef<Path>,
    {
        // Fail if the root is not an existing directory.
        let file = File::open(&path).await?;
        let meta = file.metadata().await?;
        if !meta.is_dir() {
            return custom_error!("NotDirectory");
        }
        let root = path.as_ref().to_path_buf();
        Ok(Self {
            root,
            name_provider,
            transformer,
        })
    }

    pub fn metadata_path(&self, id: &ResourceId) -> PathBuf {
        let mut metadata_path = self.root.clone();
        metadata_path.push(&self.name_provider.metadata_name(id));
        metadata_path
    }

    pub fn variant_path(&self, id: &ResourceId, variant: &str) -> PathBuf {
        let mut content_path = self.root.clone();
        content_path.push(&self.name_provider.variant_name(id, variant));
        content_path
    }

    /// Creates a file and set permission to rw for the owner only.
    async fn create_file<P: AsRef<Path>>(path: P) -> Result<File, ResourceStoreError> {
        use std::os::unix::fs::PermissionsExt;

        let file = File::create(&path).await?;
        file.set_permissions(async_std::fs::Permissions::from_mode(0o600))
            .await?;

        Ok(file)
    }

    async fn create_or_update(
        &self,
        metadata: &ResourceMetadata,
        content: Option<Variant>,
        create: bool,
    ) -> Result<(), ResourceStoreError> {
        // 0. TODO: check if we have enough storage available.

        let id = metadata.id();
        let meta_path = self.metadata_path(&id);

        // 1. When creating, check if we already have files for this id, and bail out if so.
        if create {
            let file = File::open(&meta_path).await;
            if file.is_ok() {
                error!("Can't create two files with path {}", meta_path.display());
                return Err(ResourceStoreError::ResourceAlreadyExists);
            }
        }

        // 2. Store the metadata.
        let mut file = Self::create_file(&meta_path).await?;
        let meta = self
            .transformer
            .transform_array_to(&metadata.write_to_vec()?);
        file.write_all(&meta).await?;
        file.sync_all().await?;

        // 3. Store the variants for leaf nodes.
        if metadata.kind() != ResourceKind::Leaf {
            return Ok(());
        }

        if let Some(content) = content {
            let name = content.metadata.name();
            if !metadata.has_variant(&name) {
                error!("Variant '{}' is not in metadata.", name);
                return Err(ResourceStoreError::InvalidVariant(name));
            }
            let mut file = Self::create_file(&self.variant_path(&id, &name)).await?;
            file.set_len(content.metadata.size() as _).await?;
            let writer = self.transformer.transform_to(content.reader);
            futures::io::copy(writer, &mut file).await?;
            file.sync_all().await?;
        }

        Ok(())
    }
}

#[async_trait(?Send)]
impl ResourceStore for FileStore {
    async fn create(
        &self,
        metadata: &ResourceMetadata,
        content: Option<Variant>,
    ) -> Result<(), ResourceStoreError> {
        self.create_or_update(metadata, content, true).await
    }

    async fn update(
        &self,
        metadata: &ResourceMetadata,
        content: Option<Variant>,
    ) -> Result<(), ResourceStoreError> {
        self.create_or_update(metadata, content, false).await
    }

    async fn update_default_variant_from_slice(
        &self,
        id: &ResourceId,
        content: &[u8],
    ) -> Result<(), ResourceStoreError> {
        let content_path = self.variant_path(id, "default");
        let mut file = Self::create_file(&content_path).await?;
        futures::io::copy(
            self.transformer.transform_array_to(content).as_slice(),
            &mut file,
        )
        .await?;
        file.sync_all().await?;

        Ok(())
    }

    async fn delete(&self, id: &ResourceId) -> Result<(), ResourceStoreError> {
        // 1. get the metadata in order to know all the possible variants.
        let metadata = self.get_metadata(id).await?;

        // 2. remove the metadata.
        let meta_path = self.metadata_path(id);
        fs::remove_file(&meta_path).await?;

        // 3. remove variants.
        for variant in metadata.variants() {
            let path = self.variant_path(id, &variant.name());
            if Path::new(&path).exists().await {
                fs::remove_file(&path).await?;
            }
        }
        Ok(())
    }

    async fn delete_variant(
        &self,
        id: &ResourceId,
        variant: &str,
    ) -> Result<(), ResourceStoreError> {
        let path = self.variant_path(id, variant);
        if Path::new(&path).exists().await {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    async fn get_metadata(&self, id: &ResourceId) -> Result<ResourceMetadata, ResourceStoreError> {
        use async_std::io::ReadExt;

        let meta_path = self.metadata_path(id);

        let mut file = File::open(&meta_path)
            .await
            .map_err(|_| ResourceStoreError::NoSuchResource)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await?;
        let metadata: ResourceMetadata =
            ResourceMetadata::read_from_buffer(&self.transformer.transform_array_from(&buffer))?;

        Ok(metadata)
    }

    async fn get_full(
        &self,
        id: &ResourceId,
        name: &str,
    ) -> Result<(ResourceMetadata, BoxedReader), ResourceStoreError> {
        use async_std::io::ReadExt;

        let meta_path = self.metadata_path(id);

        let mut file = File::open(&meta_path)
            .await
            .map_err(|_| ResourceStoreError::NoSuchResource)?;
        let mut buffer = vec![];
        file.read_to_end(&mut buffer).await?;
        let metadata: ResourceMetadata =
            ResourceMetadata::read_from_buffer(&self.transformer.transform_array_from(&buffer))?;

        let content_path = self.variant_path(id, name);
        let file = File::open(&content_path)
            .await
            .map_err(|_| ResourceStoreError::NoSuchResource)?;

        Ok((
            metadata,
            self.transformer
                .transform_from(Box::new(BufReader::new(file))),
        ))
    }

    async fn get_variant(
        &self,
        id: &ResourceId,
        name: &str,
    ) -> Result<BoxedReader, ResourceStoreError> {
        let content_path = self.variant_path(id, name);

        let file = File::open(&content_path)
            .await
            .map_err(|_| ResourceStoreError::NoSuchResource)?;

        Ok(self
            .transformer
            .transform_from(Box::new(BufReader::new(file))))
    }
}
