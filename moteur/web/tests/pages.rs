//! Tests des routes : requêtes réelles contre le routeur, sur une base temporaire.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use insmv_campagne::{ActionCombat, Campagne, EntreeJournal, Evenement};
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
        pp_max_scenario: None,
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
    for (texte, seance, visible) in [
        ("Arrivée au centre commercial", "Séance 1", true),
        ("Le concierge <b>louche</b>", "Séance 2", true),
        ("Secret : le concierge est un démon", "Séance 2", false),
    ] {
        campagne
            .noter(EntreeJournal {
                id: 0,
                horodatage: 0,
                seance: seance.into(),
                categorie: "scene".into(),
                texte: texte.into(),
                personnages: vec!["jean-de-lavenue".into(), "concierge-secret".into()],
                visible,
            })
            .unwrap();
    }
    let (statut, html) = get(&base, "/journal").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(html.contains("Arrivée au centre commercial"));
    assert!(html.contains("Le concierge &lt;b&gt;louche&lt;/b&gt;"), "texte échappé");
    assert!(!html.contains("Secret : le concierge"), "une entrée du MJ n'est pas montrée");
    assert!(!html.contains("concierge-secret"), "un personnage non rencontré n'est pas cité");

    let (_, filtre) = get(&base, "/journal?seance=S%C3%A9ance%202&contient=").await;
    assert!(filtre.contains("louche") && !filtre.contains("centre commercial"));
    let (_, aucun) = get(&base, "/journal?personnage=fantome").await;
    assert!(aucun.contains("Aucune entrée"));
}

fn billy() -> Personnage {
    let mut p = jean();
    p.nom = "Billy Diniez".into();
    p.role = Role::Pnj;
    p.espece = Espece::Demon;
    p.patron = Some("Furfur".into());
    p.finaliser();
    p
}

#[tokio::test]
async fn un_pnj_n_existe_pour_le_joueur_qu_une_fois_revele() {
    let (base, campagne) = base_avec_jean("pnj");
    campagne.creer(&billy()).unwrap();
    // Jamais rencontré : ni dans la liste, ni par son identifiant, ni par son nom.
    let (_, accueil) = get(&base, "/").await;
    assert!(!accueil.contains("Billy") && !accueil.contains("Furfur") && !accueil.contains("Démon"));
    let (statut, _) = get(&base, "/personnage/billy-diniez").await;
    assert_eq!(statut, StatusCode::NOT_FOUND);

    campagne.reveler("personnage", "Billy Diniez", "le batteur américain", "Blouson noir, accent fort.").unwrap();
    let (_, accueil) = get(&base, "/").await;
    assert!(accueil.contains("le batteur américain") && !accueil.contains("Billy"));
    let (statut, fiche) = get(&base, "/personnage/billy-diniez").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(fiche.contains("Blouson noir, accent fort."));
    for secret in ["Billy", "Furfur", "Démon", "PP", "Force"] {
        assert!(!fiche.contains(secret), "« {secret} » ne doit pas fuiter : {fiche}");
    }
    // L'équipe garde sa fiche complète.
    let (_, jean) = get(&base, "/personnage/jean-de-lavenue").await;
    assert!(jean.contains("Ange · Joueur"));
}

#[tokio::test]
async fn lieux_montrent_ce_que_le_joueur_sait() {
    let (base, campagne) = base_avec_jean("lieux");
    let (_, vide) = get(&base, "/lieux").await;
    assert!(vide.contains("Aucun lieu connu"));
    campagne.reveler("lieu", "Le Gibus", "Le Gibus", "Club du faubourg du <Temple>.").unwrap();
    campagne.reveler("lieu", "le gibus", "", "Camionnette dans la cour.").unwrap();
    let (statut, html) = get(&base, "/lieux").await;
    assert_eq!(statut, StatusCode::OK);
    assert_eq!(html.matches("<h2>Le Gibus</h2>").count(), 1, "un seul lieu, deux informations");
    assert!(html.contains("Club du faubourg du &lt;Temple&gt;.") && html.contains("Camionnette dans la cour."));
}

#[tokio::test]
async fn la_page_partie_masque_les_adversaires() {
    let (base, campagne) = base_avec_jean("masque");
    campagne.creer(&billy()).unwrap();
    campagne
        .emettre(&Evenement::Action(ActionCombat {
            acteur: "Jean De Lavenue".into(),
            cible: Some("Billy Diniez".into()),
            resume: "Jean De Lavenue attaque Billy Diniez (contact) : 366 → touche\nDommages : 7 (3 unités)\nBilly Diniez : 7 dommage(s) au total, Force 0 (Valide)."
                .into(),
        }))
        .unwrap();
    let (_, html) = get(&base, "/partie").await;
    assert!(html.contains("un inconnu") && html.contains("366 → touche"));
    for secret in ["Billy", "Dommages", "dommage(s) au total", "Force 0"] {
        assert!(!html.contains(secret), "« {secret} » ne doit pas fuiter : {html}");
    }
    campagne.reveler("personnage", "Billy Diniez", "le batteur", "x").unwrap();
    let (_, connu) = get(&base, "/partie").await;
    assert!(connu.contains("attaque le batteur") && !connu.contains("Billy"));
}

