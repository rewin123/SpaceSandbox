
pub enum AssetType {
    Mesh
}

trait Asset {
    fn get_type(&self) -> AssetType;
}