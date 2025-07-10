use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::fs;
use rocksdb::{DB, Options, ColumnFamily, ColumnFamilyDescriptor, WriteBatch, WriteOptions};
use tiny_keccak::{Hasher, Keccak};
use serde::{Serialize, Deserialize};

// Error types
#[derive(Debug)]
#[allow(dead_code)]
pub enum MerkleTreeError {
    RocksDB(rocksdb::Error),
    InvalidArgument(String),
    IllegalState(String),
    IO(std::io::Error),
    Serialization(String),
}

impl From<rocksdb::Error> for MerkleTreeError {
    fn from(err: rocksdb::Error) -> Self {
        MerkleTreeError::RocksDB(err)
    }
}

impl From<std::io::Error> for MerkleTreeError {
    fn from(err: std::io::Error) -> Self {
        MerkleTreeError::IO(err)
    }
}

type Result<T> = std::result::Result<T, MerkleTreeError>;

// Constants
const HASH_LENGTH: usize = 32;
const METADATA_CF_NAME: &str = "metaData";
const NODES_CF_NAME: &str = "nodes";
const KEY_DATA_CF_NAME: &str = "keyData";

// Metadata keys
const KEY_ROOT_HASH: &str = "rootHash";
const KEY_NUM_LEAVES: &str = "numLeaves";
const KEY_DEPTH: &str = "depth";
const KEY_HANGING_NODE_PREFIX: &str = "hangingNode";

// Utility wrapper for byte arrays to use as HashMap keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ByteArrayWrapper(Vec<u8>);

impl ByteArrayWrapper {
    fn new(data: Vec<u8>) -> Self {
        ByteArrayWrapper(data)
    }
    
    fn data(&self) -> &[u8] {
        &self.0
    }
}

// Node structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    hash: Vec<u8>,
    left: Option<Vec<u8>>,
    right: Option<Vec<u8>>,
    parent: Option<Vec<u8>>,
    node_hash_to_remove_from_db: Option<Vec<u8>>,
}

#[allow(dead_code)]
impl Node {
    // Construct a leaf node with a known hash
    pub fn new_leaf(hash: Vec<u8>) -> Result<Self> {
        if hash.is_empty() {
            return Err(MerkleTreeError::InvalidArgument("Node hash cannot be empty".to_string()));
        }
        
        Ok(Node {
            hash,
            left: None,
            right: None,
            parent: None,
            node_hash_to_remove_from_db: None,
        })
    }
    
    // Construct a node with all fields
    pub fn new_with_fields(
        hash: Vec<u8>,
        left: Option<Vec<u8>>,
        right: Option<Vec<u8>>,
        parent: Option<Vec<u8>>,
    ) -> Result<Self> {
        if hash.is_empty() {
            return Err(MerkleTreeError::InvalidArgument("Node hash cannot be empty".to_string()));
        }
        
        Ok(Node {
            hash,
            left,
            right,
            parent,
            node_hash_to_remove_from_db: None,
        })
    }
    
    // Construct a node (non-leaf) with left and right hashes, auto-calculate node hash
    pub fn new_internal(left: Option<Vec<u8>>, right: Option<Vec<u8>>) -> Result<Self> {
        if left.is_none() && right.is_none() {
            return Err(MerkleTreeError::InvalidArgument(
                "At least one of left or right hash must be non-null".to_string()
            ));
        }
        
        let hash = Self::calculate_hash_static(&left, &right)?;
        
        Ok(Node {
            hash,
            left,
            right,
            parent: None,
            node_hash_to_remove_from_db: None,
        })
    }
    
    // Calculate hash based on left and right child hashes
    fn calculate_hash_static(left: &Option<Vec<u8>>, right: &Option<Vec<u8>>) -> Result<Vec<u8>> {
        if left.is_none() && right.is_none() {
            return Err(MerkleTreeError::InvalidArgument("Cannot calculate hash with no children".to_string()));
        }
        
        let left_hash = left.as_ref().unwrap_or_else(|| right.as_ref().unwrap());
        let right_hash = right.as_ref().unwrap_or_else(|| left.as_ref().unwrap());
        
        Ok(keccak_256_two_inputs(left_hash, right_hash))
    }
    
    pub fn calculate_hash(&self) -> Result<Vec<u8>> {
        Self::calculate_hash_static(&self.left, &self.right)
    }
    
