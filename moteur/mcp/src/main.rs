use insmv_campagne::Campagne;
use insmv_mcp::serveur::{chemin_campagne_par_defaut, ServeurMj};
use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // stdout est réservé au protocole MCP : tout log doit aller sur stderr.
    let chemin = chemin_campagne_par_defaut();
    eprintln!("insmv-mcp : campagne dans {}", chemin.display());
    let service = ServeurMj::new(Campagne::ouvrir(&chemin)?).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
