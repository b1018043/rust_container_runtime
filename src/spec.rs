#[derive(Serialize,Deserialize,Debug)]
pub struct Spec{
    #[serde(rename="ociVersion")]
    pub oci_version: String,
}