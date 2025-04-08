use crate::controller::{Controller, TOOLS};
use anyhow::Result;
use rmcp::{
    Error as RmcpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, JsonObject, JsonRpcMessage, ListToolsResult, PaginatedRequestParam,
        ServerInfo,
    },
    service::RequestContext,
};
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

/// Message type for requests that can be sent through the transport
#[derive(Debug)]
pub enum MCPRequest {
    /// Get server info
    GetInfo,
    /// Call a tool with parameters
    CallTool(CallToolRequestParam),
    /// List available tools
    ListTools(PaginatedRequestParam),
}

/// Response types that can be returned from the transport
#[derive(Debug)]
pub enum MCPResponse {
    /// Server info
    ServerInfo(ServerInfo),
    /// Result of calling a tool
    CallToolResult(rmcp::model::CallToolResult),
    /// Result of listing tools
    ListToolsResult(ListToolsResult),
}

/// Message to be sent through the mpsc channel
#[derive(Debug)]
pub struct MpscTransportMessage {
    /// The request to process
    pub request: MCPRequest,
    /// The oneshot channel to send the response back
    pub response_tx: oneshot::Sender<Result<MCPResponse, RmcpError>>,
}

/// A transport for RMCP that uses tokio mpsc channels
pub struct MpscTransport {
    controller: Controller,
    rx: mpsc::Receiver<MpscTransportMessage>,
}

impl MpscTransport {
    /// Create a new tokio transport with the given controller
    pub fn new(controller: Controller) -> (Self, mpsc::Sender<MpscTransportMessage>) {
        let (tx, rx) = mpsc::channel(100); // Buffer size of 100
        (Self { controller, rx }, tx)
    }

    /// Spawn a task to handle incoming messages and return a client
    pub fn spawn(self) -> MpscClient {
        let (tx, _) = mpsc::channel::<MpscTransportMessage>(100);
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            self.run().await.unwrap_or_else(|e| {
                error!("Error running MPSC transport: {}", e);
            });
        });

        MpscClient::new(tx_clone)
    }

    /// Run the transport, processing messages from the channel
    pub async fn run(mut self) -> Result<()> {
        info!("Tokio transport started");

        while let Some(message) = self.rx.recv().await {
            let controller = self.controller.clone();
            let response_tx = message.response_tx;

            // Process the request
            let request = message.request;
            debug!("Received request: {:?}", request);

            // Handle the request using a different approach that avoids the problematic types
            let result = match request {
                MCPRequest::CallTool(call) => {
                    // Extract the tool name and arguments
                    let tool_name = call.name.as_ref();
                    let args = call.arguments.unwrap_or_default();

                    // Directly use the Controller methods based on the tool name
                    match tool_name {
                        "firecrawl_search" => match controller.search(args).await {
                            Ok(text) => {
                                let result = rmcp::model::CallToolResult::success(vec![
                                    rmcp::model::Content::text(text),
                                ]);
                                Ok(MCPResponse::CallToolResult(result))
                            }
                            Err(e) => Err(e),
                        },
                        "firecrawl_scrape" => match controller.scrape(args).await {
                            Ok(text) => {
                                let result = rmcp::model::CallToolResult::success(vec![
                                    rmcp::model::Content::text(text),
                                ]);
                                Ok(MCPResponse::CallToolResult(result))
                            }
                            Err(e) => Err(e),
                        },
                        "firecrawl_map" => match controller.map(args).await {
                            Ok(text) => {
                                let result = rmcp::model::CallToolResult::success(vec![
                                    rmcp::model::Content::text(text),
                                ]);
                                Ok(MCPResponse::CallToolResult(result))
                            }
                            Err(e) => Err(e),
                        },
                        "firecrawl_crawl" => match controller.crawl(args).await {
                            Ok(text) => {
                                let result = rmcp::model::CallToolResult::success(vec![
                                    rmcp::model::Content::text(text),
                                ]);
                                Ok(MCPResponse::CallToolResult(result))
                            }
                            Err(e) => Err(e),
                        },
                        "firecrawl_batch_scrape" => match controller.batch_scrape(args).await {
                            Ok(text) => {
                                let result = rmcp::model::CallToolResult::success(vec![
                                    rmcp::model::Content::text(text),
                                ]);
                                Ok(MCPResponse::CallToolResult(result))
                            }
                            Err(e) => Err(e),
                        },
                        _ => Err(RmcpError::invalid_request(
                            format!("Tool not found: {}", tool_name),
                            None,
                        )),
                    }
                }
                MCPRequest::ListTools(_) => {
                    // Use the Arc<Vec<Tool>> directly, no need to clone the vec itself
                    let tools_result = ListToolsResult {
                        tools: Vec::from(TOOLS.as_ref()),
                        next_cursor: None,
                    };
                    Ok(MCPResponse::ListToolsResult(tools_result))
                }
                MCPRequest::GetInfo => {
                    let info = controller.get_info();
                    Ok(MCPResponse::ServerInfo(info))
                }
            };

            // Send the response
            if let Err(e) = response_tx.send(result) {
                error!("Failed to send response: {:?}", e);
            }
        }

        info!("Tokio transport stopped");
        Ok(())
    }
}

