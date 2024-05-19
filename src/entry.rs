use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub hash: String,
}
