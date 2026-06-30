//! Batch chunking for Airtable endpoint limits.

use nest_error::{NestError, NestResult};

use crate::codes::NEST_AIRTABLE_BATCH_LIMIT_EXCEEDED;
use crate::types::{AirtableBatchUpdate, MAX_BATCH_UPDATE_SIZE};

/// Splits updates into chunks that respect Airtable batch limits.
pub struct AirtableBatch<T> {
    items: Vec<T>,
    chunk_size: usize,
}

impl AirtableBatch<AirtableBatchUpdate> {
    /// Creates a batch helper for record updates.
    pub fn updates(updates: Vec<AirtableBatchUpdate>) -> NestResult<Self> {
        if updates.len() > MAX_BATCH_UPDATE_SIZE {
            return Err(
                NestError::validation(format!(
                    "batch exceeds Airtable limit of {MAX_BATCH_UPDATE_SIZE} records"
                ))
                .with_code(NEST_AIRTABLE_BATCH_LIMIT_EXCEEDED)
                .with_module("nest-airtable"),
            );
        }
        Ok(Self {
            items: updates,
            chunk_size: MAX_BATCH_UPDATE_SIZE,
        })
    }

    /// Chunks an arbitrary update list into batches of at most 10 records.
    pub fn chunk_updates(updates: Vec<AirtableBatchUpdate>) -> Vec<Vec<AirtableBatchUpdate>> {
        updates
            .chunks(MAX_BATCH_UPDATE_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl<T> AirtableBatch<T> {
    /// Creates a batch with a custom chunk size.
    pub fn with_chunk_size(items: Vec<T>, chunk_size: usize) -> Self {
        Self { items, chunk_size }
    }

    /// Returns non-overlapping chunks.
    pub fn chunks(&self) -> impl Iterator<Item = &[T]> {
        self.items.chunks(self.chunk_size)
    }

    /// Returns the total item count.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Consumes the batch and returns the items.
    pub fn into_items(self) -> Vec<T> {
        self.items
    }

    /// Returns whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AirtableFields;

    #[test]
    fn chunk_updates_splits_at_ten() {
        let updates: Vec<_> = (0..25)
            .map(|index| AirtableBatchUpdate {
                id: format!("rec{index}"),
                fields: AirtableFields::new(),
            })
            .collect();
        let chunks = AirtableBatch::chunk_updates(updates);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 10);
        assert_eq!(chunks[2].len(), 5);
    }

    #[test]
    fn updates_rejects_oversized_single_batch() {
        let updates: Vec<_> = (0..11)
            .map(|index| AirtableBatchUpdate {
                id: format!("rec{index}"),
                fields: AirtableFields::new(),
            })
            .collect();
        assert!(AirtableBatch::updates(updates).is_err());
    }
}
