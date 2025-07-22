use serde::{Deserialize, Serialize};

use crate::backend::malware::Malware;


#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SecuritySystem {
    patches: Vec<Malware>
}

impl SecuritySystem {
    #[must_use]
    pub fn new(patches: Vec<Malware>) -> Self {
        Self { patches }
    }

    #[must_use]
    pub fn patches(&self) -> &[Malware] {
        self.patches.as_ref()
    }

    #[must_use]
    pub fn is_patched(&self, malware: &Malware) -> bool {
        self.patches.contains(malware)
    }
}
