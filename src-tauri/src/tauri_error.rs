/// Implement our own error supporting anyhow's Result type 
/// but with serde's Serialise implemented to make it compatible
/// with tauri
/// Stolen from https://github.com/tauri-apps/tauri/discussions/3913
use serde::{ser::Serializer, Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum TauriError {
  #[error(transparent)]
  Error(#[from] anyhow::Error),
}

impl Serialize for TauriError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.to_string().as_ref())
  }
}

pub type TauriResult<T, E = TauriError> = anyhow::Result<T, E>;