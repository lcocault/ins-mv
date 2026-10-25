use insmv_mcp::serveur::ServeurMj;
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // stdout est réservé au protocole MCP : tout log doit aller sur stderr.
    let service = ServeurMj::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
