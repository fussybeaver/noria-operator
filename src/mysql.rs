use super::Builder;
use super::NoriaMysqlConfig;
use super::{DEFAULT_NORIA_VERSION, NORIA_IMAGE, ZOOKEEPER_CLIENT_SERVICE_NAME};

use roperator::serde_json::{json, Value};

pub struct Config<'svc> {
    id: &'svc str,
    name: String,
    version: &'svc str,
    replicas: usize,
}

// Default Noria Mysql settings
const DEFAULT_NORIA_MYSQL_REPLICAS: usize = 3;

pub fn create_config<'svc>(
    noria_mysql: &'svc Option<NoriaMysqlConfig>,
    deployment_id: &'svc str,
) -> Config<'svc> {
    let noria_mysql_replicas = match noria_mysql {
        Some(NoriaMysqlConfig {
            replicas: Some(n), ..
        }) => *n,
        _ => DEFAULT_NORIA_MYSQL_REPLICAS,
    };

    let noria_mysql_version = match noria_mysql {
        Some(NoriaMysqlConfig {
            version: Some(v), ..
        }) => v,
        _ => DEFAULT_NORIA_VERSION,
    };

    let noria_mysql_name = format!("noria-mysql-{}", deployment_id);

    Config {
        name: noria_mysql_name,
        id: deployment_id,
        version: noria_mysql_version,
        replicas: noria_mysql_replicas,
    }
}

impl<'svc> Builder for Config<'svc> {
    fn children(self: &Config<'svc>, namespace: &str) -> Vec<Value> {
        let mut children = vec![];
        // Noria-mysql Service
        children.push(json!({
            "apiVersion": "v1",
            "kind": "Service",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "ports": [{
                    "port": 3306,
                    "name": "mysql",
                    "targetPort": 3306,
                }],
                "selector": {
                    "noria-operator.io/kind": "noria-mysql",
                    "noria-operator.io/name": self.id
                }
            }
        }));

        let noria_mysql_command = format!(
            r#"/usr/local/bin/noria-mysql --address ${{NODE_IP}}:3306 \
              --deployment {} --zookeeper-address {}:2181"#,
            self.id, ZOOKEEPER_CLIENT_SERVICE_NAME
        );

        // Noria-mysql Deployment
        children.push(json!({
            "apiVersion": "extensions/v1beta1",
            "kind": "Deployment",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "replicas": self.replicas,
                "selector": {
                    "matchLabels": {
                        "noria-operator.io/kind": "noria-mysql",
                        "noria-operator.io/name": self.id,
                    }
                },
                "strategy": {
                    "type": "RollingUpdate"
                },
                "template": {
                    "metadata": {
                        "name": self.name,
                        "labels": {
                            "noria-operator.io/kind": "noria-mysql",
                            "noria-operator.io/name": self.id
                        }
                    },
                    "spec": {
                        "containers": [{
                            "command": ["bash", "-exc"],
                            "args": [noria_mysql_command],
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
                            "name": "noria-mysql",
                            "ports": [{
                                "containerPort": 3306,
                                "name": "clients",
                                "protocol": "TCP",
                            }],
                            "livenessProbe": {
                                "exec": {
                                    "command": [
                                        "bash",
                                        "-exc",
                                        "mysqladmin ping -h \"$NODE_IP\"",
                                    ]
                                },
                                "failureThreshold": 3,
                                "initialDelaySeconds": 60,
                                "periodSeconds": 10,
                                "successThreshold": 1,
                                "timeoutSeconds": 5,
                            },
                        }]
                    }
                }
            }
        }));

        children
    }
}
