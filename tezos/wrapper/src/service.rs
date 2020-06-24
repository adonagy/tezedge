// Copyright (c) SimpleStaking and Tezedge Contributors
// SPDX-License-Identifier: MIT

use std::cell::RefCell;
use std::convert::AsRef;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;

use failure::Fail;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use strum_macros::IntoStaticStr;
use wait_timeout::ChildExt;

use crypto::hash::{ChainId, ContextHash, ProtocolHash};
use ipc::*;
use tezos_api::environment::TezosEnvironmentConfiguration;
use tezos_api::ffi::*;
use tezos_api::identity::Identity;
use tezos_context::channel::{context_receive, context_send, ContextAction};
use tezos_messages::p2p::encoding::prelude::*;

use crate::protocol::*;

/// This command message is generated by tezedge node and is received by the protocol runner.
#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
enum ProtocolMessage {
    ApplyBlockCall(ApplyBlockParams),
    ChangeRuntimeConfigurationCall(TezosRuntimeConfiguration),
    InitProtocolContextCall(InitProtocolContextParams),
    GenesisResultDataCall(GenesisResultDataParams),
    GenerateIdentity(GenerateIdentityParams),
    ShutdownCall,
}

#[derive(Serialize, Deserialize, Debug)]
struct ApplyBlockParams {
    chain_id: ChainId,
    block_header: BlockHeader,
    predecessor_block_header: BlockHeader,
    operations: Vec<Option<OperationsForBlocksMessage>>,
    max_operations_ttl: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct InitProtocolContextParams {
    storage_data_dir: String,
    genesis: GenesisChain,
    genesis_max_operations_ttl: u16,
    protocol_overrides: ProtocolOverrides,
    commit_genesis: bool,
    enable_testchain: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenesisResultDataParams {
    genesis_context_hash: ContextHash,
    chain_id: ChainId,
    genesis_protocol_hash: ProtocolHash,
    genesis_max_operations_ttl: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct GenerateIdentityParams {
    expected_pow: f64,
}

/// This event message is generated as a response to the `ProtocolMessage` command.
#[derive(Serialize, Deserialize, Debug, IntoStaticStr)]
enum NodeMessage {
    ApplyBlockResult(Result<ApplyBlockResponse, ApplyBlockError>),
    ChangeRuntimeConfigurationResult(Result<(), TezosRuntimeConfigurationError>),
    InitProtocolContextResult(Result<InitProtocolContextResult, TezosStorageInitError>),
    CommitGenesisResultData(Result<CommitGenesisResult, GetDataError>),
    GenerateIdentityResult(Result<Identity, TezosGenerateIdentityError>),
    ShutdownResult,
}

/// Empty message
#[derive(Serialize, Deserialize, Debug)]
struct NoopMessage;

pub fn process_protocol_events<P: AsRef<Path>>(socket_path: P) -> Result<(), IpcError> {
    let ipc_client: IpcClient<NoopMessage, ContextAction> = IpcClient::new(socket_path);
    let (_, mut tx) = ipc_client.connect()?;
    while let Ok(action) = context_receive() {
        tx.send(&action)?;
        if let ContextAction::Shutdown = action {
            break;
        }
    }

    Ok(())
}

/// Establish connection to existing IPC endpoint (which was created by tezedge node).
/// Begin receiving commands from the tezedge node until `ShutdownCall` command is received.
pub fn process_protocol_commands<Proto: ProtocolApi, P: AsRef<Path>>(socket_path: P) -> Result<(), IpcError> {
    let ipc_client: IpcClient<ProtocolMessage, NodeMessage> = IpcClient::new(socket_path);
    let (mut rx, mut tx) = ipc_client.connect()?;
    while let Ok(cmd) = rx.receive() {
        match cmd {
            ProtocolMessage::ApplyBlockCall(params) => {
                let res = Proto::apply_block(
                    &params.chain_id,
                    &params.block_header,
                    &params.predecessor_block_header,
                    &params.operations,
                    params.max_operations_ttl,
                );
                tx.send(&NodeMessage::ApplyBlockResult(res))?;
            }
            ProtocolMessage::ChangeRuntimeConfigurationCall(params) => {
                let res = Proto::change_runtime_configuration(params);
                tx.send(&NodeMessage::ChangeRuntimeConfigurationResult(res))?;
            }
            ProtocolMessage::InitProtocolContextCall(params) => {
                let res = Proto::init_protocol_context(
                    params.storage_data_dir,
                    params.genesis,
                    params.protocol_overrides,
                    params.commit_genesis,
                    params.enable_testchain,
                );
                tx.send(&NodeMessage::InitProtocolContextResult(res))?;
            }
            ProtocolMessage::GenesisResultDataCall(params) => {
                let res = Proto::genesis_result_data(
                    &params.genesis_context_hash,
                    &params.chain_id,
                    &params.genesis_protocol_hash,
                    params.genesis_max_operations_ttl,
                );
                tx.send(&NodeMessage::CommitGenesisResultData(res))?;
            }
            ProtocolMessage::GenerateIdentity(params) => {
                let res = Proto::generate_identity(params.expected_pow);
                tx.send(&NodeMessage::GenerateIdentityResult(res))?;
            }
            ProtocolMessage::ShutdownCall => {
                context_send(ContextAction::Shutdown).expect("Failed to send shutdown command to context channel");
                tx.send(&NodeMessage::ShutdownResult)?;
                break;
            }
        }
    }

    Ok(())
}

/// Error types generated by a tezos protocol.
#[derive(Fail, Debug)]
pub enum ProtocolError {
    /// Protocol rejected to apply a block.
    #[fail(display = "Apply block error: {}", reason)]
    ApplyBlockError {
        reason: ApplyBlockError
    },
    /// Error in configuration.
    #[fail(display = "OCaml runtime configuration error: {}", reason)]
    TezosRuntimeConfigurationError {
        reason: TezosRuntimeConfigurationError
    },
    /// OCaml part failed to initialize tezos storage.
    #[fail(display = "OCaml storage init error: {}", reason)]
    OcamlStorageInitError {
        reason: TezosStorageInitError
    },
    /// OCaml part failed to generate identity.
    #[fail(display = "Failed to generate tezos identity: {}", reason)]
    TezosGenerateIdentityError {
        reason: TezosGenerateIdentityError
    },
    /// OCaml part failed to get genesis data.
    #[fail(display = "Failed to get genesis data: {}", reason)]
    GenesisResultDataError {
        reason: GetDataError
    },
}

/// Errors generated by `protocol_runner`.
#[derive(Fail, Debug)]
pub enum ProtocolServiceError {
    /// Generic IPC communication error. See `reason` for more details.
    #[fail(display = "IPC error: {}", reason)]
    IpcError {
        reason: IpcError,
    },
    /// Tezos protocol error.
    #[fail(display = "Protocol error: {}", reason)]
    ProtocolError {
        reason: ProtocolError,
    },
    /// Unexpected message was received from IPC channel
    #[fail(display = "Received unexpected message: {}", message)]
    UnexpectedMessage {
        message: &'static str,
    },
    /// Tezedge node failed to spawn new `protocol_runner` sub-process.
    #[fail(display = "Failed to spawn tezos protocol wrapper sub-process: {}", reason)]
    SpawnError {
        reason: io::Error,
    },
    /// Invalid data error
    #[fail(display = "Invalid data error: {}", message)]
    InvalidDataError {
        message: String,
    },
}

impl slog::Value for ProtocolServiceError {
    fn serialize(&self, _record: &slog::Record, key: slog::Key, serializer: &mut dyn slog::Serializer) -> slog::Result {
        serializer.emit_arguments(key, &format_args!("{}", self))
    }
}

impl From<IpcError> for ProtocolServiceError {
    fn from(error: IpcError) -> Self {
        ProtocolServiceError::IpcError { reason: error }
    }
}

impl From<ProtocolError> for ProtocolServiceError {
    fn from(error: ProtocolError) -> Self {
        ProtocolServiceError::ProtocolError { reason: error }
    }
}

/// Protocol configuration (transferred via IPC from tezedge node to protocol_runner.
#[derive(Clone, Getters, CopyGetters)]
pub struct ProtocolEndpointConfiguration {
    #[get = "pub"]
    runtime_configuration: TezosRuntimeConfiguration,
    #[get = "pub"]
    environment: TezosEnvironmentConfiguration,
    #[get_copy = "pub"]
    enable_testchain: bool,
    #[get = "pub"]
    data_dir: PathBuf,
    #[get = "pub"]
    executable_path: PathBuf,
}

impl ProtocolEndpointConfiguration {
    pub fn new<P: AsRef<Path>>(runtime_configuration: TezosRuntimeConfiguration, environment: TezosEnvironmentConfiguration, enable_testchain: bool, data_dir: P, executable_path: P) -> Self {
        ProtocolEndpointConfiguration {
            runtime_configuration,
            environment,
            enable_testchain,
            data_dir: data_dir.as_ref().into(),
            executable_path: executable_path.as_ref().into(),
        }
    }
}

/// IPC command server is listening for incoming IPC connections.
pub struct IpcCmdServer(IpcServer<NodeMessage, ProtocolMessage>, ProtocolEndpointConfiguration);

/// Difference between `IpcCmdServer` and `IpcEvtServer` is:
/// * `IpcCmdServer` is used to create IPC channel over which commands from node are transferred to the protocol runner.
/// * `IpcEvtServer` is used to create IPC channel over which events are transmitted from protocol runner to the tezedge node.
impl IpcCmdServer {
    const IO_TIMEOUT: Duration = Duration::from_secs(10);

    /// Create new IPC endpoint
    pub fn new(configuration: ProtocolEndpointConfiguration) -> Self {
        IpcCmdServer(IpcServer::bind_path(&temp_sock()).unwrap(), configuration)
    }

    /// Start accepting incoming IPC connection.
    ///
    /// Returns a [`protocol controller`](ProtocolController) if new IPC channel is successfully created.
    /// This is a blocking operation.
    pub fn accept(&mut self) -> Result<ProtocolController, IpcError> {
        let (rx, tx) = self.0.accept()?;
        // configure IO timeouts
        rx.set_read_timeout(Some(Self::IO_TIMEOUT))
            .and(tx.set_write_timeout(Some(Self::IO_TIMEOUT)))
            .map_err(|err| IpcError::SocketConfigurationError { reason: err })?;

        Ok(ProtocolController {
            io: RefCell::new(IpcIO { rx, tx }),
            configuration: &self.1,
        })
    }
}

/// IPC event server is listening for incoming IPC connections.
pub struct IpcEvtServer(IpcServer<ContextAction, NoopMessage>);

/// Difference between `IpcCmdServer` and `IpcEvtServer` is:
/// * `IpcCmdServer` is used to create IPC channel over which commands from node are transferred to the protocol runner.
/// * `IpcEvtServer` is used to create IPC channel over which events are transmitted from protocol runner to the tezedge node.
impl IpcEvtServer {
    pub fn new() -> Self {
        IpcEvtServer(IpcServer::bind_path(&temp_sock()).unwrap())
    }

    /// Synchronously wait for new incoming IPC connection.
    pub fn accept(&mut self) -> Result<IpcReceiver<ContextAction>, IpcError> {
        let (rx, _) = self.0.accept()?;
        Ok(rx)
    }

    pub fn client_path(&self) -> PathBuf {
        self.0.client().path().to_path_buf()
    }
}

/// Endpoint consists of a protocol runner and IPC communication (command and event channels).
pub struct ProtocolRunnerEndpoint {
    pub runner: ProtocolRunner,
    pub commands: IpcCmdServer,
    pub events: IpcEvtServer,
}

impl ProtocolRunnerEndpoint {
    pub fn new(configuration: ProtocolEndpointConfiguration) -> ProtocolRunnerEndpoint {
        let protocol_runner_path = configuration.executable_path.clone();
        let evt_server = IpcEvtServer::new();
        let cmd_server = IpcCmdServer::new(configuration);
        ProtocolRunnerEndpoint {
            runner: ProtocolRunner::new(&protocol_runner_path, cmd_server.0.client().path(), evt_server.0.client().path()),
            commands: cmd_server,
            events: evt_server,
        }
    }
}

struct IpcIO {
    rx: IpcReceiver<NodeMessage>,
    tx: IpcSender<ProtocolMessage>,
}

/// Encapsulate IPC communication.
pub struct ProtocolController<'a> {
    io: RefCell<IpcIO>,
    configuration: &'a ProtocolEndpointConfiguration,
}

/// Provides convenience methods for IPC communication.
///
/// Instead of manually sending and receiving messages over IPC channel use provided methods.
/// Methods also handle things such as timeouts and also checks is correct response type is received.
impl<'a> ProtocolController<'a> {
    const GENERATE_IDENTITY_TIMEOUT: Duration = Duration::from_secs(600);
    const APPLY_BLOCK_TIMEOUT: Duration = Duration::from_secs(6000);

    /// Apply block
    pub fn apply_block(&self, chain_id: &Vec<u8>, block_header: &BlockHeader, predecessor_block_header: &BlockHeader, operations: &Vec<Option<OperationsForBlocksMessage>>, max_operations_ttl: u16) -> Result<ApplyBlockResponse, ProtocolServiceError> {
        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::ApplyBlockCall(ApplyBlockParams {
            chain_id: chain_id.clone(),
            block_header: block_header.clone(),
            predecessor_block_header: predecessor_block_header.clone(),
            operations: operations.clone(),
            max_operations_ttl,
        }))?;
        // this might take a while, so we will use unusually long timeout
        io.rx.set_read_timeout(Some(Self::APPLY_BLOCK_TIMEOUT)).map_err(|err| IpcError::SocketConfigurationError { reason: err })?;
        let receive_result = io.rx.receive();
        // restore default timeout setting
        io.rx.set_read_timeout(Some(IpcCmdServer::IO_TIMEOUT)).map_err(|err| IpcError::SocketConfigurationError { reason: err })?;
        match receive_result? {
            NodeMessage::ApplyBlockResult(result) => result.map_err(|err| ProtocolError::ApplyBlockError { reason: err }.into()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() })
        }
    }

