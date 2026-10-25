//! Adaptateur MCP : expose les outils de [`crate::outils`] et [`crate::jeu`].

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use insmv_campagne::Campagne;
use insmv_regles::personnage::Role;
use insmv_regles::DesAleatoires;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::jeu::{
    self, ChercherPouvoirParams, CreationParams, LireJournalParams, NoterParams, TestParams,
    TirerPouvoirParams,
};
use crate::outils::{self, ActionParams};

/// Emplacement par défaut de la base : `campagne/jeu.sqlite` à la racine du projet.
pub fn chemin_campagne_par_defaut() -> PathBuf {
    match std::env::var_os("INSMV_CAMPAGNE") {
        Some(chemin) => PathBuf::from(chemin),
        None => PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../campagne/jeu.sqlite")),
    }
}

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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ConflitPersonnagesParams {
    /// Test de l'attaquant (lancé en premier).
    pub attaquant: TestParams,
    /// Test du défenseur.
    pub defenseur: TestParams,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NomParams {
    /// Nom du personnage (ou son identifiant).
    pub nom: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeltaParams {
    /// Nom du personnage (ou son identifiant).
    pub nom: String,
    /// Variation entière : positive ou négative selon l'outil.
    pub delta: i32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListerParams {
    /// Ne lister que ce rôle : joueur, partenaire ou pnj.
    #[serde(default)]
    pub role: Option<Role>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RequeteParams {
    /// Texte contenu dans le nom ou la catégorie (ex. « combat », « esquive »).
    pub requete: String,
}

#[derive(Clone)]
pub struct ServeurMj {
    tool_router: ToolRouter<Self>,
    campagne: Arc<Mutex<Campagne>>,
}

impl ServeurMj {
    pub fn new(campagne: Campagne) -> Self {
        Self { tool_router: Self::tool_router(), campagne: Arc::new(Mutex::new(campagne)) }
    }

    /// Exécute une opération sur la campagne et renvoie son résultat en JSON.
    fn avec<T: serde::Serialize>(
        &self,
        f: impl FnOnce(&Campagne) -> Result<T, String>,
    ) -> Result<String, String> {
        let campagne = self.campagne.lock().map_err(|_| "base de campagne verrouillée".to_string())?;
        en_json(f(&campagne))
    }
}

fn en_json<T: serde::Serialize>(valeur: Result<T, String>) -> Result<String, String> {
    valeur.and_then(|v| serde_json::to_string_pretty(&v).map_err(|e| e.to_string()))
}

#[tool_router]
impl ServeurMj {
    // ---- Dés et résolution ----

    #[tool(description = "Lance des dés à six faces (D6, 2D6+3, D66, D666) et renvoie les faces et le total. \
                          À utiliser pour tout tirage qui n'est pas un test sur la table UM (dommages libres, tables aléatoires...).")]
    async fn lancer_des(&self, Parameters(p): Parameters<LancerDesParams>) -> Result<String, String> {
        en_json(outils::lancer_des(&p.notation, &mut DesAleatoires::default()))
    }

    #[tool(description = "Résout une action ad hoc sur la table unique multiple (table UM), sans fiche : lance un D666, \
                          lit le seuil selon la caractéristique, le niveau de talent et les décalages de colonnes, \
                          et renvoie réussite/échec et unités. Pour un personnage enregistré, préférer tester_personnage.")]
    async fn resoudre_action(&self, Parameters(p): Parameters<ActionParams>) -> Result<String, String> {
        en_json(outils::resoudre_action(&p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Résout un conflit ad hoc entre un attaquant et un défenseur, sans fiche : deux jets sur la table UM, \
                          puis comparaison des unités si les deux réussissent. En cas d'unités égales, \
                          l'issue est « arbitrage_mj » : le maître de jeu doit trancher.")]
    async fn resoudre_conflit(&self, Parameters(p): Parameters<ConflitParams>) -> Result<String, String> {
        en_json(outils::resoudre_conflit(&p.attaquant, &p.defenseur, &mut DesAleatoires::default()))
    }

    #[tool(description = "Teste un personnage enregistré sur la table UM à partir de sa fiche : caractéristique + talent ou pouvoir \
                          (ou difficulté seule). Un talent absent de la fiche utilise la ligne 0 avec le malus du catalogue. \
                          `cout_pp` prélève les points de pouvoir avant le jet. Les unités sont à interpréter par le MJ.")]
    async fn tester_personnage(&self, Parameters(p): Parameters<TestParams>) -> Result<String, String> {
        self.avec(|c| jeu::tester_personnage(c, &p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Résout un conflit entre deux personnages enregistrés (attaquant lancé en premier). \
                          Valide les deux tests avant de prélever les PP. Unités égales = « arbitrage_mj ».")]
    async fn conflit_personnages(
        &self,
        Parameters(p): Parameters<ConflitPersonnagesParams>,
    ) -> Result<String, String> {
        self.avec(|c| jeu::conflit_personnages(c, &p.attaquant, &p.defenseur, &mut DesAleatoires::default()))
    }

    // ---- Fiches ----

    #[tool(description = "Crée un personnage (joueur, partenaire ou PNJ) après contrôle des budgets de création de son espèce \
                          (caractéristiques, talents, pouvoirs). Un dépassement de budget est refusé sauf `exception` motivée ; \
                          une fiche incohérente (pouvoir inconnu...) est toujours refusée. Calcule les PP (VO + tirages de pouvoir).")]
    async fn creer_personnage(&self, Parameters(p): Parameters<CreationParams>) -> Result<String, String> {
        self.avec(|c| jeu::creer_personnage(c, p))
    }

    #[tool(description = "Affiche la fiche complète d'un personnage : caractéristiques, talents, pouvoirs détaillés (catalogue), PP, dommages, budget de l'espèce.")]
    async fn voir_personnage(&self, Parameters(p): Parameters<NomParams>) -> Result<String, String> {
        self.avec(|c| jeu::voir_personnage(c, &p.nom))
    }

    #[tool(description = "Liste les personnages de la campagne (nom, espèce, rôle, PP, dommages), éventuellement filtrés par rôle.")]
    async fn lister_personnages(&self, Parameters(p): Parameters<ListerParams>) -> Result<String, String> {
        self.avec(|c| jeu::lister_personnages(c, p.role))
    }

    #[tool(description = "Supprime définitivement un personnage. Irréversible : à n'utiliser que sur demande explicite du joueur.")]
    async fn supprimer_personnage(&self, Parameters(p): Parameters<NomParams>) -> Result<String, String> {
        self.avec(|c| jeu::supprimer_personnage(c, &p.nom))
    }

    #[tool(description = "Ajuste les points de pouvoir d'un personnage : delta négatif = dépense (refusée si insuffisant), \
                          positif = récupération (plafonnée au maximum).")]
    async fn ajuster_pp(&self, Parameters(p): Parameters<DeltaParams>) -> Result<String, String> {
        self.avec(|c| jeu::ajuster_pp(c, &p.nom, p.delta))
    }

    #[tool(description = "Ajuste les points de dommages d'un personnage : delta positif = blessure, négatif = soin (jamais sous 0). \
                          Les effets des blessures (perte de force, malus) ne sont pas encore appliqués automatiquement.")]
    async fn ajuster_dommages(&self, Parameters(p): Parameters<DeltaParams>) -> Result<String, String> {
        self.avec(|c| jeu::ajuster_dommages(c, &p.nom, p.delta))
    }

    // ---- Catalogues ----

    #[tool(description = "Cherche un pouvoir dans la table du livre (ange = Magna Veritas, demon = In Nomine Satanis) \
                          par code à trois chiffres ou par texte (nom, catégorie, famille). Renvoie caractéristiques d'attaque/défense et coût en PP.")]
    async fn chercher_pouvoir(&self, Parameters(p): Parameters<ChercherPouvoirParams>) -> Result<String, String> {
        en_json(jeu::chercher_pouvoir(&p))
    }

    #[tool(description = "Tire un pouvoir sur la table du livre : méthode aléatoire (D666) ou semi-aléatoire (le joueur choisit \
                          les centaines, un D66 donne le reste). À utiliser à la création ou à l'acquisition d'un pouvoir.")]
    async fn tirer_pouvoir(&self, Parameters(p): Parameters<TirerPouvoirParams>) -> Result<String, String> {
        en_json(jeu::tirer_pouvoir(&p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Cherche un talent dans le catalogue (nom ou catégorie) : malus colonnes si absent de la fiche et caractéristiques conseillées.")]
    async fn chercher_talent(&self, Parameters(p): Parameters<RequeteParams>) -> Result<String, String> {
        en_json(Ok(jeu::chercher_talent(&p.requete)))
    }

    // ---- Journal ----

    #[tool(description = "Ajoute une entrée au journal de campagne (scène jouée, PNJ rencontré, lieu, intrigue, décision de règle...). \
                          Le journal ne se réécrit pas : une correction est une nouvelle entrée.")]
    async fn noter_journal(&self, Parameters(p): Parameters<NoterParams>) -> Result<String, String> {
        self.avec(|c| jeu::noter_journal(c, p))
    }

    #[tool(description = "Relit le journal de campagne (les plus récentes en dernier), filtrable par séance, catégorie, texte ou personnage. \
                          À consulter en début de séance pour retrouver le fil de l'histoire.")]
    async fn lire_journal(&self, Parameters(p): Parameters<LireJournalParams>) -> Result<String, String> {
        self.avec(|c| jeu::lire_journal(c, p))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ServeurMj {}
