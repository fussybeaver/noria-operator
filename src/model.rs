use roperator::prelude::*;
use std::collections::HashMap;

pub static PARENT_TYPE_NORIA_CLUSTER: &K8sType = &K8sType {
    api_version: "noria-operator.io/v1alpha1",
    kind: "noria",
    plural_kind: "norias",
};

#[derive(Serialize, Deserialize)]
pub struct Noria {
    pub metadata: Metadata,
    pub spec: NoriaSpec,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub namespace: String,
}

#[derive(Serialize, Deserialize)]
pub struct NoriaSpec {
    pub deployments: Vec<Deployment>,
    pub zookeeper: Option<ZookeeperConfig>,
    pub noria_ui: Option<NoriaUiConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub noria_server: Option<NoriaServerConfig>,
    pub noria_mysql: Option<NoriaMysqlConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct NoriaConfig {
    pub version: String,
}

trait Opt {}

impl Opt for ZookeeperConfig {}

#[derive(Serialize, Deserialize)]
pub struct ZookeeperConfig {
    pub version: Option<String>,
    pub max_heap: Option<u64>,
    pub storage_size: Option<u64>,
    pub replicas: Option<usize>,
    pub additional_properties: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct NoriaServerConfig {
    pub version: Option<String>,
    pub max_heap: Option<u64>,
    pub storage_size: Option<u64>,
    pub replicas: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct NoriaMysqlConfig {
    pub version: Option<String>,
    pub max_heap: Option<u64>,
    pub replicas: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct NoriaUiConfig {
    pub version: Option<String>,
    pub max_heap: Option<u64>,
}
