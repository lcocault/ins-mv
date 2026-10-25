//! Routes HTTP (lecture seule) et accès à la base.

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use insmv_campagne::{Campagne, ErreurCampagne, FiltreJournal};
use serde::Deserialize;

use crate::rendu::{self, FiltresAffiches};

#[derive(Clone)]
pub struct Etat {
    base: Arc<PathBuf>,
}

impl Etat {
    pub fn new(chemin: PathBuf) -> Self {
        Self { base: Arc::new(chemin) }
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
