use std::{collections::HashMap, error::Error, fmt};

/// Error returned when a key is missing from a [`MemoryStore`] namespace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryStoreError {
    namespace: String,
    key: String,
}

impl MemoryStoreError {
    /// Create a missing-key error for a namespace/key pair.
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
        }
    }

    /// Namespace searched by the failed lookup.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Key searched by the failed lookup.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl fmt::Display for MemoryStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "key '{}' not found in namespace '{}'",
            self.key, self.namespace
        )
    }
}

impl Error for MemoryStoreError {}

/// Simple namespace-aware in-memory store compatible with upstream concepts.
#[derive(Clone, Debug)]
pub struct MemoryStore<T> {
    namespace: String,
    values: HashMap<String, HashMap<String, T>>,
}

impl<T> Default for MemoryStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> MemoryStore<T> {
    /// Create an empty memory store.
    pub fn new() -> Self {
        Self {
            namespace: String::new(),
            values: HashMap::new(),
        }
    }

    /// Switch the active namespace, creating it if needed.
    pub fn namespace(&mut self, namespace: impl Into<String>) {
        self.namespace = namespace.into();
        self.values.entry(self.namespace.clone()).or_default();
    }

    /// Return the active namespace.
    pub fn current_namespace(&self) -> &str {
        &self.namespace
    }

    /// Get a value by key from the active namespace.
    pub fn get(&self, key: impl AsRef<str>) -> Result<&T, MemoryStoreError> {
        let key = key.as_ref();
        self.values
            .get(&self.namespace)
            .and_then(|namespace| namespace.get(key))
            .ok_or_else(|| MemoryStoreError::new(self.namespace.clone(), key))
    }

    /// Set a value in the active namespace and return a shared reference to it.
    pub fn set(&mut self, key: impl Into<String>, value: T) -> &T {
        let key = key.into();
        self.values
            .entry(self.namespace.clone())
            .or_default()
            .entry(key)
            .insert_entry(value)
            .into_mut()
    }

    /// Remove all namespaces and values.
    pub fn close(&mut self) {
        self.values.clear();
    }

    /// Return true when all namespaces are empty.
    pub fn is_empty(&self) -> bool {
        self.values.values().all(HashMap::is_empty)
    }

    /// Return the total number of stored values across namespaces.
    pub fn len(&self) -> usize {
        self.values.values().map(HashMap::len).sum()
    }
}
