use super::Builder;
use super::NoriaServerConfig;
use super::{DEFAULT_NORIA_VERSION, NORIA_IMAGE, ZOOKEEPER_CLIENT_SERVICE_NAME};

use roperator::serde_json::{json, Value};

pub struct Config<'svc> {
    id: &'svc str,
    name: String,
    version: &'svc str,
    max_heap: u64,
    storage_size: u64,
    replicas: usize,
}

// Default Noria Server settings
const DEFAULT_NORIA_SERVER_MAX_HEAP: u64 = 96;
const DEFAULT_NORIA_SERVER_STORAGE_SIZE: u64 = 1024;
const DEFAULT_NORIA_SERVER_REPLICAS: usize = 3;

pub fn create_config<'svc>(
    noria_server: &'svc Option<NoriaServerConfig>,
    deployment_id: &'svc str,
) -> Config<'svc> {
    let noria_server_name = format!("noria-server-{}", deployment_id);

    let noria_server_max_heap = match noria_server {
        Some(NoriaServerConfig {
            max_heap: Some(m), ..
        }) => *m,
        _ => DEFAULT_NORIA_SERVER_MAX_HEAP,
    };

    let noria_server_storage_size = match noria_server {
        Some(NoriaServerConfig {
            storage_size: Some(m),
            ..
        }) => *m,
        _ => DEFAULT_NORIA_SERVER_STORAGE_SIZE,
    };

    let noria_server_replicas = match noria_server {
        Some(NoriaServerConfig {
            replicas: Some(n), ..
        }) => *n,
        _ => DEFAULT_NORIA_SERVER_REPLICAS,
    };

    let noria_server_version = match noria_server {
        Some(NoriaServerConfig {
            version: Some(v), ..
        }) => v,
        _ => DEFAULT_NORIA_VERSION,
    };

    Config {
        id: deployment_id,
        name: noria_server_name,
        version: noria_server_version,
        max_heap: noria_server_max_heap,
        storage_size: noria_server_storage_size,
        replicas: noria_server_replicas,
    }
}

impl<'svc> Builder for Config<'svc> {
    fn children(self: &Config<'svc>, namespace: &str) -> Vec<Value> {
        let mut children = vec![];
        // Noria-server Service
        children.push(json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "ports": [{
                    "port": 6033,
                    "name": "noria",
                    "targetPort": 6033,
                }],
                "selector": {
                    "noria-operator.io/kind": "noria-server",
                    "noria-operator.io/name": self.id
                }
            }
        }));

        let noria_server_docker_mem = (1.3 * (self.max_heap as f32)) as u64;
        let noria_server_command = format!(
            r#"/usr/local/bin/noria-server --address $NODE_IP \
              --deployment {} --log-dir /var/lib/noria --memory {} \
              --quorum {} --shards 0 --zookeeper {}:2181"#,
            self.id,
            (self.max_heap * 1024 * 1024).to_string(),
            self.replicas.to_string(),
            ZOOKEEPER_CLIENT_SERVICE_NAME
        );

        // Noria-Server StatefulSet
        children.push(json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "replicas": self.replicas,
                "serviceName": self.name,
                "selector": {
                    "matchLabels": {
                        "noria-operator.io/kind": "noria-server",
                        "noria-operator.io/name": self.id,
                    }
                },
                "template": {
                    "metadata": {
                        "name": self.name,
                        "labels": {
                            "noria-operator.io/kind": "noria-server",
                            "noria-operator.io/name": self.id
                        }
                    },
                    "spec": {
                        "containers": [{
                            "command": ["bash", "-exc"],
                            "args": [noria_server_command],
                            "env": [{
                                "name": "RUST_LOG",
                                "value": "debug"
                            },{
                                "name": "NODE_IP",
                                "valueFrom": {
                                    "fieldRef": {
                                        "apiVersion": "v1",
                                        "fieldPath": "status.podIP",
                                    }
                                }
                            }],
                            "image": format!("{}:{}", NORIA_IMAGE, self.version),
                            "imagePullPolicy": "Always",
                            "name": "noria-server",
                            "ports": [{
                                "containerPort": 6033,
                                "name": "api",
                                "protocol": "TCP",
                            }],
                            "resources": {
                                "limits": {
                                    "memory": format!("{}Mi", noria_server_docker_mem)
                                },
                                "requests": {
                                    "memory": format!("{}Mi", noria_server_docker_mem)
                                }
                            },
                            "livenessProbe": {
                                "tcpSocket": {
                                    "port": 6033
                                },
                                "failureThreshold": 3,
                                "initialDelaySeconds": 60,
                                "periodSeconds": 10,
                                "successThreshold": 1,
                                "timeoutSeconds": 5,
                            },
                            "volumeMounts": [{
                                "mountPath": "/var/lib/noria",
                                "name": "data"
                            }],
                        }],
                    }
                },
                "updateStrategy": {
                    "type": "OnDelete"
                },
                "volumeClaimTemplates": [{
                    "metadata": {
                        "name": "data"
                    },
                    "spec": {
                        "accessModes": ["ReadWriteOnce"],
                        "resources": {
                            "requests": {
                                "storage": format!("{}Gi", self.storage_size / 1024 as u64)
                            }
                        }
                    }
                }]
            }
        }));

        children
    }
}