    // Encode the node into bytes for storage
    pub fn encode(&self) -> Vec<u8> {
        let has_left = self.left.is_some();
        let has_right = self.right.is_some();
        let has_parent = self.parent.is_some();
        
        let mut encoded = Vec::new();
        
        // Add hash
        encoded.extend_from_slice(&self.hash);
        
        // Add flags
        encoded.push(if has_left { 1 } else { 0 });
        encoded.push(if has_right { 1 } else { 0 });
        encoded.push(if has_parent { 1 } else { 0 });
        
        // Add optional fields
        if let Some(ref left) = self.left {
            encoded.extend_from_slice(left);
        }
        if let Some(ref right) = self.right {
            encoded.extend_from_slice(right);
        }
        if let Some(ref parent) = self.parent {
            encoded.extend_from_slice(parent);
        }
        
        encoded
    }
    
    // Decode a node from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < HASH_LENGTH + 3 {
            return Err(MerkleTreeError::InvalidArgument("Invalid encoded data length".to_string()));
        }
        
        let mut offset = 0;
        
        // Read hash
        let hash = data[offset..offset + HASH_LENGTH].to_vec();
        offset += HASH_LENGTH;
        
        // Read flags
        let has_left = data[offset] == 1;
        let has_right = data[offset + 1] == 1;
        let has_parent = data[offset + 2] == 1;
        offset += 3;
        
        // Read optional fields
        let left = if has_left {
            let left_hash = data[offset..offset + HASH_LENGTH].to_vec();
            offset += HASH_LENGTH;
            Some(left_hash)
        } else {
            None
        };
        
        let right = if has_right {
            let right_hash = data[offset..offset + HASH_LENGTH].to_vec();
            offset += HASH_LENGTH;
            Some(right_hash)
        } else {
            None
        };
        
        let parent = if has_parent {
            let parent_hash = data[offset..offset + HASH_LENGTH].to_vec();
            Some(parent_hash)
        } else {
            None
        };
        
        Ok(Node {
            hash,
            left,
            right,
            parent,
            node_hash_to_remove_from_db: None,
        })
    }
    
    pub fn set_parent_node_hash(&mut self, parent_hash: Vec<u8>) {
        self.parent = Some(parent_hash);
    }
    
    pub fn update_leaf(&mut self, old_leaf_hash: &[u8], new_leaf_hash: Vec<u8>) -> Result<()> {
        if let Some(ref left) = self.left {
            if left == old_leaf_hash {
                self.left = Some(new_leaf_hash);
                return Ok(());
            }
        }
        
        if let Some(ref right) = self.right {
            if right == old_leaf_hash {
                self.right = Some(new_leaf_hash);
                return Ok(());
            }
        }
        
        Err(MerkleTreeError::InvalidArgument(
            "Old hash not found among this node's children".to_string()
        ))
    }
    
    pub fn add_leaf(&mut self, leaf_hash: Vec<u8>) -> Result<()> {
        if self.left.is_none() {
            self.left = Some(leaf_hash);
        } else if self.right.is_none() {
            self.right = Some(leaf_hash);
        } else {
            return Err(MerkleTreeError::InvalidArgument(
                "Node already has both left and right children".to_string()
            ));
        }
        Ok(())
    }
}

// Global registry of open trees
lazy_static::lazy_static! {
    static ref OPEN_TREES: Mutex<HashMap<String, Arc<MerkleTree>>> = Mutex::new(HashMap::new());
}

// Main MerkleTree structure
#[allow(dead_code)]
pub struct MerkleTree {
    tree_name: String,
    path: String,
    db: Arc<DB>,
    
    // Caches
    nodes_cache: RwLock<HashMap<ByteArrayWrapper, Node>>,
    hanging_nodes: RwLock<HashMap<i32, Vec<u8>>>,
    key_data_cache: RwLock<HashMap<ByteArrayWrapper, Vec<u8>>>,
    
    // Metadata
    num_leaves: RwLock<i32>,
    depth: RwLock<i32>,
    root_hash: RwLock<Option<Vec<u8>>>,
    
    // State
    closed: RwLock<bool>,
    has_unsaved_changes: RwLock<bool>,
}

