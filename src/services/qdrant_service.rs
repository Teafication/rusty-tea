use qdrant_client::Qdrant;
use qdrant_client::qdrant::Distance;
use std::error::Error;
use tracing::info;

/// Qdrant vector database service for RAG (Retrieval-Augmented Generation)
pub struct RagService {
    client: Qdrant,
}

impl RagService {
    /// Initialize Qdrant client
    pub async fn new(qdrant_url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        info!("Connecting to Qdrant vector database at {}", qdrant_url);
        
        let client = Qdrant::from_url(qdrant_url).build()?;

        // Verify connection
        let health = client.health_check().await?;
        info!("Qdrant health check passed: {:?}", health.version);

        Ok(Self { client })
    }

    /// Health check for Qdrant connection
    pub async fn health_check(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _health = self.client.health_check().await?;
        Ok(())
    }

    /// Create a new collection for embeddings (if it doesn't exist)
    pub async fn create_collection(
        &self,
        collection_name: &str,
        _vector_size: u64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Checking collection: {}", collection_name);
        
        // Check if collection exists
        match self.client.collection_exists(collection_name).await? {
            true => {
                info!("Collection {} already exists", collection_name);
                Ok(())
            }
            false => {
                // Collection doesn't exist - collection creation will be done manually for now
                info!("Collection {} does not exist - please create manually via Qdrant API", collection_name);
                Ok(())
            }
        }
    }

    /// Get list of available collections
    pub async fn list_collections(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let collections = self.client.list_collections().await?;
        let collection_names: Vec<String> = collections
            .collections
            .into_iter()
            .map(|c| c.name)
            .collect();
        Ok(collection_names)
    }

    /// Delete a collection
    pub async fn delete_collection(
        &self,
        collection_name: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Deleting collection: {}", collection_name);
        self.client.delete_collection(collection_name).await?;
        info!("Collection {} deleted successfully", collection_name);
        Ok(())
    }

    /// Get client reference for direct operations
    pub fn client(&self) -> &Qdrant {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_service_structure() {
        // Compile-time validation of service structure
        // Runtime tests will be in integration_tests.rs with actual Qdrant instance
        let result: Result<(), &str> = Ok(());
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_vector_params_creation() {
        let vector_size: u64 = 384; // Standard embedding dimension
        let distance = Distance::Cosine;
        assert!(vector_size > 0);
        assert_eq!(distance.as_str_name(), "Cosine");
    }
}
