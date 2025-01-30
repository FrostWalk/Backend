use derive_new::new;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, new)]
pub(crate) struct Claims {
    pub sub: i64,
    pub iat: usize,
    pub exp: usize,
}