#[allow(dead_code)]
impl MerkleTree {
    pub fn new(tree_name: String) -> Result<Arc<Self>> {
        // Check if tree is already open
        {
            let open_trees = OPEN_TREES.lock().unwrap();
            if open_trees.contains_key(&tree_name) {
                return Err(MerkleTreeError::IllegalState(
                    "There is already an open instance of this tree".to_string()
                ));
            }
        }
        
        // Create directory
        let path = format!("merkleTree/{}", tree_name);
        fs::create_dir_all(&path)?;
        
        // Initialize database
        let db = Self::initialize_db(&path)?;
        
        let tree = Arc::new(MerkleTree {
            tree_name: tree_name.clone(),
            path,
            db,
            nodes_cache: RwLock::new(HashMap::new()),
            hanging_nodes: RwLock::new(HashMap::new()),
            key_data_cache: RwLock::new(HashMap::new()),
            num_leaves: RwLock::new(0),
            depth: RwLock::new(0),
            root_hash: RwLock::new(None),
            closed: RwLock::new(false),
            has_unsaved_changes: RwLock::new(false),
        });
        
        // Load metadata
        tree.load_metadata()?;
        
        // Register instance
        {
            let mut open_trees = OPEN_TREES.lock().unwrap();
            open_trees.insert(tree_name, tree.clone());
        }
        
        // Compact on startup
        let _ = tree.db.compact_range::<&[u8], &[u8]>(None, None);
        
        Ok(tree)
    }
    
