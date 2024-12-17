
#[derive(serde::Deserialize, Clone)]
pub struct GetObjectBody {
    pub object_id: Option<String>,
    pub catalog: Option<String>,
}
