pub mod mem {
    use dashmap::DashMap;
    use std::sync::Arc;

    use crate::persistence::SqliteStore;

    #[derive(Clone)]
    pub struct MemoryStore<V: Clone + Send + Sync + 'static> {
        data: Arc<DashMap<String, V>>,
        db: Option<Arc<SqliteStore>>,
        table: String,
    }

    impl<V: Clone + Send + Sync + 'static> Default for MemoryStore<V> {
        fn default() -> Self {
            Self {
                data: Arc::new(DashMap::new()),
                db: None,
                table: String::new(),
            }
        }
    }

    impl<V: Clone + Send + Sync + 'static> MemoryStore<V> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn get(&self, key: &str) -> Option<V> {
            self.data.get(key).map(|v| v.value().clone())
        }

        pub fn remove(&self, key: &str) -> Option<V> {
            if let Some(ref db) = self.db {
                let _ = db.delete(&self.table, key);
            }
            self.data.remove(key).map(|(_, v)| v)
        }

        pub fn contains(&self, key: &str) -> bool {
            self.data.contains_key(key)
        }

        pub fn list(&self) -> Vec<(String, V)> {
            self.data
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect()
        }

        pub fn list_values(&self) -> Vec<V> {
            self.data
                .iter()
                .map(|entry| entry.value().clone())
                .collect()
        }

        pub fn len(&self) -> usize {
            self.data.len()
        }

        pub fn is_empty(&self) -> bool {
            self.data.is_empty()
        }
    }

    impl<V: Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static>
        MemoryStore<V>
    {
        /// Create optionally persistence-backed store.
        pub fn new_with_db(table: &str, db: &Option<Arc<SqliteStore>>) -> Self {
            match db {
                Some(db) => Self::with_persistence(table, db.clone()),
                None => Self::default(),
            }
        }

        /// Insert with write-through to SQLite if persistence is enabled.
        pub fn insert(&self, key: String, value: V) {
            if let Some(ref db) = self.db {
                if let Ok(json) = serde_json::to_string(&value) {
                    let _ = db.put(&self.table, &key, &json);
                }
            }
            self.data.insert(key, value);
        }

        /// Create a persistence-backed store, rehydrating from SQLite.
        pub fn with_persistence(table: &str, db: Arc<SqliteStore>) -> Self {
            let data = Arc::new(DashMap::new());
            if let Ok(rows) = db.list(table) {
                for (key, json) in rows {
                    if let Ok(val) = serde_json::from_str::<V>(&json) {
                        data.insert(key, val);
                    }
                }
            }
            Self {
                data,
                db: Some(db),
                table: table.to_string(),
            }
        }
    }
}
