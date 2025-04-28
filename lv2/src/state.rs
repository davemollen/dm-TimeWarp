use {
  crate::DmTimeWarp,
  lv2::prelude::{
    path::{FreePath, MakePath, MapPath, PathManager},
    *,
  },
  std::{ffi::CStr, path::Path},
};

#[derive(FeatureCollection)]
pub struct StateFeatures<'a> {
  make_path: Option<MakePath<'a>>,
  map_path: Option<MapPath<'a>>,
  free_path: Option<FreePath<'a>>,
  log: Log<'a>,
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

        // let _ = store
        //   .draft(self.urids.sample)
        //   .init(self.urids.atom.path)?
        //   .append(&self.file_path);

        let message = format!("saving file path: {}\n\0", &self.file_path);
        let _ = features.log.print_cstr(
          self.urids.log.note,
          CStr::from_bytes_with_nul(message.as_bytes()).unwrap(),
        );

        store.commit_all()
      }
      _ => Ok(()),
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

        // self.file_path = store
        //   .retrieve(self.urids.sample)?
        //   .read(self.urids.atom.path)?
        //   .to_string();

        let message = format!("restoring file path: {}\n\0", self.file_path);
        let _ = features.log.print_cstr(
          self.urids.log.note,
          CStr::from_bytes_with_nul(message.as_bytes()).unwrap(),
        );

        Ok(())
      }
      _ => Ok(()),
    }
  }
}
