//! Routes HTTP et accès à la campagne ainsi qu'à l'agent de jeu.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Form, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::Router;
use insmv_campagne::{Campagne, ErreurCampagne, FiltreJournal};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::rendu::{self, FiltresAffiches, MessageChat};

const LIMITE_MESSAGES: usize = 200;
const LIMITE_MESSAGE_OCTETS: usize = 8_000;
const DELAI_AGENT: Duration = Duration::from_secs(120);

#[derive(Default)]
struct Conversation {
    session_id: Option<String>,
    messages: Vec<MessageChat>,
}

struct Agent {
    executable: OsString,
    racine: PathBuf,
}

#[derive(Clone)]
pub struct Etat {
    base: Arc<PathBuf>,
    conversation: Arc<Mutex<Conversation>>,
    agent: Arc<Agent>,
}

impl Etat {
    pub fn new(chemin: PathBuf) -> Self {
        let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let executable =
            std::env::var_os("INSMV_CLAUDE").unwrap_or_else(|| OsString::from("claude"));
        Self {
            base: Arc::new(chemin),
            conversation: Arc::new(Mutex::new(Conversation::default())),
            agent: Arc::new(Agent { executable, racine }),
        }
    }

    /// Ouvre la base à chaque requête : la lecture seule est peu coûteuse et ne
    /// retient aucun verrou entre deux pages.
    fn campagne(&self) -> Result<Campagne, (StatusCode, Html<String>)> {
        Campagne::ouvrir_lecture_seule(&self.base).map_err(|e| {
            erreur(
                StatusCode::SERVICE_UNAVAILABLE,
                "Campagne indisponible",
                &format!("Impossible d'ouvrir {} : {e}. La campagne est créée au premier lancement du serveur MCP.", self.base.display()),
            )
        })
    }

    async fn envoyer(&self, message: &str) {
        let mut conversation = self.conversation.lock().await;
        conversation.messages.push(MessageChat {
            joueur: true,
            contenu: message.to_string(),
        });
        match self
            .agent
            .repondre(message, conversation.session_id.as_deref())
            .await
        {
            Ok((reponse, session_id)) => {
                conversation.session_id = Some(session_id);
                conversation.messages.push(MessageChat {
                    joueur: false,
                    contenu: reponse,
                });
            }
            Err(e) => conversation.messages.push(MessageChat {
                joueur: false,
                contenu: format!("Le maître de jeu n'a pas pu répondre : {e}"),
            }),
        }
        if conversation.messages.len() > LIMITE_MESSAGES {
            let excedent = conversation.messages.len() - LIMITE_MESSAGES;
            conversation.messages.drain(..excedent);
        }
    }
}

impl Agent {
    async fn repondre(
        &self,
        message: &str,
        session_id: Option<&str>,
    ) -> Result<(String, String), String> {
        let mut commande = Command::new(&self.executable);
        commande
            .current_dir(&self.racine)
            .arg("--print")
            .arg("--agent")
            .arg("maitre-de-jeu")
            .arg("--output-format")
            .arg("json")
            .arg("--permission-mode")
            .arg("dontAsk")
            .arg("--allowedTools")
            .arg("Read")
            .arg("Grep")
            .arg("Glob")
            .arg("mcp__mj__*")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(session_id) = session_id {
            commande.arg("--resume").arg(session_id);
        }
        let mut enfant = commande.spawn().map_err(|e| {
            format!("impossible de lancer Claude Code ({e}). Vérifiez son installation ou INSMV_CLAUDE.")
        })?;
        if let Some(mut entree) = enfant.stdin.take() {
            entree
                .write_all(message.as_bytes())
                .await
                .map_err(|e| format!("envoi du message impossible : {e}"))?;
        }
        let sortie = tokio::time::timeout(DELAI_AGENT, enfant.wait_with_output())
            .await
            .map_err(|_| "délai de réponse dépassé (120 s)".to_string())?
            .map_err(|e| format!("lecture de la réponse impossible : {e}"))?;
        if !sortie.status.success() {
            return Err(format!(
                "Claude Code a quitté avec le statut {}",
                sortie.status
            ));
        }
        let resultat: serde_json::Value = serde_json::from_slice(&sortie.stdout)
            .map_err(|e| format!("réponse de Claude Code illisible : {e}"))?;
        let reponse = resultat
            .get("result")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "réponse du maître de jeu absente".to_string())?;
        let session_id = resultat
            .get("session_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "identifiant de conversation absent".to_string())?;
        Ok((reponse.to_string(), session_id.to_string()))
    }
}

type Reponse = Result<Html<String>, (StatusCode, Html<String>)>;

fn erreur(statut: StatusCode, titre: &str, message: &str) -> (StatusCode, Html<String>) {
    (statut, Html(rendu::page_erreur(titre, message)))
}

fn interne(e: ErreurCampagne) -> (StatusCode, Html<String>) {
    erreur(StatusCode::INTERNAL_SERVER_ERROR, "Erreur de lecture", &e.to_string())
}

pub fn routeur(etat: Etat) -> Router {
    Router::new()
        .route("/", get(accueil))
        .route("/partie", get(partie))
        .route("/chat", post(chat))
        .route("/personnage/{id}", get(personnage))
        .route("/journal", get(journal))
        .with_state(etat)
}

