//! Tests des routes : requêtes réelles contre le routeur, sur une base temporaire.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use insmv_campagne::{Campagne, EntreeJournal};
use insmv_regles::personnage::{Caracteristiques, Espece, Personnage, PouvoirPossede, Role, TalentPossede};
use insmv_web::{routeur, Etat};
use tower::ServiceExt;

struct BaseTemporaire(std::path::PathBuf);

impl BaseTemporaire {
    fn nouvelle(nom: &str) -> Self {
        let dossier = std::env::temp_dir().join(format!("insmv-web-{}-{nom}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        Self(dossier.join("jeu.sqlite"))
    }
}

impl Drop for BaseTemporaire {
    fn drop(&mut self) {
        if let Some(d) = self.0.parent() {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}

fn jean() -> Personnage {
    let mut p = Personnage {
        id: String::new(),
        nom: "Jean De Lavenue".into(),
        espece: Espece::Ange,
        role: Role::Joueur,
        caracteristiques: Caracteristiques { fo: 4, pr: 2, vo: 4, pe: 3, ag: 5, ap: 2 },
        talents: vec![TalentPossede { nom: "Arme de contact".into(), niveau: 2 }],
        pouvoirs: vec![PouvoirPossede { code: Some(166), nom: String::new(), niveau: 0, limitation: false }],
        patron: Some("Laurent".into()),
        pp: None,
        dommages: 0,
        equipement: vec!["Erendil".into()],
        notes: String::new(),
    };
    p.finaliser();
    p
}

async fn get(base: &BaseTemporaire, uri: &str) -> (StatusCode, String) {
    let reponse = routeur(Etat::new(base.0.clone()))
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let statut = reponse.status();
    let corps = reponse.into_body().collect().await.unwrap().to_bytes();
    (statut, String::from_utf8(corps.to_vec()).unwrap())
}

fn base_avec_jean(nom: &str) -> (BaseTemporaire, Campagne) {
    let base = BaseTemporaire::nouvelle(nom);
    let campagne = Campagne::ouvrir(&base.0).unwrap();
    campagne.creer(&jean()).unwrap();
    (base, campagne)
}

#[tokio::test]
async fn accueil_liste_les_personnages() {
    let (base, _c) = base_avec_jean("accueil");
    let (statut, html) = get(&base, "/").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(html.contains("Jean De Lavenue") && html.contains("/personnage/jean-de-lavenue"));
}

#[tokio::test]
async fn fiche_par_identifiant_et_par_nom_avec_chances_de_jet() {
    let (base, _c) = base_avec_jean("fiche");
    let (statut, html) = get(&base, "/personnage/jean-de-lavenue").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(html.contains("Ange · Joueur · au service de Laurent"));
    assert!(html.contains("(100 %)"), "arme de contact +2 avec AG 5");
    // Le nom lui-même (encodé) retrouve aussi la fiche.
    let (statut, _) = get(&base, "/personnage/Jean%20De%20Lavenue").await;
    assert_eq!(statut, StatusCode::OK);
}

#[tokio::test]
async fn personnage_inconnu_donne_404() {
    let (base, _c) = base_avec_jean("inconnu");
    let (statut, html) = get(&base, "/personnage/fantome").await;
    assert_eq!(statut, StatusCode::NOT_FOUND);
    assert!(html.contains("Personnage introuvable"));
}

#[tokio::test]
async fn la_vue_suit_les_ecritures_du_serveur_principal() {
    let (base, campagne) = base_avec_jean("live");
    let (_, avant) = get(&base, "/personnage/jean-de-lavenue").await;
    assert!(avant.contains("<b>0</b>"), "aucun dommage au départ");

    campagne
        .modifier("Jean De Lavenue", |p| {
            p.ajuster_dommages(4);
            p.depenser_pp(2).map(|_| ())
        })
        .unwrap()
        .unwrap();
    let (_, apres) = get(&base, "/personnage/jean-de-lavenue").await;
    assert!(apres.contains("<b>4</b>"), "dommages à jour");
    assert!(apres.contains("PP 3/5") || apres.contains("<b>3</b> / 5"), "PP à jour : {apres}");
}

#[tokio::test]
async fn journal_et_filtres() {
    let (base, campagne) = base_avec_jean("journal");
    for (texte, seance) in [("Arrivée au centre commercial", "Séance 1"), ("Le concierge <b>louche</b>", "Séance 2")] {
        campagne
            .noter(EntreeJournal {
                id: 0,
                horodatage: 0,
                seance: seance.into(),
                categorie: "scene".into(),
                texte: texte.into(),
                personnages: vec!["jean-de-lavenue".into()],
            })
            .unwrap();
    }
    let (statut, html) = get(&base, "/journal").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(html.contains("Arrivée au centre commercial"));
    assert!(html.contains("Le concierge &lt;b&gt;louche&lt;/b&gt;"), "texte échappé");

    let (_, filtre) = get(&base, "/journal?seance=S%C3%A9ance%202&contient=").await;
    assert!(filtre.contains("concierge") && !filtre.contains("centre commercial"));
    let (_, aucun) = get(&base, "/journal?personnage=fantome").await;
    assert!(aucun.contains("Aucune entrée"));
}

#[tokio::test]
async fn base_absente_donne_un_message_clair() {
    let base = BaseTemporaire::nouvelle("absente");
    let (statut, html) = get(&base, "/").await;
    assert_eq!(statut, StatusCode::SERVICE_UNAVAILABLE);
    assert!(html.contains("Campagne indisponible"));
}

#[tokio::test]
async fn routes_inconnues_et_ecritures_refusees() {
    let (base, _c) = base_avec_jean("routes");
    assert_eq!(get(&base, "/nimporte/quoi").await.0, StatusCode::NOT_FOUND);
    let reponse = routeur(Etat::new(base.0.clone()))
        .oneshot(Request::post("/personnage/jean-de-lavenue").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(reponse.status(), StatusCode::METHOD_NOT_ALLOWED);
}
