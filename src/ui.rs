use super::Builder;
use super::NoriaUiConfig;
use super::{DEFAULT_NORIA_VERSION, NORIA_IMAGE};

use roperator::serde_json::{json, Value};

pub struct Config<'svc> {
    name: &'svc str,
    version: &'svc str,
}

pub fn create_config<'svc>(noria_ui: &'svc Option<NoriaUiConfig>) -> Config<'svc> {
    let noria_ui_version = match noria_ui {
        Some(NoriaUiConfig {
            version: Some(v), ..
        }) => v,
        _ => DEFAULT_NORIA_VERSION,
    };

    Config {
        name: "noria-ui",
        version: noria_ui_version,
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
                    "port": 80,
                    "name": "ui",
                    "targetPort": 8000,
                }],
                "selector": {
                    "noria-operator.io/kind": "noria-ui",
                    "noria-operator.io/name": "noria"
                }
            }
        }));

        // Noria-mysql Deployment
        children.push(json!({
            "apiVersion": "extensions/v1beta1",
            "kind": "Deployment",
            "metadata": {
                "name": self.name,
                "namespace": namespace,
            },
            "spec": {
                "replicas": 1,
                "selector": {
                    "matchLabels": {
                        "noria-operator.io/kind": "noria-ui",
                        "noria-operator.io/name": "noria",
                    }
                },
                "strategy": {
                    "type": "RollingUpdate"
                },
                "template": {
                    "metadata": {
                        "name": self.name,
                        "labels": {
                            "noria-operator.io/kind": "noria-ui",
                            "noria-operator.io/name": "noria"
                        }
                    },
                    "spec": {
                        "containers": [{
                            "command": ["python3", "-m", "http.server"],
                            "image": format!("{}:{}", NORIA_IMAGE, self.version),
                            "imagePullPolicy": "Always",
                            "workingDir": "/srv/noria-ui",
                            "name": "noria-ui",
                            "ports": [{
                                "containerPort": 8000,
                                "name": "web",
                                "protocol": "TCP",
                            }],
                            "livenessProbe": {
                                "httpGet": {
                                    "path": "/",
                                    "port": 8000
                                },
                                "failureThreshold": 3,
                                "initialDelaySeconds": 30,
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
