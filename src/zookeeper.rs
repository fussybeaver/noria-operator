use super::Builder;
use super::ZookeeperConfig;
use super::CONFLUENT_ZOOKEEPER_IMAGE;

use roperator::serde_json::{json, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Default Zookeeper settings
const DEFAULT_ZOOKEEPER_MAX_HEAP: u64 = 512;
const DEFAULT_ZOOKEEPER_STORAGE_SIZE: u64 = 1024;
const DEFAULT_ZOOKEEPER_REPLICAS: usize = 3;
const DEFAULT_ZOOKEEPER_VERSION: &str = "5.3.3";

pub struct Config<'zk> {
    name: &'zk str,
    version: &'zk str,
    max_heap: u64,
    storage_size: u64,
    replicas: usize,
    properties: Vec<(&'zk str, &'zk str)>,
}

fn calculate_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    let b = s.finish().to_be_bytes();
    hex::encode(b)
}

pub fn create_config<'zk>(
    zookeeper: &'zk Option<ZookeeperConfig>,
    extend_properties: &'zk mut Vec<(String, String)>,
) -> Config<'zk> {
    let zookeeper_name = "zookeeper-noria";
    let zookeeper_max_heap = match zookeeper {
        Some(ZookeeperConfig {
            max_heap: Some(m), ..
        }) => *m,
        _ => DEFAULT_ZOOKEEPER_MAX_HEAP,
    };

    let zookeeper_storage_size = match zookeeper {
        Some(ZookeeperConfig {
            storage_size: Some(m),
            ..
        }) => *m,
        _ => DEFAULT_ZOOKEEPER_STORAGE_SIZE,
    };

    let zookeeper_replicas = match zookeeper {
        Some(ZookeeperConfig {
            replicas: Some(n), ..
        }) => *n,
        _ => DEFAULT_ZOOKEEPER_REPLICAS,
    };

    let zookeeper_version = match zookeeper {
        Some(ZookeeperConfig {
            version: Some(v), ..
        }) => v,
        _ => DEFAULT_ZOOKEEPER_VERSION,
    };

    let mut zookeeper_properties = vec![
        ("autopurge.purgeInterval", "1"),
        ("tickTime", "2000"),
        ("initLimit", "5"),
        ("syncLimit", "2"),
        ("dataDir", "/var/lib/zookeeper"),
        ("clientPort", "2181"),
        ("4lw.commands.whitelist", "stat, ruok, conf, isro"),
    ];

    //let mut extend_properties = vec![];
    let zookeeper_nodes_service_name = format!("{}-nodes", zookeeper_name);
    for n in 0..zookeeper_replicas {
        extend_properties.push((
            format!("server.{}", n + 1),
            format!(
                "{}-{}.{}:{}:{}",
                zookeeper_name, n, zookeeper_nodes_service_name, 2888, 3888
            ),
        ));
    }

    // is there a better way of doing this?
    zookeeper_properties.append(
        &mut extend_properties
            .iter()
            .map(|(key, val)| (key.as_str(), val.as_str()))
            .collect::<Vec<(&str, &str)>>(),
    );

    match zookeeper {
        Some(ZookeeperConfig {
            additional_properties: Some(props),
            ..
        }) => zookeeper_properties.extend_from_slice(
            &props
                .iter()
                .map(|(key, val)| (key.as_str(), val.as_str()))
                .collect::<Vec<(&str, &str)>>(),
        ),
        _ => (),
    };

    Config {
        name: zookeeper_name,
        version: zookeeper_version,
        max_heap: zookeeper_max_heap,
        storage_size: zookeeper_storage_size,
        replicas: zookeeper_replicas,
        properties: zookeeper_properties,
    }
}

