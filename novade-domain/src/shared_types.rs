//! Shared types module for the NovaDE domain layer.
//!
//! This module provides common type definitions used across
//! different modules in the domain layer.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A unique identifier for domain entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Creates a new random entity ID.
    pub fn new() -> Self {
        EntityId(Uuid::new_v4())
    }
    
    /// Creates an entity ID from a UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        EntityId(uuid)
    }
    
    /// Gets the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    /// Converts the entity ID to a string.
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EntityId {
    fn from(uuid: Uuid) -> Self {
        EntityId(uuid)
    }
}

impl From<EntityId> for Uuid {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

/// A version number for tracking changes to entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(u64);

impl Version {
    /// Creates a new version with the specified number.
    pub fn new(version: u64) -> Self {
        Version(version)
    }
    
    /// Creates the initial version (0).
    pub fn initial() -> Self {
        Version(0)
    }
    
    /// Gets the next version.
    pub fn next(&self) -> Self {
        Version(self.0 + 1)
    }
    
    /// Gets the underlying version number.
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::initial()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A result type for domain operations.
pub type DomainResult<T> = Result<T, crate::error::DomainError>;

/// A trait for entities that can be identified.
pub trait Identifiable {
    /// Gets the entity ID.
    fn id(&self) -> EntityId;
}

/// A trait for entities that can be versioned.
pub trait Versionable {
    /// Gets the entity version.
    fn version(&self) -> Version;
    
    /// Increments the entity version.
    fn increment_version(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_id_new() {
        let id1 = EntityId::new();
        let id2 = EntityId::new();
        
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_entity_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = EntityId::from_uuid(uuid);
        
        assert_eq!(id.as_uuid(), &uuid);
    }
    
    #[test]
    fn test_entity_id_to_string() {
        let id = EntityId::new();
        let id_str = id.to_string();
        
        assert_eq!(id_str, id.0.to_string());
    }
    
    #[test]
    fn test_entity_id_display() {
        let id = EntityId::new();
        
        assert_eq!(format!("{}", id), id.0.to_string());
    }
    
    #[test]
    fn test_entity_id_from_uuid_conversion() {
        let uuid = Uuid::new_v4();
        let id: EntityId = uuid.into();
        
        assert_eq!(id.as_uuid(), &uuid);
    }
    
    #[test]
    fn test_entity_id_to_uuid_conversion() {
        let id = EntityId::new();
        let uuid: Uuid = id.into();
        
        assert_eq!(uuid, id.0);
    }
    
    #[test]
    fn test_version_new() {
        let version = Version::new(42);
        
        assert_eq!(version.value(), 42);
    }
    
    #[test]
    fn test_version_initial() {
        let version = Version::initial();
        
        assert_eq!(version.value(), 0);
    }
    
    #[test]
    fn test_version_next() {
        let version = Version::new(42);
        let next = version.next();
        
        assert_eq!(next.value(), 43);
    }
    
    #[test]
    fn test_version_display() {
        let version = Version::new(42);
        
        assert_eq!(format!("{}", version), "v42");
    }
    
    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(1);
        let v2 = Version::new(2);
        
        assert!(v1 < v2);
        assert!(v2 > v1);
    }
}
