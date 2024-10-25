use crate::clients::Client;
use crate::clients::{BEACON_API_PORT, CL_PROMETHEUS_PORT, ENGINE_API_PORT};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::Validator;
use crate::CowStr;
use crate::Error;
use serde::Deserialize;
use std::collections::HashMap;

const PORT: &str = "31000";

#[derive(Deserialize, Debug, Clone)]
pub struct Prysm {
    pub executable: CowStr,
    #[serde(default)]
    pub extra_args: String,
}

#[typetag::deserialize(name = "prysm")]
impl Client for Prysm {
    fn add_to_node<'a>(
        &self,
        node: &NodeInfo<'a>,
        ctx: &mut SimulationContext<'a>,
        _validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("prysm");
        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        let ip = node.ip();

        ctx.add_cl_http_endpoint(format!("{ip}:{BEACON_API_PORT}"));
        ctx.add_cl_monitoring_endpoint(
            node.location(),
            node.reliability(),
            format!("{ip}:{CL_PROMETHEUS_PORT}"),
        );

        let meta_dir = ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?;

        Ok(Process {
            path: self.executable.clone(),
            args: format!(
                "--chain-config-file \"{meta_dir}/config.yaml\" \
                --contract-deployment-block 0 \
                --deposit-contract 0x4242424242424242424242424242424242424242 \
                --genesis-state \"{meta_dir}/genesis.ssz\"
                --datadir \"{dir}\" \
                --execution-endpoint http://localhost:{ENGINE_API_PORT} \
                --jwt-secret \"{}\" \
                --bootstrap-node {} \
                --p2p-tcp-port {PORT} \
                --p2p-udp-port {PORT} \
                --p2p-host-ip {ip} \
                --http-port {BEACON_API_PORT} \
                --monitoring-host 0.0.0.0 \
                --monitoring-port {CL_PROMETHEUS_PORT} \
                {} ",
                ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?,
                ctx.cl_bootnode_enrs().join(" --bootstrap-node "),
                self.extra_args,
            ),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn is_cl_client(&self) -> bool {
        true
    }
}
