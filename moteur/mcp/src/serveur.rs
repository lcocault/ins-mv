//! Adaptateur MCP : expose les outils de [`crate::outils`] et [`crate::jeu`].

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use insmv_campagne::{Campagne, JetUm};
use insmv_regles::personnage::Role;
use insmv_regles::DesAleatoires;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::jeu::{
    self, AttaqueParams, ChercherPouvoirParams, InitiativeParams, UsagePouvoirParams, CreationParams, LireJournalParams, NoterParams, TestParams,
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
pub struct SemainesParams {
    /// Nom du personnage (ou son identifiant).
    pub nom: String,
    /// Nombre de semaines de repos total.
    pub semaines: u32,
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
        let sortie = outils::resoudre_action(&p, &mut DesAleatoires::default())?;
        let action = p.vers_action().map_err(|e| e.to_string())?;
        self.avec(|c| {
            jeu::afficher_jets(c, vec![JetUm { libelle: "Jet sans fiche".into(), action, jet: sortie.jet }]);
            Ok(&sortie)
        })
    }

    #[tool(description = "Résout un conflit ad hoc entre un attaquant et un défenseur, sans fiche : deux jets sur la table UM, \
                          puis comparaison des unités si les deux réussissent. En cas d'unités égales, \
                          l'issue est « arbitrage_mj » : le maître de jeu doit trancher.")]
    async fn resoudre_conflit(&self, Parameters(p): Parameters<ConflitParams>) -> Result<String, String> {
        let sortie = outils::resoudre_conflit(&p.attaquant, &p.defenseur, &mut DesAleatoires::default())?;
        let (action_a, action_d) = (
            p.attaquant.vers_action().map_err(|e| e.to_string())?,
            p.defenseur.vers_action().map_err(|e| e.to_string())?,
        );
        self.avec(|c| {
            jeu::afficher_jets(
                c,
                vec![
                    JetUm { libelle: "Attaquant (sans fiche)".into(), action: action_a, jet: sortie.attaquant.jet },
                    JetUm { libelle: "Défenseur (sans fiche)".into(), action: action_d, jet: sortie.defenseur.jet },
                ],
            );
            Ok(&sortie)
        })
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

    #[tool(description = "Résout une attaque physique entre deux personnages enregistrés : jet d'attaque, défense (Esquive par défaut, \
                          parade avec une arme possible), conflit, puis dommages = unités finales + puissance de l'arme + bonus de Force (contact, FO ≥ 4) \
                          - protection (armure, pouvoir Armure corporelle). Applique les dommages au défenseur et renvoie son état de santé. \
                          Unités égales = « arbitrage_mj » : rien n'est appliqué, c'est au MJ de trancher. Les notes du livre sur l'arme sont transmises.")]
    async fn attaquer(&self, Parameters(p): Parameters<AttaqueParams>) -> Result<String, String> {
        self.avec(|c| jeu::attaquer(c, &p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Utilise un pouvoir de la fiche : prélève son coût en PP (déduit du livre, ou `cout_pp` s'il est variable), \
                          lance le jet avec la caractéristique du pouvoir (Volonté, Précision...) et son niveau, et renvoie la règle du livre à appliquer. \
                          Avec `cible`, la cible se défend (caractéristique de défense du livre, niveau 0) : conflit résolu, unités comparées. \
                          Les effets (durée en minutes, distance...) se déduisent des unités : à interpréter par le MJ en suivant la règle renvoyée.")]
    async fn utiliser_pouvoir(&self, Parameters(p): Parameters<UsagePouvoirParams>) -> Result<String, String> {
        self.avec(|c| jeu::utiliser_pouvoir(c, &p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Ordre d'action d'un tour de combat : phase psychique par Volonté décroissante (égalités au D66), \
                          puis phase physique par Agilité décroissante (mêmes valeurs : simultané). Écarte les inconscients et les morts.")]
    async fn initiative(&self, Parameters(p): Parameters<InitiativeParams>) -> Result<String, String> {
        self.avec(|c| jeu::initiative(c, &p, &mut DesAleatoires::default()))
    }

    #[tool(description = "Annonce une action sans la lancer : la table UM de l'interface Web montre la ligne et la colonne applicables, \
                          et ce que donnerait chaque décalage de -3 à +3 colonnes (seuil et chances de réussite). À appeler avant \
                          tester_personnage quand le joueur doit décider d'un décalage de colonnes (malus volontaire, circonstances). \
                          Mêmes paramètres que tester_personnage ; ne prélève rien et ne lance aucun dé.")]
    async fn annoncer_action(&self, Parameters(p): Parameters<TestParams>) -> Result<String, String> {
        self.avec(|c| jeu::annoncer_action(c, &p))
    }

    #[tool(description = "Termine le combat affiché dans l'interface Web (ordre d'action et fil des actions). \
                          À appeler quand le combat est fini : le prochain appel d'initiative ouvre un nouveau combat au tour 1.")]
    async fn terminer_combat(&self) -> Result<String, String> {
        self.avec(jeu::terminer_combat)
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

    #[tool(description = "Inflige (delta > 0) ou soigne (delta < 0) des points de dommages. Renvoie l'état de santé selon l'espèce : Force perdue, \
                          évanouissement/coma/mort, malus de blessure (humains, soldats, familiers), et tire la durée d'inconscience à la bascule. \
                          Les malus de blessure sont appliqués automatiquement aux tests ; un inconscient ou un mort ne peut plus agir.")]
    async fn ajuster_dommages(&self, Parameters(p): Parameters<DeltaParams>) -> Result<String, String> {
        self.avec(|c| jeu::ajuster_dommages(c, &p.nom, p.delta, &mut DesAleatoires::default()))
    }

    #[tool(description = "Soins naturels : chaque semaine de repos total rend un point de Force à un personnage (1 à 3 points de dommage selon l'espèce).")]
    async fn soigner_semaines(&self, Parameters(p): Parameters<SemainesParams>) -> Result<String, String> {
        self.avec(|c| jeu::soigner_semaines(c, &p.nom, p.semaines))
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
