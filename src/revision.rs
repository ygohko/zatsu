use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::entry::Entry;

#[derive(Serialize, Deserialize)]
pub struct Revision {
    pub entries: Vec<Entry>,
}