impl<'zk> Builder for Config<'zk> {
    fn children(self: &Config<'zk>, namespace: &str) -> Vec<Value> {
        let mut children = vec![];

        let chksum = calculate_hash(&self.properties);

        let zookeeper_properties_name = format!("{}-properties-{}", self.name, chksum);
        let zookeeper_client_service_name = format!("{}-client", self.name);
        let zookeeper_nodes_service_name = format!("{}-nodes", self.name);

        let zookeeper_docker_mem = (1.3 * (self.max_heap as f32)) as u64;
        children.push(json!({
            "apiVersion": "v1",
            "kind": "ConfigMap",
            "metadata": {
                "name": zookeeper_properties_name,
                "namespace": namespace,
            },
            "data": {
                "zookeeper.properties":
                    self.properties.iter().fold(String::new(), |mut acc, (key, val)| {
                        acc.push_str(&format!("{}={}\n", key, val));
                        acc
                    })
            }
        }));

        children.push(json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": format!("{}-nodes", self.name),
                "namespace": namespace,
            },
            "spec": {
                "ports": [{
                    "port": 2181,
                    "name": "clients",
                    "targetPort": 2181,
                },{
                    "port": 2888,
                    "name": "clustering",
                    "targetPort": 2888,
                },{
                    "port": 3888,
                    "name": "leader-election",
                    "targetPort": 3888,
                }],
                "publishNotReadyAddresses": true,
                "clusterIP": "None",
                "sessionAffinity": "None",
                "selector": {
                    "noria-operator.io/kind": "zookeeper",
                    "noria-operator.io/name": "noria"
                }
            }
        }));

        children.push(json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": zookeeper_client_service_name,
                "namespace": namespace,
            },
            "spec": {
                "ports": [{
                    "port": 2181,
                    "name": "clients",
                    "targetPort": 2181,
                }],
                "selector": {
                    "noria-operator.io/kind": "zookeeper",
                    "noria-operator.io/name": "noria"
                }
            }
        }));

        // Zookeeper StatefulSet
        children.push(json!({
            "apiVersion": "apps/v1",
            "kind": "StatefulSet",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "podManagementPolicy": "Parallel",
                "replicas": self.replicas,
                "serviceName": zookeeper_nodes_service_name,
                "selector": {
                    "matchLabels": {
                        "noria-operator.io/kind": "zookeeper",
                        "noria-operator.io/name": "noria",
                    }
                },
                "template": {
                    "metadata": {
                        "name": self.name,
                        "labels": {
                            "noria-operator.io/kind": "zookeeper",
                            "noria-operator.io/name": "noria"
                        }
                    },
                    "spec": {
                        "initContainers": [{
                            "name": "init-zookeeper",
                            "image": format!("{}:{}", CONFLUENT_ZOOKEEPER_IMAGE, self.version),
                            "command": ["bash", "-c",
                            r#"set -ex
                        [[ `hostname` =~ -([0-9]+)$ ]] || exit 1
                        echo $((BASH_REMATCH[1] + 1)) > /var/lib/zookeeper/myid"#],
                            "volumeMounts": [{
                                "mountPath": "/var/lib/zookeeper",
                                "name": "data"
                            }]
                        }],
                        "containers": [{
                            "command": ["/usr/bin/zookeeper-server-start"],
                            "args": ["/etc/kafka/zookeeper.properties"],
                            "env": [{
                                "name": "KAFKA_HEAP_OPTS",
                                "value": format!("-Xmx{}m", self.max_heap)
                            }],
                            "image": CONFLUENT_ZOOKEEPER_IMAGE,
                            "imagePullPolicy": "IfNotPresent",
                            "name": "zookeeper",
                            "ports": [{
                                "containerPort": 2181,
                                "name": "clients",
                                "protocol": "TCP",
                            },{
                                "containerPort": 2888,
                                "name": "clustering",
                                "protocol": "TCP",
                            },{
                                "containerPort": 3888,
                                "name": "leader-election",
                                "protocol": "TCP",
                            }],
                            "resources": {
                                "limits": {
                                    "memory": format!("{}Mi", zookeeper_docker_mem)
                                },
                                "requests": {
                                    "memory": format!("{}Mi", zookeeper_docker_mem)
                                }
                            },
                            "livenessProbe": {
                                "exec": {
                                    "command": [
                                        "bash",
                                        "-exc",
                                        r#"[[ "$(echo ruok | nc 127.0.0.1 2181)" = "imok" ]]"#,
                                    ]
                                },
                                "failureThreshold": 3,
                                "initialDelaySeconds": 60,
                                "periodSeconds": 10,
                                "successThreshold": 1,
                                "timeoutSeconds": 5,
                            },
                            "volumeMounts": [{
                                "mountPath": "/var/lib/zookeeper",
                                "name": "data"
                            }, {
                                "mountPath": "/etc/kafka/zookeeper.properties",
                                "subPath": "zookeeper.properties",
                                "name": "properties"
                            }]
                        }],
                        "volumes": [{
                            "name": "properties",
                            "configMap": {
                                "name": zookeeper_properties_name
                            }
                        }]
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
                                "storage": format!("{}Gi",  self.storage_size / 1024 as u64)
                            }
                        }
                    }
                }]
            }

        }));

        children
    }
}
