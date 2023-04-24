/// Various utilities to convert data types.
use crate::generated::common::*;
use chrono::{DateTime, Utc};
use costaeres::common::{ResourceKind as ResourceKindC, ResourceMetadata, VariantMetadata};

impl From<VisitPriority> for costaeres::scorer::VisitPriority {
    fn from(val: VisitPriority) -> Self {
        match val {
            VisitPriority::Normal => Self::Normal,
            VisitPriority::High => Self::High,
            VisitPriority::VeryHigh => Self::VeryHigh,
        }
    }
}

impl From<ResourceKindC> for ResourceKind {
    fn from(val: ResourceKindC) -> Self {
        match val {
            ResourceKindC::Container => ResourceKind::Container,
            ResourceKindC::Leaf => ResourceKind::Leaf,
        }
    }
}

impl From<ResourceKind> for ResourceKindC {
    fn from(val: ResourceKind) -> Self {
        match val {
            ResourceKind::Container => ResourceKindC::Container,
            ResourceKind::Leaf => ResourceKindC::Leaf,
        }
    }
}

impl From<&VariantMetadata> for Variant {
    fn from(val: &VariantMetadata) -> Self {
        Self {
            name: val.name(),
            mime_type: val.mime_type(),
            size: val.size() as _,
        }
    }
}

impl From<&Variant> for VariantMetadata {
    fn from(val: &Variant) -> Self {
        VariantMetadata::new(&val.name, &val.mime_type, val.size as _)
    }
}

// Using free-floating functions for time conversions because we can't implement
// From<> or Into<> on foreign types.
fn chrono_to_system_time(from: DateTime<Utc>) -> common::SystemTime {
    use std::time::{Duration, UNIX_EPOCH};

    let time = UNIX_EPOCH
        .checked_add(Duration::from_nanos(from.timestamp_nanos() as _))
        .unwrap();
    common::SystemTime::from(time)
}

fn system_time_to_chrono(from: common::SystemTime) -> DateTime<Utc> {
    use chrono::TimeZone;

    let nanos = from
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    Utc.timestamp_nanos(nanos as _)
}

impl From<ResourceMetadata> for Metadata {
    fn from(val: ResourceMetadata) -> Self {
        let variants: Vec<Variant> = val.variants().iter().map(|item| item.into()).collect();
        Self {
            id: val.id().into(),
            parent: val.parent().into(),
            name: val.name(),
            tags: val.tags().clone(),
            variants,
            kind: val.kind().into(),
            created: chrono_to_system_time(val.created().into()),
            modified: chrono_to_system_time(val.modified().into()),
        }
    }
}

impl From<Metadata> for ResourceMetadata {
    fn from(val: Metadata) -> Self {
        let variants: Vec<VariantMetadata> = val.variants.iter().map(|item| item.into()).collect();

        let mut meta = ResourceMetadata::new(
            &val.id.into(),
            &val.parent.into(),
            val.kind.into(),
            &val.name,
            val.tags,
            variants,
        );

        meta.set_created(system_time_to_chrono(val.created).into());
        meta.set_modified(system_time_to_chrono(val.modified).into());

        meta
    }
}

#[test]
fn time_conversion() {
    let now = Utc::now();

    let system_now = chrono_to_system_time(now);
    let now2 = system_time_to_chrono(system_now);

    assert_eq!(now, now2);
}
