use bollard::container::{ListContainersOptions, LogOutput, RestartContainerOptions, UploadToContainerOptions};
use bollard::errors::Error;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{ContainerInspectResponse, ContainerState, ContainerSummary, Health, HealthStatusEnum};
use bollard::Docker;
use bytes::Bytes;
use futures::StreamExt;
use pipewire_common::utils::Backoff;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;
use crate::containers::options::{CreateContainerOptionsBuilder, StopContainerOptionsBuilder};

pub struct ContainerApi {
    runtime: Arc<Runtime>,
    api: Arc<Docker>
}

impl ContainerApi {
    pub fn new(runtime: Arc<Runtime>, api: Arc<Docker>) -> Self {
        Self {
            runtime,
            api,
        }
    }

    pub fn get_all(&self) -> Result<Vec<ContainerSummary>, Error>{
        let mut filter = HashMap::new();
        filter.insert("label", vec!["test.container=true"]);
        let options = ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        };
        let call = self.api.list_containers(Some(options));
        self.runtime.block_on(call)
    }

    pub fn clean(&self, id: &String, state: &ContainerState) {
        println!("Clean container with id {}", id);
        if state.running.unwrap() {
            let stop_options = StopContainerOptionsBuilder::default().build();
            let call = self.api.stop_container(id, Some(stop_options));
            self.runtime.block_on(call).unwrap();
        }
        let call = self.api.remove_container(id, None);
        self.runtime.block_on(call).unwrap();
    }

    pub fn create(&self, options: &mut CreateContainerOptionsBuilder) -> String {
        let options = options
            .with_label("test.container", true.to_string())
            .build();
        println!("Create container with image {}", options.image.as_ref().unwrap());
        let call = self.api.create_container::<String, String>(None, options);
        let result = self.runtime.block_on(call).unwrap();
        result.id
    }

    pub fn start(&self, id: &String) {
        println!("Start container with id {}", id);
        let call = self.api.start_container::<String>(id, None);
        self.runtime.block_on(call).unwrap();
    }

    pub fn stop(&self, id: &String, options: &mut StopContainerOptionsBuilder) {
        println!("Stop container with id {}", id);
        let options = options.build();
        let call = self.api.stop_container(id, Some(options));
        self.runtime.block_on(call).unwrap();
    }

    pub fn restart(&self, id: &String) {
        println!("Restart container with id {}", id);
        let options = RestartContainerOptions {
            t: 0,
        };
        let call = self.api.restart_container(id, Some(options));
        self.runtime.block_on(call).unwrap();
    }

    pub fn remove(&self, id: &String) {
        println!("Remove container with id {}", id);
        let call = self.api.remove_container(id, None);
        self.runtime.block_on(call).unwrap();
    }

    pub fn inspect(&self, id: &String) -> Result<ContainerInspectResponse, pipewire_common::error::Error> {
        let call = self.api.inspect_container(id, None);
        self.runtime.block_on(call).map_err(|error| {
            pipewire_common::error::Error {
                description: error.to_string(),
            }
        })
    }

    pub fn upload(&self, id: &String, path: &str, archive: Bytes) {
        let options = UploadToContainerOptions {
            path: path.to_string(),
            no_overwrite_dir_non_dir: true.to_string(),
        };
        let call = self.api.upload_to_container(id, Some(options), archive);
        self.runtime.block_on(call).unwrap();
    }

    pub fn wait_healthy(&self, id: &String) {
        println!("Wait container with id {} to be healthy", id);
        let operation = || {
            let response = self.inspect(id);
            match response {
                Ok(value) => {
                    let state = value.state.unwrap();
                    let health = state.health.unwrap();
                    match health {
                        Health { status, .. } => {
                            match status.unwrap() {
                                HealthStatusEnum::HEALTHY => Ok(()),
                                _ => Err(pipewire_common::error::Error {
                                    description: "Container not yet healthy".to_string(),
                                })
                            }
                        }
                    }
                }
                Err(value) => Err(pipewire_common::error::Error {
                    description: format!("Container {} not ready: {}", id, value),
                })
            }
        };
        let mut backoff = Backoff::default();
        backoff.retry(operation).unwrap()
    }

    pub fn exec(
        &self,
        id: &String,
        command: Vec<&str>,
        detach: bool,
        expected_exit_code: u32,
    ) -> Result<Vec<String>, pipewire_common::error::Error> {
        let create_exec_options = CreateExecOptions {
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            cmd: Some(command),
            ..Default::default()
        };
        let call = self.api.create_exec(id.as_str(), create_exec_options);
        let create_exec_result = self.runtime.block_on(call).unwrap();
        let exec_id = create_exec_result.id;
        let start_exec_options = StartExecOptions {
            detach,
            tty: true,
            ..Default::default()
        };
        let call = self.api.start_exec(exec_id.as_str(), Some(start_exec_options));
        let start_exec_result = self.runtime.block_on(call).unwrap();
        let mut output_result: Vec<String> = Vec::new();
        if let StartExecResults::Attached { mut output, .. } = start_exec_result {
            while let Some(Ok(message)) = self.runtime.block_on(output.next()) {
                match message {
                    LogOutput::StdOut { message } => {
                        output_result.push(
                            String::from_utf8(message.to_vec()).unwrap()
                        )
                    }
                    LogOutput::StdErr { message } => {
                        eprint!("{}", String::from_utf8(message.to_vec()).unwrap())
                    }
                    LogOutput::Console { message } => {
                        output_result.push(
                            String::from_utf8(message.to_vec()).unwrap()
                        )
                    }
                    _ => {}
                }
            }
            let call = self.api.inspect_exec(exec_id.as_str());
            let exec_inspect_result = self.runtime.block_on(call).unwrap();
            let exit_code = exec_inspect_result.exit_code.unwrap();
            if exit_code != expected_exit_code as i64 {
                return Err(pipewire_common::error::Error {
                    description: format!("Unexpected exit code: {exit_code}"),
                });
            }
            let output_result = output_result.iter()
                .flat_map(move |output| {
                    output.split('\n')
                        .map(move |line| line.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Ok(output_result)
        } else {
            Ok(output_result)
        }
    }

    pub fn top(&self, id: &String) -> HashMap<String, String> {
        let call = self.api.top_processes::<&str>(id, None);
        let result = self.runtime.block_on(call).unwrap();
        let titles = result.titles.unwrap();
        let pid_column_index = titles.iter().position(move |title| *title == "PID").unwrap();
        let cmd_column_index = titles.iter().position(move |title| *title == "CMD").unwrap();
        let processes = result.processes.unwrap().iter()
            .map(|process| {
                let pid = process.get(pid_column_index).unwrap();
                let cmd = process.get(cmd_column_index).unwrap();
                (cmd.clone(), pid.clone())
            })
            .collect::<HashMap<_, _>>();
        processes
    }
}

impl Clone for ContainerApi {
    fn clone(&self) -> Self {
        Self {
            runtime: self.runtime.clone(),
            api: self.api.clone(),
        }
    }
}