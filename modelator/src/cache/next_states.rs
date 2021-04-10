use super::Cache;
use crate::module::tlc::NextStatesCached;
use crate::{Error, Options};

const CACHE_NAME: &str = "next_states";

pub(crate) struct NextStatesCache {
    cache: Cache,
}

impl NextStatesCache {
    pub(crate) fn new(options: &Options) -> Result<Self, Error> {
        let cache = Cache::new(CACHE_NAME, options)?;
        Ok(Self { cache })
    }

    #[allow(clippy::ptr_arg)]
    pub(crate) fn get(&self, key: &String) -> Result<Option<NextStatesCached>, Error> {
        self.cache.get(key)
    }

    pub(crate) fn insert(&mut self, key: String, value: &NextStatesCached) -> Result<(), Error> {
        self.cache.insert(key, value)
    }
}
