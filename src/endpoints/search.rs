//! Endpoint functions related to search

use std::collections::HashMap;

use crate::client::{Tidal, TidalSearch};
use crate::config::params;
use crate::error::TidalResult;

pub struct Search<'a>(pub &'a Tidal);

impl Search<'_> {
    pub async fn find(&self, term: &str, limit: Option<u16>) -> TidalResult<TidalSearch> {
        let url = "/search";
        let limit = limit.unwrap_or(10);
        let mut query_params: HashMap<String, String> = HashMap::with_capacity(2);
        query_params.insert(params::QUERY.to_owned(), term.to_owned());
        query_params.insert(params::LIMIT.to_owned(), limit.to_string());
        let result = self.0.get(&url, &query_params).await?;
        Tidal::convert_result::<TidalSearch>(&result)
    }
}

#[cfg(test)]
mod tests {
    // Tests have been moved to separate test files for better organization
}
