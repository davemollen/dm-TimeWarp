use {
  crate::DmTimeWarp,
  lv2::prelude::{
    path::{FreePath, MakePath, MapPath, PathManager},
    *,
  },
  std::path::Path,
};

#[derive(FeatureCollection)]
pub struct StateFeatures<'a> {
  make_path: Option<MakePath<'a>>,
  map_path: Option<MapPath<'a>>,
  free_path: Option<FreePath<'a>>,
}

impl State for DmTimeWarp {
  type StateFeatures = StateFeatures<'static>;

  fn save(&self, mut store: StoreHandle, features: Self::StateFeatures) -> Result<(), StateErr> {
    match (features.make_path, features.map_path, features.free_path) {
      (Some(make_path), Some(map_path), Some(free_path)) => {
        let mut manager = PathManager::new(make_path, map_path, free_path);

        let (_, abstract_path) = manager.allocate_path(Path::new(&self.file_path))?;

        let _ = store
          .draft(self.urids.sample)
          .init(self.urids.atom.path)?
          .append(&*abstract_path);

        store.commit_all()
      }
      (_, _, _) => Ok(()),
    }
  }
  fn restore(
    &mut self,
    store: RetrieveHandle,
    features: Self::StateFeatures,
  ) -> Result<(), StateErr> {
    match (
      features.make_path,
      features.map_path,
      features.free_path,
      self.activated,
    ) {
      (Some(make_path), Some(map_path), Some(free_path), true) => {
        let mut manager = PathManager::new(make_path, map_path, free_path);

        let abstract_path = store
          .retrieve(self.urids.sample)?
          .read(self.urids.atom.path)?;

        self.file_path = manager
          .deabstract_path(abstract_path)?
          .to_string_lossy()
          .to_string();

        Ok(())
      }
      (_, _, _, _) => Ok(()),
    }
  }
}
