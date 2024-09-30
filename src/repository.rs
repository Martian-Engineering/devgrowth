use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository {
    pub id: i32,
    pub name: string,
    pub owner: string,
}
