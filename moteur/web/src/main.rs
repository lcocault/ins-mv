use std::net::SocketAddr;
use std::path::PathBuf;

use insmv_web::{routeur, Etat};

fn chemin_campagne() -> PathBuf {
    match std::env::var_os("INSMV_CAMPAGNE") {
        Some(c) => PathBuf::from(c),
        None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../campagne/jeu.sqlite")),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = std::env::var("INSMV_WEB_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8787);
    // Local uniquement : aucune authentification, donc pas d'exposition réseau.
    let adresse = SocketAddr::from(([127, 0, 0, 1], port));
    let chemin = chemin_campagne();
    eprintln!("insmv-web : campagne {} — http://localhost:{port}", chemin.display());
    let ecoute = tokio::net::TcpListener::bind(adresse).await?;
    axum::serve(ecoute, routeur(Etat::new(chemin))).await?;
    Ok(())
}
