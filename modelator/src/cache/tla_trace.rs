use super::Cache;
use crate::artifact::TlaTrace;
use crate::{Error, Options};

const CACHE_NAME: &str = "tla_trace";

pub(crate) struct TlaTraceCache {
    cache: Cache,
}

impl TlaTraceCache {
    pub(crate) fn new(options: &Options) -> Result<Self, Error> {
        let cache = Cache::new(CACHE_NAME, options)?;
        Ok(Self { cache })
    }

    #[allow(clippy::ptr_arg)]
    pub(crate) fn get(&self, key: &String) -> Result<Option<TlaTrace>, Error> {
        self.cache.get(key)
    }

    pub(crate) fn insert(&mut self, key: String, tla_trace: &TlaTrace) -> Result<(), Error> {
        self.cache.insert(key, tla_trace)
    }
}
