/// Helper struct to manage observer lists for apis that
/// use a add_observer(some_type, F) / remove_observer(some_type, F) pattern.
use crate::traits::{DispatcherId, TrackerId};
use std::collections::HashMap;
use std::hash::Hash;

pub struct ServiceObserverTracker<K> {
    observers: HashMap<TrackerId, Vec<(K, DispatcherId)>>
}

impl<K> Default for ServiceObserverTracker<K> {
    fn default() -> Self {
        Self { observers: HashMap::new() }
    }
}

impl<K> ServiceObserverTracker<K>
where
    K: Eq + Hash,
{
    pub fn add(&mut self, observer: TrackerId, key: K, id: DispatcherId) {
        if let Some(observers) = self.observers.get_mut(&observer) {
            observers.push((key, id));
        } else {
            self.observers.insert(observer, vec![(key, id)]);
        }        
    }

    pub fn remove<P>(&mut self, observer: TrackerId, key: K, obt: &mut ObserverTracker<K, P>) -> bool
    {
        match self.observers.get_mut(&observer) {
            Some(observers) => {
                let mut i = 0;
                while i != observers.len() {
                    if observers[i].0 == key && obt.remove(&key, observers[i].1) {
                        observers.remove(i);
                    } else {
                        i += 1;
                    }
                }
                true
            }
            None => {
                false
            }
        }
    }

    pub fn clear<P>(&mut self, obt: &mut ObserverTracker<K, P>)
    {
        for observers in self.observers.values() {
            for observer in observers {
                obt.remove(&observer.0, observer.1);
            }
        }

        self.observers.clear();
    }
}

pub struct ObserverTracker<K, P> {
    id: DispatcherId,
    observers: HashMap<K, Vec<(P, DispatcherId)>>,
}

impl<K, P> Default for ObserverTracker<K, P> {
    fn default() -> Self {
        Self {
            id: 0,
            observers: HashMap::new(),
        }
    }
}

impl<K, P> ObserverTracker<K, P>
where
    K: Eq + Hash,
{
    // Return the total number of observer entries.
    pub fn count(&self) -> usize {
        self.observers.values().map(|vec| vec.len()).sum()
    }

    // Return the number of observer keys.
    pub fn key_count(&self) -> usize {
        self.observers.len()
    }

    // Returns a new DispatcherId that uniquely identifies this (key, item) tuple.
    pub fn add(&mut self, key: K, item: P) -> DispatcherId {
        self.id += 1;

        if let Some(observers) = self.observers.get_mut(&key) {
            observers.push((item, self.id));
        } else {
            self.observers.insert(key, vec![(item, self.id)]);
        }

        self.id
    }

    // Returns true if an observer for the given parameters was actually removed.
    pub fn remove(&mut self, key: &K, id: DispatcherId) -> bool {
        let mut removed = false;
        let mut remove_by_key = false;

        if let Some(items) = self.observers.get_mut(key) {
            // Remove the vector items that have the matching id.
            // Note: Once it's in stable Rustc, we could simply use:
            // entry.drain_filter(|item| item.1 == id);
            let mut i = 0;
            while i != items.len() {
                if items[i].1 == id {
                    items.remove(i);
                    removed = true;
                } else {
                    i += 1;
                }
            }

            remove_by_key = items.is_empty();
        }

        // Check if the vector for this key is now empty and remove it if so.
        if remove_by_key {
            self.observers.remove(key);
        }

        removed
    }

    // Calls the function parameter for each list matching this key.
    pub fn for_each<F>(&mut self, key: &K, mut f: F)
    where
        F: FnMut(&mut P, DispatcherId),
    {
        if let Some(observers) = self.observers.get_mut(key) {
            for observer in observers {
                f(&mut observer.0, observer.1)
            }
        }
    }
}
