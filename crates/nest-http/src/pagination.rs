//! Pagination types.

/// Pagination request parameters.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageRequest {
    /// Page number (1-based).
    pub page: u64,
    /// Items per page.
    pub page_size: u64,
}

impl PageRequest {
    /// Creates a page request.
    pub fn new(page: u64, page_size: u64) -> Self {
        Self { page, page_size }
    }

    /// Returns the zero-based offset.
    pub fn offset(self) -> u64 {
        (self.page.saturating_sub(1)) * self.page_size
    }
}

/// Paginated result set.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Page<T> {
    /// Items in this page.
    pub items: Vec<T>,
    /// Total item count, if known.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub total: Option<u64>,
}

impl<T> Page<T> {
    /// Creates a page of items.
    pub fn new(items: Vec<T>) -> Self {
        Self { items, total: None }
    }

    /// Creates a page with a total count.
    pub fn with_total(items: Vec<T>, total: u64) -> Self {
        Self {
            items,
            total: Some(total),
        }
    }
}