    /// Change tezos runtime configuration
    pub fn change_runtime_configuration(&self, settings: TezosRuntimeConfiguration) -> Result<(), ProtocolServiceError> {
        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::ChangeRuntimeConfigurationCall(settings))?;
        match io.rx.receive()? {
            NodeMessage::ChangeRuntimeConfigurationResult(result) => result.map_err(|err| ProtocolError::TezosRuntimeConfigurationError { reason: err }.into()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() })
        }
    }

    /// Command tezos ocaml code to initialize context and protocol.
    /// CommitGenesisResult is returned only if commit_genesis is set to true
    fn init_protocol_context(&self, storage_data_dir: String, tezos_environment: &TezosEnvironmentConfiguration, commit_genesis: bool, enable_testchain: bool) -> Result<InitProtocolContextResult, ProtocolServiceError> {
        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::InitProtocolContextCall(InitProtocolContextParams {
            storage_data_dir,
            genesis: tezos_environment.genesis.clone(),
            genesis_max_operations_ttl: tezos_environment.genesis_additional_data().max_operations_ttl,
            protocol_overrides: tezos_environment.protocol_overrides.clone(),
            commit_genesis,
            enable_testchain,
        }))?;
        match io.rx.receive()? {
            NodeMessage::InitProtocolContextResult(result) => result.map_err(|err| ProtocolError::OcamlStorageInitError { reason: err }.into()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() })
        }
    }

    /// Command tezos ocaml code to generate a new identity.
    pub fn generate_identity(&self, expected_pow: f64) -> Result<Identity, ProtocolServiceError> {
        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::GenerateIdentity(GenerateIdentityParams {
            expected_pow,
        }))?;
        // this might take a while, so we will use unusually long timeout
        io.rx.set_read_timeout(Some(Self::GENERATE_IDENTITY_TIMEOUT)).map_err(|err| IpcError::SocketConfigurationError { reason: err })?;
        let receive_result = io.rx.receive();
        // restore default timeout setting
        io.rx.set_read_timeout(Some(IpcCmdServer::IO_TIMEOUT)).map_err(|err| IpcError::SocketConfigurationError { reason: err })?;
        match receive_result? {
            NodeMessage::GenerateIdentityResult(result) => result.map_err(|err| ProtocolError::TezosGenerateIdentityError { reason: err }.into()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() })
        }
    }

    /// Gracefully shutdown protocol runner
    pub fn shutdown(&self) -> Result<(), ProtocolServiceError> {
        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::ShutdownCall)?;
        match io.rx.receive()? {
            NodeMessage::ShutdownResult => Ok(()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() }),
        }
    }

    /// Initialize protocol environment from default configuration.
    pub fn init_protocol(&self, commit_genesis: bool) -> Result<InitProtocolContextResult, ProtocolServiceError> {
        self.change_runtime_configuration(self.configuration.runtime_configuration().clone())?;
        self.init_protocol_context(
            self.configuration.data_dir().to_str().unwrap().to_string(),
            self.configuration.environment(),
            commit_genesis,
            self.configuration.enable_testchain(),
        )
    }

    /// Gets data for genesis.
    pub fn genesis_result_data(&self, genesis_context_hash: &ContextHash) -> Result<CommitGenesisResult, ProtocolServiceError> {
        let tezos_environment = self.configuration.environment();
        let main_chain_id = tezos_environment.main_chain_id().map_err(|e| ProtocolServiceError::InvalidDataError { message: format!("{:?}", e)})?;
        let protocol_hash = tezos_environment.genesis_protocol().map_err(|e| ProtocolServiceError::InvalidDataError { message: format!("{:?}", e)})?;

        let mut io = self.io.borrow_mut();
        io.tx.send(&ProtocolMessage::GenesisResultDataCall(GenesisResultDataParams {
            genesis_context_hash: genesis_context_hash.clone(),
            chain_id: main_chain_id,
            genesis_protocol_hash: protocol_hash,
            genesis_max_operations_ttl: tezos_environment.genesis_additional_data().max_operations_ttl,
        }))?;
        match io.rx.receive()? {
            NodeMessage::CommitGenesisResultData(result) => result.map_err(|err| ProtocolError::GenesisResultDataError { reason: err }.into()),
            message => Err(ProtocolServiceError::UnexpectedMessage { message: message.into() })
        }
    }
}