    fn initialize_db(path: &str) -> Result<Arc<DB>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(100);
        opts.set_max_background_jobs(1);
        
        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new("default", Options::default()),
            ColumnFamilyDescriptor::new(METADATA_CF_NAME, Options::default()),
            ColumnFamilyDescriptor::new(NODES_CF_NAME, Options::default()),
            ColumnFamilyDescriptor::new(KEY_DATA_CF_NAME, Options::default()),
        ];
        
        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;
        Ok(Arc::new(db))
    }
    
    fn get_cf_handle(&self, name: &str) -> Result<&ColumnFamily> {
        self.db.cf_handle(name).ok_or_else(|| {
            MerkleTreeError::IllegalState(format!("Column family '{}' not found", name))
        })
    }
    
    fn load_metadata(&self) -> Result<()> {
        let metadata_cf = self.get_cf_handle(METADATA_CF_NAME)?;
        
        // Load root hash
        if let Some(root_hash_bytes) = self.db.get_cf(metadata_cf, KEY_ROOT_HASH)? {
            *self.root_hash.write().unwrap() = Some(root_hash_bytes);
        }
        
        // Load num leaves
        if let Some(num_leaves_bytes) = self.db.get_cf(metadata_cf, KEY_NUM_LEAVES)? {
            let num_leaves = i32::from_le_bytes(
                num_leaves_bytes.try_into().map_err(|_| {
                    MerkleTreeError::InvalidArgument("Invalid num_leaves format".to_string())
                })?
            );
            *self.num_leaves.write().unwrap() = num_leaves;
        }
        
        // Load depth
        if let Some(depth_bytes) = self.db.get_cf(metadata_cf, KEY_DEPTH)? {
            let depth = i32::from_le_bytes(
                depth_bytes.try_into().map_err(|_| {
                    MerkleTreeError::InvalidArgument("Invalid depth format".to_string())
                })?
            );
            *self.depth.write().unwrap() = depth;
        }
        
        // Load hanging nodes
        let depth = *self.depth.read().unwrap();
        let mut hanging_nodes = self.hanging_nodes.write().unwrap();
        hanging_nodes.clear();
        
        for i in 0..=depth {
            let key = format!("{}{}", KEY_HANGING_NODE_PREFIX, i);
            if let Some(hash) = self.db.get_cf(metadata_cf, key)? {
                hanging_nodes.insert(i, hash);
            }
        }
        
        Ok(())
    }
    
    fn error_if_closed(&self) -> Result<()> {
        if *self.closed.read().unwrap() {
            return Err(MerkleTreeError::IllegalState("MerkleTree is closed".to_string()));
        }
        Ok(())
    }
    
    pub fn get_root_hash(&self) -> Result<Option<Vec<u8>>> {
        self.error_if_closed()?;
        let root_hash = self.root_hash.read().unwrap();
        Ok(root_hash.clone())
    }
    
    pub fn get_num_leaves(&self) -> Result<i32> {
        self.error_if_closed()?;
        Ok(*self.num_leaves.read().unwrap())
    }
    
    pub fn get_depth(&self) -> Result<i32> {
        self.error_if_closed()?;
        Ok(*self.depth.read().unwrap())
    }
    
    pub fn get_data(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.error_if_closed()?;
        
        // Check cache first
        {
            let cache = self.key_data_cache.read().unwrap();
            if let Some(data) = cache.get(&ByteArrayWrapper::new(key.to_vec())) {
                return Ok(Some(data.clone()));
            }
        }
        
        // Check database
        let key_data_cf = self.get_cf_handle(KEY_DATA_CF_NAME)?;
        Ok(self.db.get_cf(key_data_cf, key)?)
    }
    
    pub fn add_or_update_data(&self, key: &[u8], data: &[u8]) -> Result<()> {
        self.error_if_closed()?;
        
        if key.is_empty() {
            return Err(MerkleTreeError::InvalidArgument("Key cannot be empty".to_string()));
        }
        if data.is_empty() {
            return Err(MerkleTreeError::InvalidArgument("Data cannot be empty".to_string()));
        }
        
        let existing_data = self.get_data(key)?;
        let old_leaf_hash = if let Some(ref existing) = existing_data {
            Some(calculate_leaf_hash(key, existing))
        } else {
            None
        };
        
        let new_leaf_hash = calculate_leaf_hash(key, data);
        
        if let Some(ref old_hash) = old_leaf_hash {
            if old_hash == &new_leaf_hash {
                return Ok(());
            }
        }
        
        // Store key-data mapping in cache
        {
            let mut cache = self.key_data_cache.write().unwrap();
            cache.insert(ByteArrayWrapper::new(key.to_vec()), data.to_vec());
        }
        *self.has_unsaved_changes.write().unwrap() = true;
        
        if old_leaf_hash.is_none() {
            // Add new leaf
            let leaf_node = Node::new_leaf(new_leaf_hash)?;
            self.add_leaf(leaf_node)?;
        } else {
            // Update existing leaf
            self.update_leaf(&old_leaf_hash.unwrap(), new_leaf_hash)?;
        }
        
        Ok(())
    }
    
    fn add_leaf(&self, leaf_node: Node) -> Result<()> {
        let mut num_leaves = self.num_leaves.write().unwrap();
        
        if *num_leaves == 0 {
            // First leaf becomes root and hanging at level 0
            {
                let mut hanging_nodes = self.hanging_nodes.write().unwrap();
                hanging_nodes.insert(0, leaf_node.hash.clone());
            }
            *self.root_hash.write().unwrap() = Some(leaf_node.hash.clone());
            *num_leaves += 1;
            self.update_node_in_cache(leaf_node);
            return Ok(());
        }
        
        // Check if there's a hanging leaf at level 0
        let hanging_leaf_hash = {
            let hanging_nodes = self.hanging_nodes.read().unwrap();
            hanging_nodes.get(&0).cloned()
        };
        
        if let Some(hanging_hash) = hanging_leaf_hash {
            // Get the hanging leaf node
            let hanging_leaf = self.get_node_by_hash(&hanging_hash)?;
            
            if let Some(hanging_leaf) = hanging_leaf {
                // Remove from hanging nodes at level 0
                {
                    let mut hanging_nodes = self.hanging_nodes.write().unwrap();
                    hanging_nodes.remove(&0);
                }
                
                if hanging_leaf.parent.is_none() {
                    // Hanging leaf is the root - create parent with both leaves
                    let parent_node = Node::new_internal(Some(hanging_hash.clone()), Some(leaf_node.hash.clone()))?;
                    
                    // Update parent references for both leaves
                    let mut hanging_leaf_mut = hanging_leaf.clone();
                    hanging_leaf_mut.set_parent_node_hash(parent_node.hash.clone());
                    self.update_node_in_cache(hanging_leaf_mut);
                    
                    let mut leaf_node_mut = leaf_node.clone();
                    leaf_node_mut.set_parent_node_hash(parent_node.hash.clone());
                    self.update_node_in_cache(leaf_node_mut);
                    
                    // Add parent node at level 1
                    self.add_node(1, parent_node)?;
                } else {
                    // Hanging leaf has a parent - add new leaf to that parent
                    if let Some(parent_hash) = &hanging_leaf.parent {
                        let mut parent_node = self.get_node_by_hash(parent_hash)?
                            .ok_or_else(|| MerkleTreeError::IllegalState("Parent node not found".to_string()))?;
                        
                        parent_node.add_leaf(leaf_node.hash.clone())?;
                        
                        // Update new leaf's parent reference
                        let mut leaf_node_mut = leaf_node.clone();
                        leaf_node_mut.set_parent_node_hash(parent_hash.clone());
                        self.update_node_in_cache(leaf_node_mut);
                        
                        // Recalculate parent hash and update
                        let new_parent_hash = parent_node.calculate_hash()?;
                        self.update_node_hash(&mut parent_node, new_parent_hash)?;
                    }
                }
            }
        } else {
            // No hanging leaf at level 0 - make this leaf hanging
            let mut hanging_nodes = self.hanging_nodes.write().unwrap();
            hanging_nodes.insert(0, leaf_node.hash.clone());
            
            // Create a parent node with just this leaf and add it to level 1
            let parent_node = Node::new_internal(Some(leaf_node.hash.clone()), None)?;
            let mut leaf_node_mut = leaf_node.clone();
            leaf_node_mut.set_parent_node_hash(parent_node.hash.clone());
            self.update_node_in_cache(leaf_node_mut);
            
            drop(hanging_nodes); // Release the lock before recursive call
            self.add_node(1, parent_node)?;
        }
        
        *num_leaves += 1;
        self.update_node_in_cache(leaf_node);
        Ok(())
    }
    
    fn add_node(&self, level: i32, node: Node) -> Result<()> {
        // Update depth if necessary
        {
            let mut depth = self.depth.write().unwrap();
            if level > *depth {
                *depth = level;
            }
        }
        
        // Get hanging node at this level
        let hanging_node_hash = {
            let hanging_nodes = self.hanging_nodes.read().unwrap();
            hanging_nodes.get(&level).cloned()
        };
        
        if let Some(hanging_hash) = hanging_node_hash {
            // There's a hanging node at this level
            let hanging_node = self.get_node_by_hash(&hanging_hash)?;
            
            if let Some(hanging_node) = hanging_node {
                // Remove hanging node from this level
                {
                    let mut hanging_nodes = self.hanging_nodes.write().unwrap();
                    hanging_nodes.remove(&level);
                }
                
                if hanging_node.parent.is_none() {
                    // Hanging node is a root - create parent with both nodes
                    let parent = Node::new_internal(Some(hanging_hash.clone()), Some(node.hash.clone()))?;
                    
                    // Update parent references
                    let mut hanging_node_mut = hanging_node.clone();
                    hanging_node_mut.set_parent_node_hash(parent.hash.clone());
                    self.update_node_in_cache(hanging_node_mut);
                    
                    let mut node_mut = node.clone();
                    node_mut.set_parent_node_hash(parent.hash.clone());
                    self.update_node_in_cache(node_mut);
                    
                    // Recursively add parent at next level
                    self.add_node(level + 1, parent)?;
                } else {
                    // Hanging node has a parent - add new node to that parent
                    if let Some(parent_hash) = &hanging_node.parent {
                        let mut parent_node = self.get_node_by_hash(parent_hash)?
                            .ok_or_else(|| MerkleTreeError::IllegalState("Parent node not found".to_string()))?;
                        
                        parent_node.add_leaf(node.hash.clone())?;
                        
                        // Update new node's parent reference
                        let mut node_mut = node.clone();
                        node_mut.set_parent_node_hash(parent_hash.clone());
                        self.update_node_in_cache(node_mut);
                        
                        // Recalculate parent hash and update
                        let new_parent_hash = parent_node.calculate_hash()?;
                        self.update_node_hash(&mut parent_node, new_parent_hash)?;
                    }
                }
            }
        } else {
            // No hanging node at this level - make this node hanging
            {
                let mut hanging_nodes = self.hanging_nodes.write().unwrap();
                hanging_nodes.insert(level, node.hash.clone());
            }
            
            // If this is at or above the current depth, it becomes the new root
            let current_depth = *self.depth.read().unwrap();
            if level >= current_depth {
                *self.root_hash.write().unwrap() = Some(node.hash.clone());
            } else {
                // Create a parent node and continue up
                let parent_node = Node::new_internal(Some(node.hash.clone()), None)?;
                let mut node_mut = node.clone();
                node_mut.set_parent_node_hash(parent_node.hash.clone());
                self.update_node_in_cache(node_mut);
                
                self.add_node(level + 1, parent_node)?;
            }
        }
        
        self.update_node_in_cache(node);
        Ok(())
    }
    
    fn update_leaf(&self, old_leaf_hash: &[u8], new_leaf_hash: Vec<u8>) -> Result<()> {
        if old_leaf_hash == new_leaf_hash {
            return Err(MerkleTreeError::InvalidArgument(
                "Old and new leaf hashes cannot be the same".to_string()
            ));
        }
        
        let mut leaf = self.get_node_by_hash(old_leaf_hash)?
            .ok_or_else(|| MerkleTreeError::InvalidArgument("Leaf not found".to_string()))?;
        
        self.update_node_hash(&mut leaf, new_leaf_hash)?;
        Ok(())
    }
    
    fn update_node_hash(&self, node: &mut Node, new_hash: Vec<u8>) -> Result<()> {
        if node.node_hash_to_remove_from_db.is_none() {
            node.node_hash_to_remove_from_db = Some(node.hash.clone());
        }
        
        let old_hash = node.hash.clone();
        node.hash = new_hash.clone();
        
        // Update hanging nodes
        {
            let mut hanging_nodes = self.hanging_nodes.write().unwrap();
            for (_level, hash) in hanging_nodes.iter_mut() {
                if hash == &old_hash {
                    *hash = new_hash.clone();
                    break;
                }
            }
        }
        
        // Update cache
        {
            let mut cache = self.nodes_cache.write().unwrap();
            cache.remove(&ByteArrayWrapper::new(old_hash.clone()));
            cache.insert(ByteArrayWrapper::new(new_hash.clone()), node.clone());
        }
        
        // Handle different node types
        let is_leaf = node.left.is_none() && node.right.is_none();
        let is_root = node.parent.is_none();
        
        // If this is the root node, update the root hash
        if is_root {
            *self.root_hash.write().unwrap() = Some(new_hash.clone());
            
            // Update children's parent references
            if let Some(ref left_hash) = node.left {
                if let Some(mut left_node) = self.get_node_by_hash(left_hash)? {
                    left_node.set_parent_node_hash(new_hash.clone());
                    self.update_node_in_cache(left_node);
                }
            }
            
            if let Some(ref right_hash) = node.right {
                if let Some(mut right_node) = self.get_node_by_hash(right_hash)? {
                    right_node.set_parent_node_hash(new_hash.clone());
                    self.update_node_in_cache(right_node);
                }
            }
        }
        
        // If this is a leaf node with a parent, update the parent
        if is_leaf && !is_root {
            if let Some(ref parent_hash) = node.parent {
                if let Some(mut parent_node) = self.get_node_by_hash(parent_hash)? {
                    parent_node.update_leaf(&old_hash, new_hash.clone())?;
                    let new_parent_hash = parent_node.calculate_hash()?;
                    self.update_node_hash(&mut parent_node, new_parent_hash)?;
                }
            }
        }
        // If this is an internal node with a parent, update the parent and children
        else if !is_leaf && !is_root {
            // Update children's parent references
            if let Some(ref left_hash) = node.left {
                if let Some(mut left_node) = self.get_node_by_hash(left_hash)? {
                    left_node.set_parent_node_hash(new_hash.clone());
                    self.update_node_in_cache(left_node);
                }
            }
            if let Some(ref right_hash) = node.right {
                if let Some(mut right_node) = self.get_node_by_hash(right_hash)? {
                    right_node.set_parent_node_hash(new_hash.clone());
                    self.update_node_in_cache(right_node);
                }
            }
            
            // Update parent
            if let Some(ref parent_hash) = node.parent {
                if let Some(mut parent_node) = self.get_node_by_hash(parent_hash)? {
                    parent_node.update_leaf(&old_hash, new_hash.clone())?;
                    let new_parent_hash = parent_node.calculate_hash()?;
                    self.update_node_hash(&mut parent_node, new_parent_hash)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn get_node_by_hash(&self, hash: &[u8]) -> Result<Option<Node>> {
        if hash.is_empty() {
            return Ok(None);
        }
        
        // Check cache first
        {
            let cache = self.nodes_cache.read().unwrap();
            if let Some(node) = cache.get(&ByteArrayWrapper::new(hash.to_vec())) {
                return Ok(Some(node.clone()));
            }
        }
        
        // Check database
        let nodes_cf = self.get_cf_handle(NODES_CF_NAME)?;
        if let Some(encoded_data) = self.db.get_cf(nodes_cf, hash)? {
            let node = Node::decode(&encoded_data)?;
            
            // Add to cache
            {
                let mut cache = self.nodes_cache.write().unwrap();
                cache.insert(ByteArrayWrapper::new(hash.to_vec()), node.clone());
            }
            
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }
    
    fn update_node_in_cache(&self, node: Node) {
        let mut cache = self.nodes_cache.write().unwrap();
        cache.insert(ByteArrayWrapper::new(node.hash.clone()), node);
    }
    
    pub fn flush_to_disk(&self) -> Result<()> {
        if !*self.has_unsaved_changes.read().unwrap() {
            return Ok(());
        }
        
        self.error_if_closed()?;
        
        let metadata_cf = self.get_cf_handle(METADATA_CF_NAME)?;
        let nodes_cf = self.get_cf_handle(NODES_CF_NAME)?;
        let key_data_cf = self.get_cf_handle(KEY_DATA_CF_NAME)?;
        
        let mut batch = WriteBatch::default();
        
        // Clear old metadata
        // Note: In production, you might want to implement a more efficient metadata clearing strategy
        
        // Write metadata
        if let Some(ref root_hash) = *self.root_hash.read().unwrap() {
            batch.put_cf(metadata_cf, KEY_ROOT_HASH, root_hash);
        }
        
        let num_leaves = *self.num_leaves.read().unwrap();
        batch.put_cf(metadata_cf, KEY_NUM_LEAVES, num_leaves.to_le_bytes());
        
        let depth = *self.depth.read().unwrap();
        batch.put_cf(metadata_cf, KEY_DEPTH, depth.to_le_bytes());
        
        // Write hanging nodes
        {
            let hanging_nodes = self.hanging_nodes.read().unwrap();
            for (level, node_hash) in hanging_nodes.iter() {
                let key = format!("{}{}", KEY_HANGING_NODE_PREFIX, level);
                batch.put_cf(metadata_cf, key, node_hash);
            }
        }
        
        // Write nodes
        {
            let nodes_cache = self.nodes_cache.read().unwrap();
            for node in nodes_cache.values() {
                batch.put_cf(nodes_cf, &node.hash, node.encode());
                
                if let Some(ref old_hash) = node.node_hash_to_remove_from_db {
                    batch.delete_cf(nodes_cf, old_hash);
                }
            }
        }
        
        // Write key data
        {
            let key_data_cache = self.key_data_cache.read().unwrap();
            for (key, data) in key_data_cache.iter() {
                batch.put_cf(key_data_cf, key.data(), data);
            }
        }
        
        // Execute batch
        let write_opts = WriteOptions::default();
        self.db.write_opt(batch, &write_opts)?;
        
        // Clear caches
        {
            let mut nodes_cache = self.nodes_cache.write().unwrap();
            nodes_cache.clear();
        }
        {
            let mut key_data_cache = self.key_data_cache.write().unwrap();
            key_data_cache.clear();
        }
        
        *self.has_unsaved_changes.write().unwrap() = false;
        
        Ok(())
    }
    
    pub fn close(&self) -> Result<()> {
        {
            let closed = self.closed.read().unwrap();
            if *closed {
                return Ok(());
            }
        }
        
        self.flush_to_disk()?;
        
        // Mark as closed
        {
            let mut closed = self.closed.write().unwrap();
            *closed = true;
        }
        
        // Remove from global registry
        {
            let mut open_trees = OPEN_TREES.lock().unwrap();
            open_trees.remove(&self.tree_name);
        }
        
        Ok(())
    }
    
    pub fn clear(&self) -> Result<()> {
        self.error_if_closed()?;
        
        let metadata_cf = self.get_cf_handle(METADATA_CF_NAME)?;
        let nodes_cf = self.get_cf_handle(NODES_CF_NAME)?;
        let key_data_cf = self.get_cf_handle(KEY_DATA_CF_NAME)?;
        
        // Clear all column families by iterating and deleting keys
        // Note: This is less efficient than delete_range but more compatible
        
        // Clear metadata CF
        let mut iter = self.db.iterator_cf(metadata_cf, rocksdb::IteratorMode::Start);
        let mut keys_to_delete = Vec::new();
        while let Some(Ok((key, _))) = iter.next() {
            keys_to_delete.push(key.to_vec());
        }
        for key in keys_to_delete {
            self.db.delete_cf(metadata_cf, &key)?;
        }
        
        // Clear nodes CF
        let mut iter = self.db.iterator_cf(nodes_cf, rocksdb::IteratorMode::Start);
        let mut keys_to_delete = Vec::new();
        while let Some(Ok((key, _))) = iter.next() {
            keys_to_delete.push(key.to_vec());
        }
        for key in keys_to_delete {
            self.db.delete_cf(nodes_cf, &key)?;
        }
        
        // Clear key_data CF
        let mut iter = self.db.iterator_cf(key_data_cf, rocksdb::IteratorMode::Start);
        let mut keys_to_delete = Vec::new();
        while let Some(Ok((key, _))) = iter.next() {
            keys_to_delete.push(key.to_vec());
        }
        for key in keys_to_delete {
            self.db.delete_cf(key_data_cf, &key)?;
        }
        
        // Compact to reclaim space
        self.db.compact_range_cf(metadata_cf, None::<&[u8]>, None::<&[u8]>);
        self.db.compact_range_cf(nodes_cf, None::<&[u8]>, None::<&[u8]>);
        self.db.compact_range_cf(key_data_cf, None::<&[u8]>, None::<&[u8]>);
        
        // Reset in-memory state
        {
            let mut nodes_cache = self.nodes_cache.write().unwrap();
            nodes_cache.clear();
        }
        {
            let mut key_data_cache = self.key_data_cache.write().unwrap();
            key_data_cache.clear();
        }
        {
            let mut hanging_nodes = self.hanging_nodes.write().unwrap();
            hanging_nodes.clear();
        }
        
        *self.root_hash.write().unwrap() = None;
        *self.num_leaves.write().unwrap() = 0;
        *self.depth.write().unwrap() = 0;
        *self.has_unsaved_changes.write().unwrap() = false;
        
        Ok(())
    }
    
    pub fn contains_key(&self, key: &[u8]) -> Result<bool> {
        self.error_if_closed()?;
        
        if key.is_empty() {
            return Err(MerkleTreeError::InvalidArgument("Key cannot be empty".to_string()));
        }
        
        let key_data_cf = self.get_cf_handle(KEY_DATA_CF_NAME)?;
        Ok(self.db.get_cf(key_data_cf, key)?.is_some())
    }
    
    pub fn revert_unsaved_changes(&self) -> Result<()> {
        if !*self.has_unsaved_changes.read().unwrap() {
            return Ok(());
        }
        
        self.error_if_closed()?;
        
        // Clear caches
        {
            let mut nodes_cache = self.nodes_cache.write().unwrap();
            nodes_cache.clear();
        }
        {
            let mut hanging_nodes = self.hanging_nodes.write().unwrap();
            hanging_nodes.clear();
        }
        {
            let mut key_data_cache = self.key_data_cache.write().unwrap();
            key_data_cache.clear();
        }
        
        // Reload metadata from disk
        self.load_metadata()?;
        
        *self.has_unsaved_changes.write().unwrap() = false;
        
        Ok(())
    }
    
    pub fn get_root_hash_saved_on_disk(&self) -> Result<Option<Vec<u8>>> {
        self.error_if_closed()?;
        
        let metadata_cf = self.get_cf_handle(METADATA_CF_NAME)?;
        Ok(self.db.get_cf(metadata_cf, KEY_ROOT_HASH)?)
    }
}

// Utility functions - PWRHash equivalent using Keccak-256
pub fn calculate_leaf_hash(key: &[u8], data: &[u8]) -> Vec<u8> {
    keccak_256_two_inputs(key, data)
}

fn keccak_256_two_inputs(input1: &[u8], input2: &[u8]) -> Vec<u8> {
    let mut hasher = Keccak::v256();
    hasher.update(input1);
    hasher.update(input2);
    let mut output = [0u8; 32];
    hasher.finalize(&mut output);
    output.to_vec()
}
