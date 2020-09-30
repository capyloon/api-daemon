// Objects instanciated needs to be tracked to allow
// method calls accross the transport layer to refer to a stable ID.

use crate::traits::*;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct ObjectTracker<T, K>
where
    K: Hash + Eq + ObjectTrackerKey,
{
    objects: HashMap<K, T>,
    current_id: K,
}

impl<T, K> Default for ObjectTracker<T, K>
where
    K: Hash + Eq + ObjectTrackerKey,
{
    fn default() -> Self {
        ObjectTracker {
            objects: HashMap::new(),
            current_id: K::first(),
        }
    }
}

impl<T, K> ObjectTrackerMethods<T, K> for ObjectTracker<T, K>
where
    K: Hash + Eq + ObjectTrackerKey,
{
    fn next_id(&self) -> K {
        self.current_id
    }

    fn track(&mut self, obj: T) -> K {
        let current = self.current_id;
        self.objects.insert(current, obj);
        self.current_id = self.current_id.next();
        current
    }

    fn untrack(&mut self, id: K) -> bool {
        self.objects.remove(&id).is_some()
    }

    fn get(&self, id: K) -> Option<&T> {
        self.objects.get(&id)
    }

    fn get_mut(&mut self, id: K) -> Option<&mut T> {
        self.objects.get_mut(&id)
    }

    fn clear(&mut self) {
        self.objects.clear();
    }

    fn track_with(&mut self, obj: T, key: K) {
        self.objects.insert(key, obj);
    }
}

#[test]
fn test_object_tracker() {
    #[derive(Debug, PartialEq)]
    pub struct Type1(u32);

    #[derive(Debug, PartialEq)]
    pub struct Type2(String);

    #[derive(Debug, PartialEq)]
    pub struct DroppedType {
        data: u32,
    }

    impl DroppedType {
        fn new() -> Self {
            DroppedType { data: 42 }
        }
    }

    #[derive(Debug, PartialEq)]
    pub enum TrackableObjects {
        TType1(Type1),
        TType2(Type2),
        TDropped(DroppedType),
    }

    let mut tracker = ObjectTracker::<TrackableObjects, TrackerId>::default();

    let o1 = Type1(2);
    let o2 = Type2("track me".to_owned());

    let id1 = tracker.track(TrackableObjects::TType1(o1));
    let id2 = tracker.track(TrackableObjects::TType2(o2));

    match *tracker.get(id1).unwrap() {
        TrackableObjects::TType1(ref val) => {
            assert_eq!(val.0, 2);
        }
        _ => panic!("Wrong type!"),
    }

    tracker.untrack(id1);
    assert_eq!(tracker.get(id1), None);

    match *tracker.get(id2).unwrap() {
        TrackableObjects::TType2(ref val) => {
            assert_eq!(val.0, "track me".to_owned());
        }
        _ => panic!("Wrong type!"),
    }

    {
        let o3 = DroppedType::new();
        let id3 = tracker.track(TrackableObjects::TDropped(o3));
        assert_eq!(id3, 3);
    }

    assert_eq!(tracker.get(3 /* id3 */).is_none(), false);
}
