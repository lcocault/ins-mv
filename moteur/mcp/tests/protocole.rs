//! Test de bout en bout : lance le binaire et dialogue en JSON-RPC sur stdio.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

/// Base de campagne jetable, supprimée à la fin du test.
struct BaseTemporaire(std::path::PathBuf);

impl BaseTemporaire {
    fn nouvelle(nom: &str) -> Self {
        let dossier = std::env::temp_dir().join(format!("insmv-proto-{}-{nom}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dossier);
        Self(dossier.join("jeu.sqlite"))
    }
}

impl Drop for BaseTemporaire {
    fn drop(&mut self) {
        if let Some(dossier) = self.0.parent() {
            let _ = std::fs::remove_dir_all(dossier);
        }
    }
}

fn echanger(requetes: &[Value], nb_reponses: usize) -> Vec<Value> {
    // Un nom unique par appel : les tests s'exécutent en parallèle.
    static COMPTEUR: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = COMPTEUR.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let base = BaseTemporaire::nouvelle(&format!("jetable-{n}"));
    echanger_sur(&base, requetes, nb_reponses)
}

fn echanger_sur(base: &BaseTemporaire, requetes: &[Value], nb_reponses: usize) -> Vec<Value> {
    let mut enfant = Command::new(env!("CARGO_BIN_EXE_insmv-mcp"))
        .env("INSMV_CAMPAGNE", &base.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("lancement du serveur");
    let mut entree = enfant.stdin.take().unwrap();
    let mut sortie = BufReader::new(enfant.stdout.take().unwrap());

    // Le serveur traite les requêtes en parallèle : on attend chaque réponse avant
    // d'envoyer la suivante, pour que les scénarios restent déterministes.
    let mut reponses = Vec::new();
    for requete in requetes {
        writeln!(entree, "{requete}").unwrap();
        if requete.get("id").is_none() {
            continue; // notification : pas de réponse
        }
        let mut ligne = String::new();
        if sortie.read_line(&mut ligne).unwrap() == 0 {
            break;
        }
        reponses.push(serde_json::from_str(&ligne).expect("réponse JSON"));
    }
    assert_eq!(reponses.len(), nb_reponses, "nombre de réponses reçues");
    drop(entree);
    let _ = enfant.kill();
    reponses
}

fn initialisation() -> Vec<Value> {
    vec![
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},
            "clientInfo":{"name":"test","version":"0"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
    ]
}

fn appel(id: u64, outil: &str, arguments: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
           "params":{"name":outil,"arguments":arguments}})
}

fn texte(reponse: &Value) -> Value {
    let brut = reponse["result"]["content"][0]["text"].as_str().expect("contenu texte");
    serde_json::from_str(brut).expect("le contenu est du JSON")
}

#[test]
fn expose_les_outils() {
    let mut requetes = initialisation();
    requetes.push(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let reponses = echanger(&requetes, 2);
    let mut noms: Vec<&str> = reponses[1]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    noms.sort();
    assert_eq!(
        noms,
        [
            "ajuster_dommages",
            "ajuster_pp",
            "annoncer_action",
            "attaquer",
            "chercher_pouvoir",
            "chercher_talent",
            "conflit_personnages",
            "creer_personnage",
            "initiative",
            "lancer_des",
            "lire_journal",
            "lister_personnages",
            "noter_journal",
            "resoudre_action",
            "resoudre_conflit",
            "soigner_semaines",
            "supprimer_personnage",
            "terminer_combat",
            "tester_personnage",
            "tirer_pouvoir",
            "utiliser_pouvoir",
            "voir_personnage",
        ]
    );
}

#[test]
fn resoudre_action_renvoie_un_resultat_coherent() {
    let mut requetes = initialisation();
    requetes.push(appel(2, "resoudre_action", json!({"caracteristique": 6, "niveau": 3})));
    let reponses = echanger(&requetes, 2);
    let sortie = texte(&reponses[1]);
    // Caractéristique 6 niveau 3 : seuil 66, réussite certaine.
    assert_eq!(sortie["reussi"], true);
    assert_eq!(sortie["seuil"]["valeur"], 66);
    assert!(sortie["resume"].as_str().unwrap().contains("RÉUSSI"));
}

#[test]
fn parametres_invalides_renvoient_une_erreur_outil() {
    let mut requetes = initialisation();
    requetes.push(appel(2, "resoudre_action", json!({"caracteristique": 9})));
    requetes.push(appel(3, "lancer_des", json!({"notation": "D8"})));
    let reponses = echanger(&requetes, 3);
    for reponse in &reponses[1..] {
        assert_eq!(reponse["result"]["isError"], true, "{reponse}");
    }
}

#[test]
fn conflit_et_des_libres() {
    let mut requetes = initialisation();
    requetes.push(appel(
        2,
        "resoudre_conflit",
        json!({"attaquant":{"caracteristique":6,"niveau":3},"defenseur":{"caracteristique":6,"niveau":3}}),
    ));
    requetes.push(appel(3, "lancer_des", json!({"notation": "2D6+3"})));
    let reponses = echanger(&requetes, 3);

    let conflit = texte(&reponses[1]);
    assert_eq!(conflit["issue"], "conflit_reel");
    let total = texte(&reponses[2])["total"].as_i64().unwrap();
    assert!((5..=15).contains(&total), "2D6+3 hors bornes : {total}");
}

fn demon(nom: &str) -> Value {
    json!({
        "nom": nom, "espece": "demon", "role": "joueur", "patron": "Belial",
        "caracteristiques": {"fo":3,"pr":3,"vo":4,"pe":3,"ag":3,"ap":2},
        "talents": [{"nom":"Arme de poing","niveau":2},{"nom":"Esquive","niveau":2},{"nom":"Baratin","niveau":1}],
        "pouvoirs": [{"code":321},{"code":112},{"code":211},{"code":311}]
    })
}

#[test]
fn la_campagne_persiste_d_une_session_a_l_autre() {
    let base = BaseTemporaire::nouvelle("persistance");

    // Session 1 : création, dépense de PP, note au journal.
    let mut r = initialisation();
    r.push(appel(2, "creer_personnage", demon("Mnofriytkgh")));
    r.push(appel(3, "ajuster_pp", json!({"nom":"Mnofriytkgh","delta":-3})));
    r.push(appel(4, "noter_journal", json!({"texte":"Rencontre du concierge","seance":"Séance 1","personnages":["Mnofriytkgh"]})));
    let reponses = echanger_sur(&base, &r, 4);
    assert_eq!(texte(&reponses[1])["pp_max"], 8);
    assert_eq!(texte(&reponses[2])["pp"], 5);

    // Session 2 : tout est retrouvé.
    let mut r = initialisation();
    r.push(appel(2, "voir_personnage", json!({"nom":"mnofriytkgh"})));
    r.push(appel(3, "lire_journal", json!({"personnage":"Mnofriytkgh"})));
    r.push(appel(4, "lister_personnages", json!({"role":"joueur"})));
    let reponses = echanger_sur(&base, &r, 4);
    let fiche = texte(&reponses[1]);
    assert_eq!((fiche["pp"].as_u64(), fiche["pp_max"].as_u64()), (Some(5), Some(8)));
    assert_eq!(fiche["fiche"]["pouvoirs"][0]["nom"], "Télépathie");
    assert_eq!(texte(&reponses[2])[0]["texte"], "Rencontre du concierge");
    assert_eq!(texte(&reponses[3])[0]["nom"], "Mnofriytkgh");
}

#[test]
fn test_sur_fiche_et_refus_de_budget_via_le_protocole() {
    let mut r = initialisation();
    r.push(appel(2, "creer_personnage", demon("Paulo")));
    r.push(appel(3, "tester_personnage", json!({
        "nom":"Paulo","caracteristique":"PE","talent":"Arme de poing","cout_pp":1})));
    let mut trop = demon("Gros bill");
    trop["caracteristiques"] = json!({"fo":5,"pr":5,"vo":5,"pe":5,"ag":5,"ap":5});
    r.push(appel(4, "creer_personnage", trop));
    let reponses = echanger(&r, 4);

    let test = texte(&reponses[2]);
    assert_eq!(test["pp_depenses"], 1);
    assert_eq!(test["pp_restants"], 7);
    assert!(test["description"].as_str().unwrap().contains("Arme de poing +2"));
    assert!(matches!(test["seuil"]["valeur"].as_u64(), Some(62)), "{test}");

    assert_eq!(reponses[3]["result"]["isError"], true);
    let message = reponses[3]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(message.contains("exception"), "{message}");
}

fn ange_jean() -> Value {
    json!({
        "nom": "Jean", "espece": "ange", "role": "joueur", "patron": "Laurent",
        "caracteristiques": {"fo":4,"pr":2,"vo":4,"pe":3,"ag":5,"ap":2},
        "talents": [{"nom":"Arme de contact","niveau":2},{"nom":"Esquive","niveau":1},{"nom":"Corps à corps","niveau":0},{"nom":"Discrétion","niveau":0}],
        "pouvoirs": [{"code":413},{"code":166},{"code":243},{"code":321},{"code":263},{"code":622}]
    })
}

#[test]
fn combat_complet_via_le_protocole() {
    let mut r = initialisation();
    r.push(appel(2, "creer_personnage", ange_jean()));
    r.push(appel(3, "creer_personnage", json!({
        "nom": "Zombi", "espece": "humain", "role": "pnj",
        "caracteristiques": {"fo":2,"pr":1,"vo":2,"pe":2,"ag":2,"ap":1},
        "talents": [{"nom":"Esquive","niveau":0}]
    })));
    r.push(appel(4, "initiative", json!({"personnages": ["Jean", "Zombi"]})));
    r.push(appel(5, "utiliser_pouvoir", json!({"nom":"Jean","pouvoir":"243"})));
    r.push(appel(6, "attaquer", json!({
        "attaquant":"Jean","defenseur":"Zombi","talent":"Arme de contact",
        "arme":"Epée longue","bonus_puissance":1})));
    r.push(appel(7, "voir_personnage", json!({"nom":"Zombi"})));
    r.push(appel(8, "voir_personnage", json!({"nom":"Jean"})));
    let reponses = echanger(&r, 8);
    for (i, reponse) in reponses.iter().enumerate().skip(1) {
        assert_eq!(reponse["result"]["isError"], false, "réponse {i} : {reponse}");
    }

    // Initiative : Jean (AG 5) agit avant Zombi (AG 2) ; seul Jean a des pouvoirs.
    let ordre = texte(&reponses[3]);
    assert_eq!(ordre["physique"][0]["personnages"][0], "Jean");
    assert_eq!(ordre["psychique"][0]["personnages"][0], "Jean");

    // Pouvoir : 1 PP de coût, texte du livre transmis.
    let pouvoir = texte(&reponses[4]);
    assert_eq!((pouvoir["cout_paye"].as_u64(), pouvoir["pp_restants"].as_u64()), (Some(1), Some(9)));
    assert!(pouvoir["regle"].as_str().unwrap().contains("caractéristique"));

    // Attaque : les dommages annoncés sont ceux qui sont persistés sur la fiche du défenseur.
    let attaque = texte(&reponses[5]);
    assert!(attaque["resume"].as_str().unwrap().contains("Jean attaque Zombi"));
    let annonces = match attaque["dommages"]["total"].as_u64() {
        Some(total) if attaque["arbitrage_mj"] == false => total,
        _ => 0,
    };
    assert_eq!(texte(&reponses[6])["fiche"]["dommages"].as_u64().unwrap(), annonces);
    // Jean, lui, n'a pas été blessé et a gardé ses PP.
    let jean = texte(&reponses[7]);
    assert_eq!((jean["pp"].as_u64(), jean["fiche"]["dommages"].as_u64()), (Some(9), Some(0)));
}
