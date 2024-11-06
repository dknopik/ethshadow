use crate::clients::CommonParams;
use crate::clients::{Client, ValidatorDemand};
use crate::clients::{BEACON_API_PORT, CL_PROMETHEUS_PORT, ENGINE_API_PORT};
use crate::config::shadow::Process;
use crate::node::{NodeInfo, SimulationContext};
use crate::validators::Validator;
use crate::{CowStr, Error};
use itertools::Itertools;
use log::{debug, error};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::Command;

const PORT: &str = "31000";

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Teku {
    #[serde(flatten)]
    pub common: CommonParams,
    pub validators: ValidatorDemand,
    pub environment: HashMap<CowStr, CowStr>,
    pub use_unsafe_test_stub: bool
}

impl Default for Teku {
    fn default() -> Self {
        Teku {
            common: CommonParams::default(),
            validators: ValidatorDemand::Any,
            environment: HashMap::new(),
            use_unsafe_test_stub: false,
        }
    }
}

#[typetag::deserialize(name = "teku")]
impl Client for Teku {
    fn add_to_node<'a>(
        &self,
        node: &NodeInfo<'a>,
        ctx: &mut SimulationContext<'a>,
        validators: &[Validator],
    ) -> Result<Process, Error> {
        let dir = node.dir().join("teku");

        let ip = node.ip();

        ctx.add_cl_http_endpoint(format!("{ip}:{BEACON_API_PORT}"));
        ctx.add_cl_monitoring_endpoint(
            node.location(),
            node.reliability(),
            format!("{ip}:{CL_PROMETHEUS_PORT}"),
        );

        let validator_arg = if validators.is_empty() {
            String::new()
        } else {
            let validators_dest = dir.join("validators");
            fs::create_dir_all(&validators_dest)?;

            for validator in validators {
                let key = validator.key().to_str().ok_or(Error::NonUTF8Path)?;
                fs::rename(
                    validator.base_path().join("secrets").join(key),
                    validators_dest.join(format!("{key}.txt")),
                )?;
                fs::rename(
                    validator.base_path().join("keys").join(key).join("voting-keystore.json"),
                    validators_dest.join(format!("{key}.json")),
                )?;
            }
            format!(
                "\\\"--validator-keys={}:{0}\\\"",
                validators_dest.to_str().ok_or(Error::NonUTF8Path)?,
            )
        };

        let ee_config = if !self.use_unsafe_test_stub {
            &format!("--ee-endpoint=http://localhost:{ENGINE_API_PORT} \
                        \\\"--ee-jwt-secret-file={}\\\"",
                    ctx.jwt_path().to_str().ok_or(Error::NonUTF8Path)?)
        } else {
            "--ee-endpoint=unsafe-test-stub"
        };

        let dir = dir.to_str().ok_or(Error::NonUTF8Path)?;

        // problem: the command "teku" is a shell script - but shadow needs an ELF. In that script
        // the actual command is built and called.
        // solution: we simply execute the script with exec aliased to echo and use the outputted
        // string to configure shadow
        let mut command = Command::new("bash");
        command.arg("-c").arg(format!(
            "\
                exec() {{
                echo $*
                }}
                cd $(dirname $(realpath $(which {1}))) && \
                BASH_ARGV0={1} {}&& \
                source {} \
                \\\"--network={}/config.yaml\\\" \
                --eth1-deposit-contract-address=0x4242424242424242424242424242424242424242 \
                \\\"--genesis-state={2}/genesis.ssz\\\" \
                \\\"--data-path={dir}\\\" \
                {ee_config} \
                --p2p-discovery-bootnodes={} \
                --p2p-port={PORT} \
                --p2p-advertised-ip={ip} \
                --p2p-nat-method=NONE \
                --rest-api-enabled=true \
                --rest-api-port={BEACON_API_PORT} \
                --metrics-interface=0.0.0.0 \
                --metrics-port={CL_PROMETHEUS_PORT} \
                --metrics-enabled=true {validator_arg} {}",
            self.environment.iter().map(|(k, v)| format!("&& {k}=\"{v}\" ")).join(""),
            self.common.executable_or("teku"),
            ctx.metadata_path().to_str().ok_or(Error::NonUTF8Path)?,
            ctx.cl_bootnode_enrs().join(","),
            self.common.arguments("--validators-proposer-default-fee-recipient=0xf97e180c050e5Ab072211Ad2C213Eb5AEE4DF134"),
        ));
        debug!("Invoking: {command:?}");
        let output = command.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("teku script failed with error: {}", stderr);
            return Err(Error::ChildProcessFailure("teku".to_string()));
        }
        let command = String::from_utf8(output.stdout).map_err(|_| Error::NonUTF8Path)?;
        debug!("resulting command: {command}");
        let Some((java, args)) = command.split_once(' ') else {
            error!("teku returned something unexpected: {command}");
            return Err(Error::ChildProcessFailure("teku".to_string()));
        };

        Ok(Process {
            path: java.to_string().into(),
            args: args.to_string(),
            environment: HashMap::new(),
            expected_final_state: "running".into(),
            start_time: "5s".into(),
        })
    }

    fn is_cl_client(&self) -> bool {
        true
    }

    fn validator_demand(&self) -> ValidatorDemand {
        self.validators
    }
}