async fn accueil(State(etat): State<Etat>) -> Reponse {
    let personnages = etat.campagne()?.lister(None).map_err(interne)?;
    Ok(Html(rendu::liste(&personnages)))
}

async fn personnage(State(etat): State<Etat>, Path(id): Path<String>) -> Reponse {
    match etat.campagne()?.charger(&id) {
        Ok(p) => Ok(Html(rendu::fiche(&p))),
        Err(ErreurCampagne::Introuvable(_)) => Err(erreur(
            StatusCode::NOT_FOUND,
            "Personnage introuvable",
            &format!("Aucun personnage « {id} » dans la campagne."),
        )),
        Err(e) => Err(interne(e)),
    }
}

/// Jets récents listés sous la table UM.
const JETS_RECENTS: usize = 6;

async fn partie(State(etat): State<Etat>) -> Reponse {
    let campagne = etat.campagne()?;
    let resolution = campagne.derniere_resolution().map_err(interne)?;
    let jets = campagne.derniers_jets(JETS_RECENTS).map_err(interne)?;
    let combat = campagne.combat_en_cours().map_err(interne)?;
    let personnages = campagne.lister(None).map_err(interne)?;
    let messages = etat.conversation.lock().await.messages.clone();
    Ok(Html(rendu::partie(
        resolution.as_ref(),
        &jets,
        combat.as_ref(),
        &personnages,
        &messages,
    )))
}

#[derive(Debug, Deserialize)]
struct ParamsChat {
    message: String,
}

async fn chat(
    State(etat): State<Etat>,
    headers: HeaderMap,
    Form(params): Form<ParamsChat>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    let origine = headers.get("origin").and_then(|v| v.to_str().ok());
    let hote = headers.get("host").and_then(|v| v.to_str().ok());
    let origine_locale = match (
        origine,
        hote.and_then(|h| h.parse::<axum::http::uri::Authority>().ok()),
    ) {
        (Some(o), Some(h)) if matches!(h.host(), "localhost" | "127.0.0.1") => {
            o == format!("http://{h}")
        }
        _ => false,
    };
    if !origine_locale {
        return Err(erreur(
            StatusCode::FORBIDDEN,
            "Requête refusée",
            "L'origine de la requête n'est pas autorisée.",
        ));
    }
    let message = params.message.trim();
    if message.is_empty() || message.len() > LIMITE_MESSAGE_OCTETS {
        return Err(erreur(
            StatusCode::BAD_REQUEST,
            "Message invalide",
            "Le message doit contenir entre 1 et 8 000 octets.",
        ));
    }
    etat.envoyer(message).await;
    Ok(Redirect::to("/partie"))
}

#[derive(Debug, Default, Deserialize)]
struct ParamsJournal {
    seance: Option<String>,
    categorie: Option<String>,
    contient: Option<String>,
    personnage: Option<String>,
}

fn non_vide(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Nombre d'entrées de journal affichées.
const LIMITE_JOURNAL: usize = 100;

async fn journal(State(etat): State<Etat>, Query(q): Query<ParamsJournal>) -> Reponse {
    let campagne = etat.campagne()?;
    let filtre = FiltreJournal {
        seance: non_vide(q.seance),
        categorie: non_vide(q.categorie),
        contient: non_vide(q.contient),
        personnage: non_vide(q.personnage),
        limite: LIMITE_JOURNAL,
    };
    let affiches = FiltresAffiches {
        seance: filtre.seance.clone().unwrap_or_default(),
        categorie: filtre.categorie.clone().unwrap_or_default(),
        contient: filtre.contient.clone().unwrap_or_default(),
        personnage: filtre.personnage.clone().unwrap_or_default(),
    };
    let entrees = campagne.lire_journal(&filtre).map_err(interne)?;
    let personnages = campagne.lister(None).map_err(interne)?;
    let nom_de = |id: &str| {
        personnages.iter().find(|p| p.id == id).map(|p| p.nom.clone()).unwrap_or_else(|| id.to_string())
    };
    Ok(Html(rendu::journal(&entrees, &nom_de, &personnages, &affiches)))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn reponse_reutilise_la_session_claude() {
        let dossier = std::env::temp_dir().join(format!(
            "insmv-agent-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dossier).unwrap();
        let executable = dossier.join("claude-factice");
        std::fs::write(
            &executable,
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> arguments.log\ncat >> messages.log\nprintf '\\n' >> messages.log\nprintf '%s' '{\"result\":\"Réponse\",\"session_id\":\"session-test\"}'\n",
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&executable, permissions).unwrap();
        let agent = Agent {
            executable: executable.into_os_string(),
            racine: dossier.clone(),
        };

        let (premiere, session) = agent.repondre("Premier message", None).await.unwrap();
        let (seconde, _) = agent
            .repondre("Deuxième message", Some(&session))
            .await
            .unwrap();

        assert_eq!(premiere, "Réponse");
        assert_eq!(seconde, "Réponse");
        let arguments = std::fs::read_to_string(dossier.join("arguments.log")).unwrap();
        assert!(arguments.contains("--agent maitre-de-jeu"));
        assert!(arguments.contains("--allowedTools Read Grep Glob mcp__mj__*"));
        assert!(arguments.contains("--resume session-test"));
        assert_eq!(
            std::fs::read_to_string(dossier.join("messages.log")).unwrap(),
            "Premier message\nDeuxième message\n"
        );
        std::fs::remove_dir_all(dossier).unwrap();
    }
}
