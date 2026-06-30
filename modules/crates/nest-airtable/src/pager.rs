//! Offset-based pagination for Airtable list endpoints.

use nest_error::NestResult;

use crate::client::AirtableClient;
use crate::error::check_cancelled;
use crate::types::{AirtableListPage, AirtableListParams};

/// Iterates Airtable list pages using offset tokens.
pub struct AirtablePager {
    client: AirtableClient,
    table: String,
    params: AirtableListParams,
    offset: Option<String>,
    finished: bool,
}

impl AirtablePager {
    /// Creates a pager for the given logical table.
    pub fn new(client: AirtableClient, table: impl Into<String>, params: AirtableListParams) -> Self {
        Self {
            client,
            table: table.into(),
            params,
            offset: None,
            finished: false,
        }
    }

    /// Fetches the next page, or `None` when exhausted.
    pub async fn next_page(&mut self) -> NestResult<Option<AirtableListPage>> {
        if self.finished {
            return Ok(None);
        }

        check_cancelled(self.params.cancel.as_ref())?;

        let page = self
            .client
            .list_records_page_with_offset(&self.table, &self.params, self.offset.as_deref())
            .await?;
        self.advance(page)
    }

    fn advance(&mut self, page: AirtableListPage) -> NestResult<Option<AirtableListPage>> {
        if page.offset.is_none() {
            self.finished = true;
        } else {
            self.offset = page.offset.clone();
        }
        Ok(Some(page))
    }
}
