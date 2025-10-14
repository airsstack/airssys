//! # NetworkActor
//!
//! Centralized actor for network operations with connection pooling.
//!
//! ## Responsibilities
//!
//! - TCP connection management
//! - UDP socket management
//! - Connection pooling and reuse
//! - Connection lifecycle tracking
//! - Centralized network audit logging

use std::collections::HashMap;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::actor::{Actor, ActorContext, ErrorAction};
use crate::broker::MessageBroker;
use crate::message::{MessageEnvelope, Message};
use crate::supervisor::{Child, ChildHealth};

use super::messages::{
    ConnectionId, ConnectionState, NetworkError, NetworkOperation, NetworkRequest, NetworkResponse,
    NetworkResult, SocketId,
};

/// NetworkActor - Centralized network operations
///
/// This actor serves as the interface between the actor runtime and
/// network operations. All application actors should send messages
/// to this actor rather than creating network connections directly.
///
/// ## Benefits
///
/// - Centralized audit logging for all network operations
/// - Connection pooling and reuse
/// - Automatic cleanup when actor stops
/// - Clean fault isolation (network failures don't crash app actors)
/// - Superior testability (mock this actor in tests)
///
/// ## Generic Parameters
///
/// - `M`: Unified message type that wraps NetworkRequest/NetworkResponse (must implement `Message + From<NetworkResponse>`)
/// - `B`: Message broker for publishing responses (must implement `MessageBroker<M>`)
pub struct NetworkActor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    /// Message broker for publishing responses
    broker: B,

    /// Phantom data for message type
    _phantom: PhantomData<M>,

    /// Active TCP connections
    active_connections: HashMap<ConnectionId, ConnectionHandle>,

    /// Active UDP sockets
    active_sockets: HashMap<SocketId, SocketHandle>,

    /// Connection ID counter
    next_connection_id: ConnectionId,

    /// Socket ID counter
    next_socket_id: SocketId,

    /// Operation counter for metrics
    operation_count: u64,

    /// Actor creation timestamp
    created_at: DateTime<Utc>,

    /// Last operation timestamp
    last_operation_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ConnectionHandle {
    connection_id: ConnectionId,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    connected_at: DateTime<Utc>,
    state: ConnectionState,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SocketHandle {
    socket_id: SocketId,
    local_addr: SocketAddr,
    bound_at: DateTime<Utc>,
}

impl<M, B> NetworkActor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    /// Create a new NetworkActor with the given broker
    ///
    /// # Arguments
    ///
    /// * `broker` - Message broker for publishing responses
    pub fn new(broker: B) -> Self {
        Self {
            broker,
            _phantom: PhantomData,
            active_connections: HashMap::new(),
            active_sockets: HashMap::new(),
            next_connection_id: 1,
            next_socket_id: 1,
            operation_count: 0,
            created_at: Utc::now(),
            last_operation_at: None,
        }
    }

    /// Get active connection count
    pub fn active_connection_count(&self) -> usize {
        self.active_connections.len()
    }

    /// Get active socket count
    pub fn active_socket_count(&self) -> usize {
        self.active_sockets.len()
    }

    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    /// Close all connections and sockets
    async fn close_all(&mut self) {
        self.active_connections.clear();
        self.active_sockets.clear();
    }

    /// Execute network operation and return result
    async fn execute_operation(&mut self, operation: NetworkOperation) -> NetworkResult {
        self.operation_count += 1;
        self.last_operation_at = Some(Utc::now());

        match operation {
            NetworkOperation::TcpConnect { addr, timeout: _ } => {
                // TODO: Integrate with airssys-osl for actual TCP connection
                let connection_id = self.next_connection_id;
                self.next_connection_id += 1;

                self.active_connections.insert(
                    connection_id,
                    ConnectionHandle {
                        connection_id,
                        local_addr: addr,
                        remote_addr: addr,
                        connected_at: Utc::now(),
                        state: ConnectionState::Connected,
                    },
                );

                NetworkResult::TcpConnectSuccess {
                    connection_id,
                    local_addr: addr,
                    remote_addr: addr,
                }
            }
            NetworkOperation::TcpDisconnect { connection_id } => {
                if self.active_connections.remove(&connection_id).is_some() {
                    NetworkResult::TcpDisconnectSuccess { connection_id }
                } else {
                    NetworkResult::Error {
                        error: NetworkError::NotFound { connection_id },
                    }
                }
            }
            NetworkOperation::UdpBind { addr } => {
                // TODO: Integrate with airssys-osl for actual UDP binding
                let socket_id = self.next_socket_id;
                self.next_socket_id += 1;

                self.active_sockets.insert(
                    socket_id,
                    SocketHandle {
                        socket_id,
                        local_addr: addr,
                        bound_at: Utc::now(),
                    },
                );

                NetworkResult::UdpBindSuccess {
                    socket_id,
                    local_addr: addr,
                }
            }
            NetworkOperation::UdpClose { socket_id } => {
                if self.active_sockets.remove(&socket_id).is_some() {
                    NetworkResult::UdpCloseSuccess { socket_id }
                } else {
                    NetworkResult::Error {
                        error: NetworkError::Other {
                            message: format!("Socket {socket_id} not found"),
                        },
                    }
                }
            }
            NetworkOperation::GetConnectionStatus { connection_id } => {
                if let Some(handle) = self.active_connections.get(&connection_id) {
                    NetworkResult::ConnectionStatus {
                        connection_id,
                        state: handle.state,
                    }
                } else {
                    NetworkResult::Error {
                        error: NetworkError::NotFound { connection_id },
                    }
                }
            }
        }
    }
}

