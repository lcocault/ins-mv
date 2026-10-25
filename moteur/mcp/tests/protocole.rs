//! Test de bout en bout : lance le binaire et dialogue en JSON-RPC sur stdio.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{json, Value};

fn echanger(requetes: &[Value], nb_reponses: usize) -> Vec<Value> {
    let mut enfant = Command::new(env!("CARGO_BIN_EXE_insmv-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("lancement du serveur");
    let mut entree = enfant.stdin.take().unwrap();
    let mut sortie = BufReader::new(enfant.stdout.take().unwrap());

    for requete in requetes {
        writeln!(entree, "{requete}").unwrap();
    }
    let mut reponses = Vec::new();
    while reponses.len() < nb_reponses {
        let mut ligne = String::new();
        if sortie.read_line(&mut ligne).unwrap() == 0 {
            break;
        }
        reponses.push(serde_json::from_str(&ligne).expect("réponse JSON"));
    }
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
fn expose_les_trois_outils() {
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
    assert_eq!(noms, ["lancer_des", "resoudre_action", "resoudre_conflit"]);
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