/// Client for interacting with the MCP service through MPSC channels
pub struct MpscClient {
    sender: mpsc::Sender<MpscTransportMessage>,
}

impl MpscClient {
    /// Create a new client with the given sender
    fn new(sender: mpsc::Sender<MpscTransportMessage>) -> Self {
        Self { sender }
    }

    /// Get server info
    pub async fn get_info(&self) -> Result<ServerInfo, RmcpError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(MpscTransportMessage {
                request: MCPRequest::GetInfo,
                response_tx: tx,
            })
            .await
            .map_err(|_| RmcpError::internal_error("Failed to send request".to_string(), None))?;

        match rx.await {
            Ok(result) => match result {
                Ok(MCPResponse::ServerInfo(info)) => Ok(info),
                Ok(_) => Err(RmcpError::internal_error(
                    "Unexpected response type".to_string(),
                    None,
                )),
                Err(e) => Err(e),
            },
            Err(_) => Err(RmcpError::internal_error(
                "Failed to receive response".to_string(),
                None,
            )),
        }
    }

    /// Call a tool with the given name and parameters
    pub async fn call_tool(
        &self,
        tool_name: &str,
        params: JsonObject,
    ) -> Result<rmcp::model::CallToolResult, RmcpError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(MpscTransportMessage {
                request: MCPRequest::CallTool(CallToolRequestParam {
                    name: tool_name.to_string().into(),
                    arguments: Some(params),
                }),
                response_tx: tx,
            })
            .await
            .map_err(|_| RmcpError::internal_error("Failed to send request".to_string(), None))?;

        match rx.await {
            Ok(result) => match result {
                Ok(MCPResponse::CallToolResult(result)) => Ok(result),
                Ok(_) => Err(RmcpError::internal_error(
                    "Unexpected response type".to_string(),
                    None,
                )),
                Err(e) => Err(e),
            },
            Err(_) => Err(RmcpError::internal_error(
                "Failed to receive response".to_string(),
                None,
            )),
        }
    }

    /// List available tools
    pub async fn list_tools(&self) -> Result<ListToolsResult, RmcpError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(MpscTransportMessage {
                request: MCPRequest::ListTools(PaginatedRequestParam::default()),
                response_tx: tx,
            })
            .await
            .map_err(|_| RmcpError::internal_error("Failed to send request".to_string(), None))?;

        match rx.await {
            Ok(result) => match result {
                Ok(MCPResponse::ListToolsResult(result)) => Ok(result),
                Ok(_) => Err(RmcpError::internal_error(
                    "Unexpected response type".to_string(),
                    None,
                )),
                Err(e) => Err(e),
            },
            Err(_) => Err(RmcpError::internal_error(
                "Failed to receive response".to_string(),
                None,
            )),
        }
    }
}

/// Extension trait for spawning an MPSC transport with a Controller
pub trait MpscServiceExt {
    /// Spawn an MPSC transport service and return a client
    fn spawn_mpsc_client(self) -> MpscClient;
}

impl MpscServiceExt for Controller {
    fn spawn_mpsc_client(self) -> MpscClient {
        let transport = MpscTransport::new(self);
        transport.0.spawn()
    }
}
