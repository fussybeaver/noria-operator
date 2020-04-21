mod error;
mod model;
mod mysql;
mod server;
mod ui;
mod zookeeper;

use std::time::Duration;
use std::{env, process};

use k8s_types::apps::v1 as apps;
use k8s_types::core::v1 as core;
use k8s_types::extensions::v1beta1 as beta;

use model::*;

use roperator::prelude::*;
use roperator::runner::run_operator_with_client_config;
use roperator::serde_json::{json, Value};

use structopt::clap::arg_enum;
use structopt::StructOpt;

use log::*;

const OPERATOR_NAME: &str = "noria-operator";
const NORIA_IMAGE: &str = "fussybeaver/noria";
const CONFLUENT_ZOOKEEPER_IMAGE: &str = "confluentinc/cp-zookeeper";

const ZOOKEEPER_CLIENT_SERVICE_NAME: &str = "zookeeper-noria-client";

const DEFAULT_NORIA_VERSION: &str = "0.4.1";

arg_enum! {
    #[derive(Debug)]
    enum SourceConfig {
        Kubeconfig,
        Serviceaccount
    }
}

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(possible_values = &SourceConfig::variants(), case_insensitive = true)]
    conf: SourceConfig,
}

trait Builder {
    fn children(&self, namespace: &str) -> Vec<Value>;
}

fn handle_error(request: &SyncRequest, err: Error) -> (Value, Option<Duration>) {
    log::error!("Failed to process request: {:?}\nCause: {:?}", request, err);

    let status = json!({
        "message": err.to_string(),
        "phase": "Error",
    });

    (status, None)
}

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "roperator=debug,info,warn");
    }

    env_logger::init();

    let opt = Opt::from_args();

    let operator_config = OperatorConfig::new(OPERATOR_NAME, model::PARENT_TYPE_NORIA_CLUSTER)
        .with_child(apps::StatefulSet, ChildConfig::recreate())
        .with_child(beta::Deployment, ChildConfig::recreate())
        .with_child(core::Service, ChildConfig::recreate())
        .with_child(core::ConfigMap, ChildConfig::recreate());

    let client_config = match opt.conf {
        SourceConfig::Kubeconfig => {
            ClientConfig::from_kubeconfig(OPERATOR_NAME).expect("Couldn't load client config")
        }
        SourceConfig::Serviceaccount => {
            ClientConfig::from_service_account(OPERATOR_NAME).expect("Couldn't load client config")
        }
    };

    info!("{:?}", client_config);

    let err = run_operator_with_client_config(
        operator_config,
        client_config,
        (handle_sync, handle_error),
    );

    log::error!("Error running operator: {}", err);
    process::exit(1);
}

fn handle_sync(request: &SyncRequest) -> Result<SyncResponse, Error> {
    let crd: model::Noria = request.deserialize_parent()?;

    let noria_namespace = crd.metadata.namespace.as_str();

    let mut children = vec![];

    let mut extend_properties = vec![];

    // --
    // Zookeeper

    children.append(
        &mut zookeeper::create_config(&crd.spec.zookeeper, &mut extend_properties)
            .children(noria_namespace),
    );

    // --
    // NoriaServer, NoriaMysql per deployment

    for deployment in crd.spec.deployments {
        if deployment.id.contains('-') {
            return Err(Box::new(error::DeploymentIdDashError { id: deployment.id }));
        }

        children.append(
            &mut server::create_config(&deployment.noria_server, &deployment.id)
                .children(noria_namespace),
        );

        children.append(
            &mut mysql::create_config(&deployment.noria_mysql, &deployment.id)
                .children(noria_namespace),
        );
    }

    // --
    // NoriaUI

    children.append(&mut ui::create_config(&crd.spec.noria_ui).children(noria_namespace));

    let status = json!({
        "message": "Sync complete",
    });

    Ok(SyncResponse {
        status,
        children,
        resync: None,
    })
}
