use serde::{Deserialize, Serialize};

use crate::backend::malware::Malware;


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SecuritySystem {
    patch_list: Vec<Malware>
}

impl SecuritySystem {
    #[must_use]
    pub fn new(patch_list: Vec<Malware>) -> Self {
        Self { patch_list }
    }

    #[must_use]
    pub fn patch_list(&self) -> &[Malware] {
        self.patch_list.as_ref()
    }

    #[must_use]
    pub fn patches(&self, malware: &Malware) -> bool {
        self.patch_list.contains(malware)
    }
}