#[async_trait]
impl<M, B> Actor for NetworkActor<M, B>
where
    M: Message + serde::Serialize + for<'de> serde::Deserialize<'de> + From<NetworkResponse> + 'static,
    B: MessageBroker<M> + Clone + Send + Sync + 'static,
{
    type Message = NetworkRequest;
    type Error = NetworkError;

    async fn handle_message<Broker: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, Broker>,
    ) -> Result<(), Self::Error> {
        // Execute operation
        let result = self.execute_operation(message.operation).await;

        // Create response
        let response = NetworkResponse {
            request_id: message.request_id,
            result,
        };

        // Wrap response in unified message type and create envelope
        let unified_message = M::from(response);
        let envelope = MessageEnvelope::new(unified_message)
            .with_reply_to(message.reply_to);

        // Publish via broker (ADR-RT-009: Broker Dependency Injection)
        self.broker
            .publish(envelope)
            .await
            .map_err(|_| NetworkError::Other {
                message: "Failed to publish response".to_string(),
            })?;

        Ok(())
    }

    async fn on_error<Broker: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, Broker>,
    ) -> ErrorAction {
        eprintln!("NetworkActor error: {error:?}");
        ErrorAction::Resume
    }
}

#[async_trait]
impl<M, B> Child for NetworkActor<M, B>
where
    M: Message + 'static,
    B: MessageBroker<M> + 'static,
{
    type Error = NetworkError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("NetworkActor starting at {}", self.created_at);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // CRITICAL: Close all connections and sockets
        if !self.active_connections.is_empty() || !self.active_sockets.is_empty() {
            println!(
                "NetworkActor stopping with {} connections and {} sockets. Closing all...",
                self.active_connections.len(),
                self.active_sockets.len()
            );
            self.close_all().await;
        }
        println!(
            "NetworkActor stopped. Total operations: {}, active connections: {}, active sockets: {}",
            self.operation_count,
            self.active_connections.len(),
            self.active_sockets.len()
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        // Check if connection pool is too large
        if self.active_connections.len() > 100 {
            ChildHealth::Degraded(format!(
                "Too many active connections: {}",
                self.active_connections.len()
            ))
        } else {
            ChildHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::InMemoryMessageBroker;

    use serde::{Deserialize, Serialize};

    /// Test message type that wraps NetworkRequest/NetworkResponse
    #[derive(Debug, Clone, Serialize, Deserialize)]
    enum TestOSLMessage {
        NetworkReq(NetworkRequest),
        NetworkResp(NetworkResponse),
    }

    impl Message for TestOSLMessage {
        const MESSAGE_TYPE: &'static str = "test_osl_message";
    }

    impl From<NetworkResponse> for TestOSLMessage {
        fn from(response: NetworkResponse) -> Self {
            TestOSLMessage::NetworkResp(response)
        }
    }

    #[test]
    fn test_network_actor_new() {
        let broker = InMemoryMessageBroker::<TestOSLMessage>::new();
        let actor = NetworkActor::new(broker);
        assert_eq!(actor.active_connection_count(), 0);
        assert_eq!(actor.active_socket_count(), 0);
        assert_eq!(actor.operation_count(), 0);
    }

    #[tokio::test]
    async fn test_network_actor_health_check() {
        let broker = InMemoryMessageBroker::<TestOSLMessage>::new();
        let actor = NetworkActor::new(broker);
        let health = actor.health_check().await;
        assert_eq!(health, ChildHealth::Healthy);
    }

    #[tokio::test]
    async fn test_network_actor_health_degraded() {
        let broker = InMemoryMessageBroker::<TestOSLMessage>::new();
        let mut actor = NetworkActor::new(broker);

        // Add many connections to trigger degraded state
        for i in 0..101 {
            let addr = format!("127.0.0.1:{}", 8000 + i)
                .parse()
                .expect("valid socket address");
            actor.active_connections.insert(
                i,
                ConnectionHandle {
                    connection_id: i,
                    local_addr: addr,
                    remote_addr: addr,
                    connected_at: Utc::now(),
                    state: ConnectionState::Connected,
                },
            );
        }

        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Degraded(_)));
    }
}
