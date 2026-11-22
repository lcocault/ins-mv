//! Routes HTTP : pages du joueur (lecture seule sur la base) et lancer de dé (seule écriture).
//!
//! Le joueur ne voit que son équipe en clair ; les autres personnages, les lieux et le journal
//! ne montrent que ce que le maître de jeu a révélé (`reveler`, entrées de journal `visible`).

use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::Router;
use insmv_campagne::{Campagne, ErreurCampagne, Evenement, FiltreJournal};
use insmv_regles::{DesAleatoires, D666};
use serde::Deserialize;

use crate::rendu::{self, FiltresAffiches};
use crate::vue::Vue;

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
        Campagne::ouvrir_lecture_seule(&self.base).map_err(|e| self.indisponible(e))
    }

    /// Ouverture en écriture, réservée au lancer de dé.
    fn campagne_ecriture(&self) -> Result<Campagne, (StatusCode, Html<String>)> {
        Campagne::ouvrir(&self.base).map_err(|e| self.indisponible(e))
    }

    fn indisponible(&self, e: ErreurCampagne) -> (StatusCode, Html<String>) {
        erreur(
            StatusCode::SERVICE_UNAVAILABLE,
            "Campagne indisponible",
            &format!("Impossible d'ouvrir {} : {e}. La campagne est créée au premier lancement du serveur MCP.", self.base.display()),
        )
    }

    fn vue(&self) -> Result<(Campagne, Vue), (StatusCode, Html<String>)> {
        let campagne = self.campagne()?;
        let personnages = campagne.lister(None).map_err(interne)?;
        let connaissances = campagne.connaissances(None).map_err(interne)?;
        Ok((campagne, Vue::nouvelle(personnages, connaissances)))
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
        .route("/lancer", post(lancer))
        .route("/personnage/{id}", get(personnage))
        .route("/lieux", get(lieux))
        .route("/journal", get(journal))
        .with_state(etat)
}

async fn accueil(State(etat): State<Etat>) -> Reponse {
    let (_, vue) = etat.vue()?;
    let mut rencontres: Vec<rendu::Rencontre> = Vec::new();
    for k in vue.connaissances.iter().filter(|k| k.genre == "personnage") {
        if vue.est_equipe(&k.sujet) {
            continue;
        }
        match rencontres.iter_mut().find(|(id, _, _)| *id == k.sujet) {
            Some((_, _, n)) => *n += 1,
            None => rencontres.push((k.sujet.clone(), vue.nom_affiche(&k.sujet).unwrap_or_default(), 1)),
        }
    }
    Ok(Html(rendu::accueil(&vue.equipe, &rencontres)))
}

async fn personnage(State(etat): State<Etat>, Path(id): Path<String>) -> Reponse {
    let (_, vue) = etat.vue()?;
    let id = insmv_regles::personnage::identifiant(&id);
    if let Some(p) = vue.equipe.iter().find(|p| p.id == id) {
        return Ok(Html(rendu::fiche(p)));
    }
    // Hors équipe, même message qu'un personnage inexistant tant que le joueur ne l'a pas rencontré.
    if vue.est_rencontre(&id) {
        let infos: Vec<_> = vue.connaissances.iter().filter(|k| k.genre == "personnage" && k.sujet == id).cloned().collect();
        return Ok(Html(rendu::fiche_connue(&vue.nom_affiche(&id).unwrap_or_default(), &infos)));
    }
    Err(erreur(StatusCode::NOT_FOUND, "Personnage introuvable", &format!("Aucun personnage « {id} » connu.")))
}

async fn lieux(State(etat): State<Etat>) -> Reponse {
    let (_, vue) = etat.vue()?;
    let mut lieux: Vec<(String, String, Vec<insmv_campagne::Connaissance>)> = Vec::new();
    for k in vue.connaissances.iter().filter(|k| k.genre == "lieu") {
        match lieux.iter_mut().find(|(sujet, _, _)| *sujet == k.sujet) {
            Some((_, nom, infos)) => {
                if !k.nom_connu.is_empty() {
                    *nom = k.nom_connu.clone();
                }
                infos.push(k.clone());
            }
            None => lieux.push((k.sujet.clone(), k.nom_connu.clone(), vec![k.clone()])),
        }
    }
    let lieux: Vec<_> = lieux.into_iter().map(|(sujet, nom, infos)| (if nom.is_empty() { sujet } else { nom }, infos)).collect();
    Ok(Html(rendu::lieux(&lieux)))
}

/// Jets récents listés sous la table UM.
const JETS_RECENTS: usize = 6;

async fn partie(State(etat): State<Etat>) -> Reponse {
    let (campagne, vue) = etat.vue()?;
    let resolution = campagne.derniere_resolution().map_err(interne)?;
    let jets = campagne.derniers_jets(JETS_RECENTS).map_err(interne)?;
    let combat = campagne.combat_en_cours().map_err(interne)?;
    Ok(Html(rendu::partie(resolution.as_ref(), &jets, combat.as_ref(), &vue)))
}

#[derive(Debug, Deserialize)]
struct FormulaireLancer {
    /// Malus volontaire choisi, de 0 à 3 colonnes.
    #[serde(default)]
    malus: u8,
}

/// Le joueur lance le dé de l'action en attente. L'écriture est limitée à cet événement.
async fn lancer(State(etat): State<Etat>, Form(f): Form<FormulaireLancer>) -> Result<Redirect, (StatusCode, Html<String>)> {
    if f.malus > 3 {
        return Err(erreur(StatusCode::BAD_REQUEST, "Malus refusé", "Le malus volontaire va de 0 à 3 colonnes."));
    }
    let campagne = etat.campagne_ecriture()?;
    if campagne.attente_en_cours().map_err(interne)?.is_none() {
        return Err(erreur(StatusCode::CONFLICT, "Rien à lancer", "Aucune action n'attend votre dé : le maître de jeu doit d'abord la proposer."));
    }
    let valeur = D666::tirer(&mut DesAleatoires::default()).valeur();
    campagne.emettre(&Evenement::DeJoueur { valeur, malus_volontaire: f.malus }).map_err(interne)?;
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
    let (campagne, vue) = etat.vue()?;
    let filtre = FiltreJournal {
        seance: non_vide(q.seance),
        categorie: non_vide(q.categorie),
        contient: non_vide(q.contient),
        personnage: non_vide(q.personnage),
        limite: LIMITE_JOURNAL,
        visibles_seulement: true,
    };
    let affiches = FiltresAffiches {
        seance: filtre.seance.clone().unwrap_or_default(),
        categorie: filtre.categorie.clone().unwrap_or_default(),
        contient: filtre.contient.clone().unwrap_or_default(),
        personnage: filtre.personnage.clone().unwrap_or_default(),
    };
    let entrees = campagne.lire_journal(&filtre).map_err(interne)?;
    let mut choix: Vec<(String, String)> = vue.equipe.iter().map(|p| (p.id.clone(), p.nom.clone())).collect();
    for k in vue.connaissances.iter().filter(|k| k.genre == "personnage") {
        if !choix.iter().any(|(id, _)| *id == k.sujet) {
            if let Some(nom) = vue.nom_affiche(&k.sujet) {
                choix.push((k.sujet.clone(), nom));
            }
        }
    }
    Ok(Html(rendu::journal(&entrees, &|id| vue.nom_affiche(id), &choix, &affiches)))
}
