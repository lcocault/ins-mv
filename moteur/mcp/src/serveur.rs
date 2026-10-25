//! Adaptateur MCP : expose les outils de [`crate::outils`].

use insmv_regles::DesAleatoires;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::outils::{self, ActionParams};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LancerDesParams {
    /// Notation du jet : `D6`, `2D6`, `1D6+3`, `D66` ou `D666`.
    pub notation: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConflitParams {
    /// Action de l'attaquant (celui qui tente l'action).
    pub attaquant: ActionParams,
    /// Action du défenseur (celui qui s'y oppose).
    pub defenseur: ActionParams,
}

#[derive(Clone)]
pub struct ServeurMj {
    tool_router: ToolRouter<Self>,
}

impl ServeurMj {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }
}

impl Default for ServeurMj {
    fn default() -> Self {
        Self::new()
    }
}

fn en_json<T: serde::Serialize>(valeur: Result<T, String>) -> Result<String, String> {
    valeur.and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
}

#[tool_router]
impl ServeurMj {
    #[tool(description = "Lance des dés à six faces (D6, 2D6+3, D66, D666) et renvoie les faces et le total. \
                          À utiliser pour tout tirage qui n'est pas un test sur la table UM (dommages libres, tables aléatoires...).")]
    async fn lancer_des(&self, Parameters(p): Parameters<LancerDesParams>) -> Result<String, String> {
        en_json(outils::lancer_des(&p.notation, &mut DesAleatoires::default()))
    }

    #[tool(description = "Résout une action sur la table unique multiple (table UM) : lance un D666, \
                          lit le seuil selon la caractéristique, le niveau de talent et les décalages de colonnes, \
                          et renvoie réussite/échec et unités. Les unités qualifient le résultat et sont à interpréter par le MJ.")]
    async fn resoudre_action(&self, Parameters(p): Parameters<ActionParams>) -> Result<String, String> {
        en_json(outils::resoudre_action(&p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Résout un conflit entre un attaquant et un défenseur : deux jets sur la table UM, \
                          puis comparaison des unités si les deux réussissent. En cas d'unités égales, \
                          l'issue est « arbitrage_mj » : le maître de jeu doit trancher.")]
    async fn resoudre_conflit(&self, Parameters(p): Parameters<ConflitParams>) -> Result<String, String> {
        en_json(outils::resoudre_conflit(&p.attaquant, &p.defenseur, &mut DesAleatoires::default()))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ServeurMj {}
