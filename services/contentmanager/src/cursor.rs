/// Implementation of MetaDataCursor
use crate::generated::common::{MetadataCursorMethods, MetadataCursorNextResponder};
use common::traits::{SimpleObjectTracker, TrackerId};
use costaeres::common::ResourceMetadata;
use log::debug;

static BATCH_SIZE: usize = 25;

pub struct MetadataCursorImpl {
    id: TrackerId,
    data: Vec<ResourceMetadata>,
}

impl MetadataCursorImpl {
    pub fn new(id: TrackerId, data: Vec<ResourceMetadata>) -> Self {
        debug!("MetaDataCursorImpl::new id={} len={}", id, data.len());
        Self { id, data }
    }
}

impl SimpleObjectTracker for MetadataCursorImpl {
    fn id(&self) -> TrackerId {
        self.id
    }
}

impl MetadataCursorMethods for MetadataCursorImpl {
    /// Returns a batch of N objects, or reject.
    fn next(&mut self, responder: MetadataCursorNextResponder) {
        debug!("MetaDataCursorImpl::next len={}", self.data.len());
        if self.data.is_empty() {
            responder.reject();
        } else {
            let end = std::cmp::min(BATCH_SIZE, self.data.len());
            let result = self.data.drain(0..end).map(|item| item.into()).collect();
            responder.resolve(result);
        }
    }
}
