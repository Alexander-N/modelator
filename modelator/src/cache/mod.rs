// cache for `TlaTrace`s.
mod tla_trace;

// cache for `Tlc::next_states`.
mod next_states;

// Re-exports;
pub(crate) use next_states::NextStatesCache;
pub(crate) use tla_trace::TlaTraceCache;

use crate::artifact::{TlaConfigFile, TlaFile};
use crate::{Error, Options};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashSet;
use std::path::PathBuf;

pub(crate) struct Cache {
    cache_dir: PathBuf,
    cached_keys: HashSet<String>,
}

impl Cache {
    pub(crate) fn new(name: impl Into<String>, options: &Options) -> Result<Self, Error> {
        // create cache dir (if it doesn't exist)
        let cache_dir = options.dir.join("cache").join(name.into());
        std::fs::create_dir_all(&cache_dir).map_err(Error::io)?;

        // read files the cache directory
        let cached_keys = crate::util::read_dir(&cache_dir)?;

        Ok(Self {
            cache_dir,
            cached_keys,
        })
    }

    #[allow(clippy::ptr_arg)]
    pub(crate) fn get<V>(&self, key: &String) -> Result<Option<V>, Error>
    where
        V: DeserializeOwned,
    {
        let value = if self.cached_keys.contains(key) {
            // if this key is cached, read it from disk
            let path = self.key_path(key);
            let file = std::fs::File::open(path).map_err(Error::io)?;
            let reader = std::io::BufReader::new(file);
            let value = serde_json::from_reader(reader).map_err(Error::serde_json)?;
            Some(value)
        } else {
            None
        };
        Ok(value)
    }

    pub(crate) fn insert<V>(&mut self, key: String, value: &V) -> Result<(), Error>
    where
        V: Serialize,
    {
        // for each key, there exists at most one value; so we panic in case
        // we're trying insert a key already cached
        assert!(
            !self.cached_keys.contains(&key),
            "[modelator] trying to cache a key already cached"
        );

        // write the value associated with this key to disk
        let path = self.key_path(&key);
        let file = std::fs::File::create(path).map_err(Error::io)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer(writer, value).map_err(Error::serde_json)?;

        // mark the key as cached
        self.cached_keys.insert(key);
        Ok(())
    }

    #[allow(clippy::ptr_arg)]
    fn key_path(&self, key: &String) -> PathBuf {
        self.cache_dir.join(key)
    }
}

/// Compute a unique key given a [TlaFile] and a [TlaConfigFile].
pub(crate) fn key(tla_file: &TlaFile, tla_config_file: &TlaConfigFile) -> Result<String, Error> {
    tracing::debug!("cache::key {} {}", tla_file, tla_config_file);

    // get all tla files in the same directory
    let tla_dir = tla_file.dir();
    let files_to_hash = crate::util::read_dir(&tla_dir)?
        .into_iter()
        .filter(|filename| filename.ends_with(".tla"))
        // compute full path
        .map(|filename| tla_dir.join(filename))
        // also hash the tla config file
        .chain(std::iter::once(tla_config_file.path().clone()))
        .map(|path| crate::util::absolute_path(&path))
        // sort files so that the hash is deterministic
        .collect();

    tracing::debug!("files to hash: {:?}", files_to_hash);
    let mut digest = crate::util::digest::digest_files(files_to_hash)?;

    // also add the absolute path of the tla file to the digest; this makes
    // sure that two tla tests files living in the same directory and using
    // the same tla configuration (which we shouldn't happen when using
    // `modelator::module::tla::generate_tests`) will have different hashes
    use sha2::Digest;
    digest.update(&crate::util::absolute_path(&tla_file.path()));

    let hash = crate::util::digest::encode(digest);
    tracing::debug!("computed hash: {}", hash);
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn cache_works() {
        let modelator_dir = "cache_works";
        let options = Options::default().dir(modelator_dir);

        // create cache
        let cache_name = "my_cache";
        let mut cache = Cache::new(cache_name, &options).unwrap();

        #[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
        struct Value(usize);

        let key_a = "A".to_string();
        let value_a = Value(10);
        let key_b = "B".to_string();

        // at the beginning, no key is cached
        assert!(cache.get::<Value>(&key_a).unwrap().is_none());
        assert!(cache.get::<Value>(&key_b).unwrap().is_none());

        // cache key A
        cache.insert(key_a.clone(), &value_a).unwrap();

        // now key A is cached
        assert_eq!(cache.get(&key_a).unwrap(), Some(value_a.clone()));
        assert!(cache.get::<Value>(&key_b).unwrap().is_none());

        // start a new cache a check that it reads the cached keys from disk
        let cache = Cache::new(cache_name, &options).unwrap();
        assert_eq!(cache.get(&key_a).unwrap(), Some(value_a));
        assert!(cache.get::<Value>(&key_b).unwrap().is_none());

        // cleanup
        std::fs::remove_dir_all(modelator_dir).unwrap();
    }
}
