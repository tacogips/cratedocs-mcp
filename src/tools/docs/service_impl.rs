use std::future::Future;
use rmcp::{
    Error as McpError, RoleServer, ServerHandler, model::*,
    service::{RequestContext, Peer},
};
use crate::tools::docs::docs::CargoDocRouter;

impl rmcp::Service<RoleServer> for CargoDocRouter {
    fn handle_request(
        &self,
        request: <RoleServer as rmcp::ServiceRole>::PeerReq,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<<RoleServer as rmcp::ServiceRole>::Resp, McpError>> + Send + '_ {
        async move {
            match request {
                ClientRequest::CallTool(param) => self.call_tool(param, context).await,
                ClientRequest::GetToolSpec(_) => self.get_tool_spec().await,
                ClientRequest::ListResources(param) => self.list_resources(param, context).await,
                ClientRequest::ReadResource(param) => self.read_resource(param, context).await,
                ClientRequest::ListPrompts(param) => self.list_prompts(param, context).await,
                ClientRequest::GetPrompt(param) => self.get_prompt(param, context).await,
                ClientRequest::ListResourceTemplates(param) => self.list_resource_templates(param, context).await,
            }
        }
    }

    fn handle_notification(
        &self,
        notification: <RoleServer as rmcp::ServiceRole>::PeerNot,
    ) -> impl Future<Output = Result<(), McpError>> + Send + '_ {
        async move {
            match notification {
                ClientNotification::Cancelled(_) => Ok(()),
            }
        }
    }

    fn get_peer(&self) -> Option<Peer<RoleServer>> {
        None
    }

    fn set_peer(&mut self, _peer: Peer<RoleServer>) {
        // Store peer if needed
    }

    fn get_info(&self) -> <RoleServer as rmcp::ServiceRole>::Info {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Rust Documentation MCP Server for accessing Rust crate documentation.".to_string()),
        }
    }
}

impl ServerHandler for CargoDocRouter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("Rust Documentation MCP Server for accessing Rust crate documentation.".to_string()),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        })
    }

    async fn read_resource(
        &self,
        _param: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Err(McpError::resource_not_found(
            "resource_not_supported",
            None,
        ))
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![],
        })
    }

    async fn get_prompt(
        &self,
        _param: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        Err(McpError::invalid_params("prompt not supported", None))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
        })
    }
}