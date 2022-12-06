use crate::{cache::storage, BoxModule, CodeGenerationResult, NormalModuleAstOrSource};
use rspack_error::Result;
use std::sync::RwLock;

type Storage = dyn storage::Storage<CodeGenerationResult>;

#[derive(Debug)]
pub struct CodeGenerateOccasion {
  storage: Option<RwLock<Box<Storage>>>,
}

impl CodeGenerateOccasion {
  pub fn new(storage: Option<Box<Storage>>) -> Self {
    Self {
      storage: storage.map(RwLock::new),
    }
  }

  #[allow(clippy::unwrap_in_result)]
  pub fn use_cache<'a, F>(
    &self,
    module: &'a BoxModule,
    generator: F,
  ) -> Result<CodeGenerationResult>
  where
    F: Fn(&'a BoxModule) -> Result<CodeGenerationResult>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return generator(module),
    };

    let mut need_cache = false;
    let id: String = module.identifier().into();
    if let Some(module) = module.as_normal_module() {
      // only cache normal module
      // TODO cache all module type
      if matches!(module.ast_or_source(), NormalModuleAstOrSource::Unbuild) {
        let storage = storage.read().unwrap();
        if let Some(data) = storage.get(&id) {
          return Ok(data);
        }
        // unbuild and no cache is unexpected
        panic!("unexpected unbuild module");
      }
      need_cache = true;
    }

    // run generator and save to cache
    let data = generator(module)?;
    if need_cache {
      storage.write().unwrap().set(id, data.clone());
    }
    Ok(data)
  }
}