impl Drop for ProtocolController<'_> {
    fn drop(&mut self) {
        // try to gracefully shutdown protocol runner
        let _ = self.shutdown();
    }
}

/// Control protocol runner sub-process.
pub struct ProtocolRunner {
    sock_cmd_path: PathBuf,
    sock_evt_path: PathBuf,
    executable_path: PathBuf,
}

impl ProtocolRunner {
    const PROCESS_WAIT_TIMEOUT: Duration = Duration::from_secs(4);

    pub fn new<P: AsRef<Path>>(executable_path: P, sock_cmd_path: &Path, sock_evt_path: &Path) -> Self {
        ProtocolRunner {
            sock_cmd_path: sock_cmd_path.to_path_buf(),
            sock_evt_path: sock_evt_path.to_path_buf(),
            executable_path: executable_path.as_ref().to_path_buf(),
        }
    }

    pub fn spawn(&self) -> Result<Child, ProtocolServiceError> {
        let process = Command::new(&self.executable_path)
            .arg("--sock-cmd")
            .arg(&self.sock_cmd_path)
            .arg("--sock-evt")
            .arg(&self.sock_evt_path)
            .spawn()
            .map_err(|err| ProtocolServiceError::SpawnError { reason: err })?;
        Ok(process)
    }

    pub fn terminate(mut process: Child) {
        match process.wait_timeout(Self::PROCESS_WAIT_TIMEOUT).unwrap() {
            Some(_) => (),
            None => {
                // child hasn't exited yet
                let _ = process.kill();
            }
        };
    }

    pub fn is_running(process: &mut Child) -> bool {
        match process.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

