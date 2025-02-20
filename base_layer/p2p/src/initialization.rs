//  Copyright 2019 The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#![allow(dead_code)]

use std::{
    fs::File,
    iter,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use fs2::FileExt;
use futures::future;
use lmdb_zero::open;
use log::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tari_common::{configuration::Network, DnsNameServer};
use tari_comms::{
    backoff::ConstantBackoff,
    multiaddr::Multiaddr,
    peer_manager::{NodeIdentity, Peer, PeerFeatures, PeerManagerError},
    pipeline,
    protocol::{
        messaging::{MessagingEventSender, MessagingProtocolExtension},
        rpc::RpcServer,
        NodeNetworkInfo,
    },
    tor,
    tor::HiddenServiceControllerError,
    transports::{MemoryTransport, SocksTransport, TcpWithTorTransport},
    utils::cidr::parse_cidrs,
    CommsBuilder,
    CommsBuilderError,
    CommsNode,
    PeerManager,
    UnspawnedCommsNode,
};
use tari_comms_dht::{Dht, DhtConfig, DhtInitializationError, DhtProtocolVersion};
use tari_service_framework::{async_trait, ServiceInitializationError, ServiceInitializer, ServiceInitializerContext};
use tari_shutdown::ShutdownSignal;
use tari_storage::{
    lmdb_store::{LMDBBuilder, LMDBConfig},
    LMDBWrapper,
};
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use tower::ServiceBuilder;

use crate::{
    comms_connector::{InboundDomainConnector, PubsubDomainConnector},
    peer_seeds::{DnsSeedResolver, SeedPeer},
    transport::{TorConfig, TransportType},
    MAJOR_NETWORK_VERSION,
    MINOR_NETWORK_VERSION,
};

const LOG_TARGET: &str = "p2p::initialization";

#[derive(Debug, Error)]
pub enum CommsInitializationError {
    #[error("Comms builder error: `{0}`")]
    CommsBuilderError(#[from] CommsBuilderError),
    #[error("Failed to initialize tor hidden service: {0}")]
    HiddenServiceControllerError(#[from] HiddenServiceControllerError),
    #[error("DHT initialization error: `{0}`")]
    DhtInitializationError(#[from] DhtInitializationError),
    #[error("Hidden service builder error: `{0}`")]
    HiddenServiceBuilderError(#[from] tor::HiddenServiceBuilderError),
    #[error("Invalid liveness CIDRs error: `{0}`")]
    InvalidLivenessCidrs(String),
    #[error("Could not add seed peers to comms layer: `{0}`")]
    FailedToAddSeedPeer(#[from] PeerManagerError),
    #[error("Cannot acquire exclusive file lock, another instance of the application is already running")]
    CannotAcquireFileLock,
    #[error("IO Error: `{0}`")]
    IoError(#[from] std::io::Error),
}

impl CommsInitializationError {
    pub fn to_friendly_string(&self) -> String {
        // Add any helpful user-facing messages here
        match self {
            CommsInitializationError::HiddenServiceControllerError(HiddenServiceControllerError::TorControlPortOffline)
                 => r#"Unable to connect to the Tor control port.
    Please check that you have the Tor proxy running and that access to the Tor control port is turned on.
    If you are unsure of what to do, use the following command to start the Tor proxy:
    tor --allow-missing-torrc --ignore-missing-torrc --clientonly 1 --socksport 9050 --controlport 127.0.0.1:9051 --log "warn stdout" --clientuseipv6 1"#
                    .to_string(),
            err => format!("Failed to initialize comms: {:?}", err),
        }
    }
}

/// Configuration for a comms node
#[derive(Clone)]
pub struct P2pConfig {
    /// Path to the LMDB data files.
    pub datastore_path: PathBuf,
    /// Name to use for the peer database
    pub peer_database_name: String,
    /// The maximum number of concurrent Inbound tasks allowed before back-pressure is applied to peers
    pub max_concurrent_inbound_tasks: usize,
    /// The maximum number of concurrent outbound tasks allowed before back-pressure is applied to outbound messaging
    /// queue
    pub max_concurrent_outbound_tasks: usize,
    /// The size of the buffer (channel) which holds pending outbound message requests
    pub outbound_buffer_size: usize,
    /// Configuration for DHT
    pub dht: DhtConfig,
    /// The p2p network currently being connected to.
    pub network: Network,
    /// The identity of this node on the network
    pub node_identity: Arc<NodeIdentity>,
    /// The type of transport to use
    pub transport_type: TransportType,
    /// Set to true to allow peers to provide test addresses (loopback, memory etc.). If set to false, memory
    /// addresses, loopback, local-link (i.e addresses used in local tests) will not be accepted from peers. This
    /// should always be false for non-test nodes.
    pub allow_test_addresses: bool,
    /// The maximum number of liveness sessions allowed for the connection listener.
    /// Liveness sessions can be used by third party tooling to determine node liveness.
    /// A value of 0 will disallow any liveness sessions.
    pub listener_liveness_max_sessions: usize,
    /// CIDR for addresses allowed to enter into liveness check mode on the listener.
    pub listener_liveness_allowlist_cidrs: Vec<String>,
    /// User agent string for this node
    pub user_agent: String,
    /// Unparsed peer seeds
    pub peer_seeds: Vec<String>,
    /// DNS seeds hosts. The DNS TXT records are queried from these hosts and the resulting peers added to the comms
    /// peer list.
    pub dns_seeds: Vec<String>,
    /// DNS name server to use for DNS seeds.
    pub dns_seeds_name_server: DnsNameServer,
    /// All DNS seed records must pass DNSSEC validation
    pub dns_seeds_use_dnssec: bool,
    /// The address to bind on using the TCP transport _in addition to_ the primary transport. This is typically useful
    /// for direct comms between a wallet and base node. If this is set to None, no listener will be bound.
    /// Default: None
    pub auxilary_tcp_listener_address: Option<Multiaddr>,
}

/// Initialize Tari Comms configured for tests
pub async fn initialize_local_test_comms(
    node_identity: Arc<NodeIdentity>,
    connector: InboundDomainConnector,
    data_path: &str,
    discovery_request_timeout: Duration,
    seed_peers: Vec<Peer>,
    shutdown_signal: ShutdownSignal,
) -> Result<(CommsNode, Dht, MessagingEventSender), CommsInitializationError> {
    let peer_database_name = {
        let mut rng = thread_rng();
        iter::repeat(())
            .map(|_| rng.sample(Alphanumeric) as char)
            .take(8)
            .collect::<String>()
    };
    std::fs::create_dir_all(data_path).unwrap();
    let datastore = LMDBBuilder::new()
        .set_path(data_path)
        .set_env_flags(open::NOLOCK)
        .set_env_config(LMDBConfig::default())
        .set_max_number_of_databases(1)
        .add_database(&peer_database_name, lmdb_zero::db::CREATE)
        .build()
        .unwrap();
    let peer_database = datastore.get_handle(&peer_database_name).unwrap();
    let peer_database = LMDBWrapper::new(Arc::new(peer_database));

    //---------------------------------- Comms --------------------------------------------//

    let comms = CommsBuilder::new()
        .allow_test_addresses()
        .with_listener_address(node_identity.public_address())
        .with_listener_liveness_max_sessions(1)
        .with_node_identity(node_identity)
        .with_user_agent("/test/1.0")
        .with_peer_storage(peer_database, None)
        .with_dial_backoff(ConstantBackoff::new(Duration::from_millis(500)))
        .with_min_connectivity(1)
        .with_shutdown_signal(shutdown_signal)
        .build()?;

    add_all_peers(&comms.peer_manager(), &comms.node_identity(), seed_peers).await?;

    // Create outbound channel
    let (outbound_tx, outbound_rx) = mpsc::channel(10);

    let dht = Dht::builder()
        .local_test()
        .with_outbound_sender(outbound_tx)
        .with_discovery_timeout(discovery_request_timeout)
        .build(
            comms.node_identity(),
            comms.peer_manager(),
            comms.connectivity(),
            comms.shutdown_signal(),
        )
        .await?;

    let dht_outbound_layer = dht.outbound_middleware_layer();
    let (event_sender, _) = broadcast::channel(100);
    let pipeline = pipeline::Builder::new()
        .outbound_buffer_size(10)
        .with_outbound_pipeline(outbound_rx, |sink| {
            ServiceBuilder::new().layer(dht_outbound_layer).service(sink)
        })
        .max_concurrent_inbound_tasks(10)
        .with_inbound_pipeline(
            ServiceBuilder::new()
                .layer(dht.inbound_middleware_layer())
                .service(connector),
        )
        .build();

    let comms = comms
        .add_protocol_extension(MessagingProtocolExtension::new(event_sender.clone(), pipeline))
        .spawn_with_transport(MemoryTransport)
        .await?;

    Ok((comms, dht, event_sender))
}

pub async fn spawn_comms_using_transport(
    comms: UnspawnedCommsNode,
    transport_type: TransportType,
) -> Result<CommsNode, CommsInitializationError> {
    let comms = match transport_type {
        TransportType::Memory { listener_address } => {
            debug!(target: LOG_TARGET, "Building in-memory comms stack");
            comms
                .with_listener_address(listener_address)
                .spawn_with_transport(MemoryTransport)
                .await?
        },
        TransportType::Tcp {
            listener_address,
            tor_socks_config,
        } => {
            debug!(
                target: LOG_TARGET,
                "Building TCP comms stack{}",
                tor_socks_config.as_ref().map(|_| " with Tor support").unwrap_or("")
            );
            let mut transport = TcpWithTorTransport::new();
            if let Some(config) = tor_socks_config {
                transport.set_tor_socks_proxy(config);
            }
            comms
                .with_listener_address(listener_address)
                .spawn_with_transport(transport)
                .await?
        },
        TransportType::Tor(tor_config) => {
            debug!(target: LOG_TARGET, "Building TOR comms stack ({})", tor_config);
            let mut hidden_service_ctl = initialize_hidden_service(tor_config).await?;
            // Set the listener address to be the address (usually local) to which tor will forward all traffic
            let transport = hidden_service_ctl.initialize_transport().await?;
            debug!(target: LOG_TARGET, "Comms and DHT configured");
            comms
                .with_listener_address(hidden_service_ctl.proxied_address())
                .with_hidden_service_controller(hidden_service_ctl)
                .spawn_with_transport(transport)
                .await?
        },
        TransportType::Socks {
            socks_config,
            listener_address,
        } => {
            debug!(target: LOG_TARGET, "Building SOCKS5 comms stack");
            let transport = SocksTransport::new(socks_config);
            comms
                .with_listener_address(listener_address)
                .spawn_with_transport(transport)
                .await?
        },
    };

    Ok(comms)
}

async fn initialize_hidden_service(
    config: TorConfig,
) -> Result<tor::HiddenServiceController, tor::HiddenServiceBuilderError> {
    let mut builder = tor::HiddenServiceBuilder::new()
        .with_hs_flags(tor::HsFlags::DETACH)
        .with_port_mapping(config.port_mapping)
        .with_socks_address_override(config.socks_address_override)
        .with_socks_authentication(config.socks_auth)
        .with_control_server_auth(config.control_server_auth)
        .with_control_server_address(config.control_server_addr)
        .with_bypass_proxy_addresses(config.tor_proxy_bypass_addresses.into());

    if config.tor_proxy_bypass_for_outbound_tcp {
        builder = builder.bypass_tor_for_tcp_addresses();
    }

    if let Some(identity) = config.identity {
        builder = builder.with_tor_identity(*identity);
    }

    builder.build().await
}

async fn configure_comms_and_dht(
    builder: CommsBuilder,
    config: &P2pConfig,
    connector: InboundDomainConnector,
) -> Result<(UnspawnedCommsNode, Dht), CommsInitializationError> {
    let file_lock = acquire_exclusive_file_lock(&config.datastore_path)?;

    let datastore = LMDBBuilder::new()
        .set_path(&config.datastore_path)
        .set_env_flags(open::NOLOCK)
        .set_env_config(LMDBConfig::default())
        .set_max_number_of_databases(1)
        .add_database(&config.peer_database_name, lmdb_zero::db::CREATE)
        .build()
        .unwrap();
    let peer_database = datastore.get_handle(&config.peer_database_name).unwrap();
    let peer_database = LMDBWrapper::new(Arc::new(peer_database));

    let listener_liveness_allowlist_cidrs = parse_cidrs(&config.listener_liveness_allowlist_cidrs)
        .map_err(CommsInitializationError::InvalidLivenessCidrs)?;

    let builder = builder
        .with_listener_liveness_max_sessions(config.listener_liveness_max_sessions)
        .with_listener_liveness_allowlist_cidrs(listener_liveness_allowlist_cidrs)
        .with_dial_backoff(ConstantBackoff::new(Duration::from_millis(500)))
        .with_peer_storage(peer_database, Some(file_lock));

    let mut comms = match config.auxilary_tcp_listener_address {
        Some(ref addr) => builder.with_auxilary_tcp_listener_address(addr.clone()).build()?,
        None => builder.build()?,
    };

    let peer_manager = comms.peer_manager();
    let connectivity = comms.connectivity();
    let node_identity = comms.node_identity();
    let shutdown_signal = comms.shutdown_signal();
    // Create outbound channel
    let (outbound_tx, outbound_rx) = mpsc::channel(config.outbound_buffer_size);

    let mut dht = Dht::builder();
    dht.with_config(config.dht.clone()).with_outbound_sender(outbound_tx);
    // TODO: remove this once enough weatherwax nodes have upgraded
    if config.network == Network::Weatherwax {
        dht.with_protocol_version(DhtProtocolVersion::v1());
    }
    let dht = dht
        .build(node_identity.clone(), peer_manager, connectivity, shutdown_signal)
        .await?;

    let dht_outbound_layer = dht.outbound_middleware_layer();

    // DHT RPC service is only available for communication nodes
    if node_identity.has_peer_features(PeerFeatures::COMMUNICATION_NODE) {
        comms = comms.add_rpc_server(RpcServer::new().add_service(dht.rpc_service()));
    }

    // Hook up DHT messaging middlewares
    // TODO: messaging events should be optional
    let (messaging_events_sender, _) = broadcast::channel(1);
    let messaging_pipeline = pipeline::Builder::new()
        .outbound_buffer_size(config.outbound_buffer_size)
        .with_outbound_pipeline(outbound_rx, |sink| {
            ServiceBuilder::new().layer(dht_outbound_layer).service(sink)
        })
        .max_concurrent_inbound_tasks(config.max_concurrent_inbound_tasks)
        .max_concurrent_outbound_tasks(config.max_concurrent_outbound_tasks)
        .with_inbound_pipeline(
            ServiceBuilder::new()
                .layer(dht.inbound_middleware_layer())
                .service(connector),
        )
        .build();

    comms = comms.add_protocol_extension(MessagingProtocolExtension::new(
        messaging_events_sender,
        messaging_pipeline,
    ));

    Ok((comms, dht))
}

/// Acquire an exclusive OS level write lock on a file in the provided path. This is used to check if another instance
/// of this database has already been initialized in order to prevent two process from using it simultaneously
/// ## Parameters
/// `db_path` - Path where the db will be initialized
///
/// ## Returns
/// Returns a File handle that must be retained to keep the file lock active.
fn acquire_exclusive_file_lock(db_path: &Path) -> Result<File, CommsInitializationError> {
    let lock_file_path = db_path.join(".p2p_file.lock");

    let file = File::create(lock_file_path)?;
    // Attempt to acquire exclusive OS level Write Lock
    if let Err(e) = file.try_lock_exclusive() {
        error!(
            target: LOG_TARGET,
            "Could not acquire exclusive write lock on database lock file: {:?}", e
        );
        return Err(CommsInitializationError::CannotAcquireFileLock);
    }

    Ok(file)
}

/// Adds a new peer to the base node
/// ## Parameters
/// `comms_node` - A reference to the comms node. This is the communications stack
/// `peers` - A list of peers to be added to the comms node, the current node identity of the comms stack is excluded if
/// found in the list.
///
/// ## Returns
/// A Result to determine if the call was successful or not, string will indicate the reason on error
async fn add_all_peers(
    peer_manager: &PeerManager,
    node_identity: &NodeIdentity,
    peers: Vec<Peer>,
) -> Result<(), CommsInitializationError> {
    for peer in peers {
        if &peer.public_key == node_identity.public_key() {
            debug!(
                target: LOG_TARGET,
                "Attempting to add yourself [{}] as a seed peer to comms layer, ignoring request", peer
            );
            continue;
        }

        debug!(target: LOG_TARGET, "Adding seed peer [{}]", peer);
        peer_manager
            .add_peer(peer)
            .await
            .map_err(CommsInitializationError::FailedToAddSeedPeer)?;
    }
    Ok(())
}

pub struct P2pInitializer {
    config: P2pConfig,
    connector: Option<PubsubDomainConnector>,
}

impl P2pInitializer {
    pub fn new(config: P2pConfig, connector: PubsubDomainConnector) -> Self {
        Self {
            config,
            connector: Some(connector),
        }
    }

    // Following are inlined due to Rust ICE: https://github.com/rust-lang/rust/issues/73537
    #[inline(always)]
    fn try_parse_seed_peers(peer_seeds_str: &[String]) -> Result<Vec<Peer>, ServiceInitializationError> {
        peer_seeds_str
            .iter()
            .map(|s| SeedPeer::from_str(s))
            .map(|r| r.map(Peer::from))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    async fn try_resolve_dns_seeds(
        resolver_addr: DnsNameServer,
        dns_seeds: &[String],
        use_dnssec: bool,
    ) -> Result<Vec<Peer>, ServiceInitializationError> {
        if dns_seeds.is_empty() {
            debug!(target: LOG_TARGET, "No DNS Seeds configured");
            return Ok(Vec::new());
        }

        debug!(target: LOG_TARGET, "Resolving DNS seeds...");
        let start = Instant::now();

        let resolver = if use_dnssec {
            debug!(
                target: LOG_TARGET,
                "Using {} to resolve DNS seeds. DNSSEC is enabled", resolver_addr
            );
            DnsSeedResolver::connect_secure(resolver_addr).await?
        } else {
            debug!(
                target: LOG_TARGET,
                "Using {} to resolve DNS seeds. DNSSEC is disabled", resolver_addr
            );
            DnsSeedResolver::connect(resolver_addr).await?
        };
        let resolving = dns_seeds.iter().map(|addr| {
            let mut resolver = resolver.clone();
            async move { (resolver.resolve(addr).await, addr) }
        });

        let peers = future::join_all(resolving)
            .await
            .into_iter()
            // Log and ignore errors
            .filter_map(|(result, addr)| match result {
                Ok(peers) => {
                    debug!(
                        target: LOG_TARGET,
                        "Found {} peer(s) from `{}` in {:.0?}",
                        peers.len(),
                        addr,
                        start.elapsed()
                    );
                    Some(peers)
                },
                Err(err) => {
                    warn!(target: LOG_TARGET, "DNS seed `{}` failed to resolve: {}", addr, err);
                    None
                },
            })
            .flatten()
            .map(Into::into)
            .collect::<Vec<_>>();

        Ok(peers)
    }
}

#[async_trait]
impl ServiceInitializer for P2pInitializer {
    async fn initialize(&mut self, context: ServiceInitializerContext) -> Result<(), ServiceInitializationError> {
        let mut config = self.config.clone();
        let connector = self.connector.take().expect("P2pInitializer called more than once");

        let mut builder = CommsBuilder::new()
            .with_shutdown_signal(context.get_shutdown_signal())
            .with_node_identity(config.node_identity.clone())
            .with_node_info(NodeNetworkInfo {
                major_version: MAJOR_NETWORK_VERSION,
                minor_version: MINOR_NETWORK_VERSION,
                network_byte: config.network.as_byte(),
                user_agent: config.user_agent.clone(),
            });

        if config.allow_test_addresses {
            builder = builder.allow_test_addresses();
        }
        // Ensure this setting always matches
        config.dht.allow_test_addresses = config.allow_test_addresses;

        let (comms, dht) = configure_comms_and_dht(builder, &config, connector).await?;

        let peer_manager = comms.peer_manager();
        let node_identity = comms.node_identity();

        let peers = match Self::try_resolve_dns_seeds(
            config.dns_seeds_name_server,
            &config.dns_seeds,
            config.dns_seeds_use_dnssec,
        )
        .await
        {
            Ok(peers) => peers,
            Err(err) => {
                warn!(target: LOG_TARGET, "Failed to resolve DNS seeds: {}", err);
                Vec::new()
            },
        };
        add_all_peers(&peer_manager, &node_identity, peers).await?;

        let peers = Self::try_parse_seed_peers(&config.peer_seeds)?;
        add_all_peers(&peer_manager, &node_identity, peers).await?;

        context.register_handle(comms.connectivity());
        context.register_handle(peer_manager);
        context.register_handle(comms);
        context.register_handle(dht);

        Ok(())
    }
}
