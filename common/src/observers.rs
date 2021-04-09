/// Helper struct to manage observer lists for apis that
/// use a add_observer(some_type, F) / remove_observer(some_type, F) pattern.
use crate::traits::DispatcherId;
use std::collections::HashMap;
use std::hash::Hash;

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

        match self.observers.get_mut(&key) {
            Some(observers) => {
                observers.push((item, self.id));
            }
            None => {
                let init = vec![(item, self.id)];
                self.observers.insert(key, init);
            }
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