async fn poster(base: &BaseTemporaire, uri: &str, corps: &str) -> (StatusCode, Option<String>) {
    let reponse = routeur(Etat::new(base.0.clone()))
        .oneshot(
            Request::post(uri)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(corps.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let lieu = reponse.headers().get("location").and_then(|v| v.to_str().ok()).map(String::from);
    (reponse.status(), lieu)
}

fn attente() -> Evenement {
    let action = insmv_regles::Action::new(
        insmv_regles::Caracteristique::new(5).unwrap(),
        insmv_regles::NiveauTalent::new(2).unwrap(),
    );
    Evenement::Attente { libelle: "Jean De Lavenue : Arme de contact".into(), action }
}

#[tokio::test]
async fn le_joueur_lance_le_de_de_l_action_proposee() {
    let (base, campagne) = base_avec_jean("lancer");
    // Rien à lancer : refusé, aucun dé écrit.
    assert_eq!(poster(&base, "/lancer", "malus=0").await.0, StatusCode::CONFLICT);
    assert!(campagne.de_joueur_disponible().unwrap().is_none());

    campagne.emettre(&attente()).unwrap();
    let (statut, lieu) = poster(&base, "/lancer", "malus=2").await;
    assert_eq!((statut, lieu.as_deref()), (StatusCode::SEE_OTHER, Some("/partie")));
    let de = campagne.de_joueur_disponible().unwrap().expect("dé lancé");
    match de.evenement {
        Evenement::DeJoueur { valeur, malus_volontaire } => {
            assert!(insmv_regles::D666::depuis_valeur(valeur).is_ok(), "trois faces de 1 à 6 : {valeur}");
            assert_eq!(malus_volontaire, 2);
        }
        autre => panic!("événement inattendu : {autre:?}"),
    }
    // Un seul lancer par action proposée.
    assert_eq!(poster(&base, "/lancer", "malus=0").await.0, StatusCode::CONFLICT);
    let (_, partie) = get(&base, "/partie").await;
    assert!(partie.contains("Dé lancé") && partie.contains("malus volontaire de 2"));
}

#[tokio::test]
async fn malus_hors_bornes_et_formulaire_invalide_sont_refuses() {
    let (base, campagne) = base_avec_jean("malus");
    campagne.emettre(&attente()).unwrap();
    assert_eq!(poster(&base, "/lancer", "malus=4").await.0, StatusCode::BAD_REQUEST);
    assert!(poster(&base, "/lancer", "malus=abc").await.0.is_client_error());
    assert!(campagne.de_joueur_disponible().unwrap().is_none(), "rien n'a été lancé");
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

#[tokio::test]
async fn partie_suit_les_evenements_du_serveur_principal() {
    let (base, campagne) = base_avec_jean("partie");
    let (statut, vide) = get(&base, "/partie").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(vide.contains("Aucun jet pour l'instant") && vide.contains("Aucun combat en cours"));

    let action = insmv_regles::Action::new(
        insmv_regles::Caracteristique::new(5).unwrap(),
        insmv_regles::NiveauTalent::new(2).unwrap(),
    );
    campagne.emettre(&Evenement::Attente { libelle: "Jean De Lavenue : Arme de contact".into(), action }).unwrap();
    let (_, attente) = get(&base, "/partie").await;
    assert!(attente.contains("À vous de lancer") && attente.contains("Jean De Lavenue : Arme de contact"));

    campagne
        .emettre(&Evenement::Action(ActionCombat { acteur: "Jean De Lavenue".into(), cible: None, resume: "Jean frappe".into() }))
        .unwrap();
    let (_, combat) = get(&base, "/partie").await;
    assert!(combat.contains("Jean frappe") && combat.contains("Initiative non tirée"));

    campagne.emettre(&Evenement::Fin).unwrap();
    let (_, fini) = get(&base, "/partie").await;
    assert!(fini.contains("Aucun combat en cours"));
}

#[tokio::test]
async fn partie_sur_une_ancienne_base_sans_evenements() {
    let base = BaseTemporaire::nouvelle("ancienne");
    std::fs::create_dir_all(base.0.parent().unwrap()).unwrap();
    // Base créée avant l'ajout de la table d'événements.
    drop(Campagne::ouvrir(&base.0).unwrap());
    let (statut, html) = get(&base, "/partie").await;
    assert_eq!(statut, StatusCode::OK);
    assert!(html.contains("Aucun combat en cours"));
}
