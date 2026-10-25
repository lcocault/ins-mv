//! Outils de campagne : fiches, état des personnages, tests sur fiche, catalogues, journal.
//! Comme [`crate::outils`], cette couche est indépendante du protocole MCP.

use insmv_campagne::{Campagne, EntreeJournal, ErreurCampagne, FiltreJournal};
use insmv_regles::catalogue::{self, Livre, Pouvoir, Talent};
use insmv_regles::personnage::{
    controler, Budget, Carac, Caracteristiques, DemandeTest, Espece, Gravite, Personnage,
    PouvoirPossede, Probleme, Role, TalentPossede,
};
use insmv_regles::{evaluer, resoudre_conflit as arbitrer, Issue, SourceDes, D666};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::outils::{decrire_action, decrire_issue, SortieAction};

fn erreur(e: ErreurCampagne) -> String {
    e.to_string()
}

// ---- Création -----------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreationParams {
    pub nom: String,
    pub espece: Espece,
    /// joueur, partenaire (PNJ qui accompagne le joueur) ou pnj. Défaut : pnj.
    #[serde(default)]
    pub role: Role,
    pub caracteristiques: Caracteristiques,
    #[serde(default)]
    pub talents: Vec<TalentPossede>,
    /// Pouvoirs (et limitations 6xx) : un code à trois chiffres de la table de
    /// l'espèce, ou à défaut un nom non ambigu.
    #[serde(default)]
    pub pouvoirs: Vec<PouvoirPossede>,
    /// Archange ou Prince-Démon dont dépend le personnage.
    #[serde(default)]
    pub patron: Option<String>,
    #[serde(default)]
    pub equipement: Vec<String>,
    #[serde(default)]
    pub notes: String,
    /// Justification d'une exception aux budgets de création (le livre admet
    /// des exceptions). Sans elle, tout dépassement de budget est refusé.
    #[serde(default)]
    pub exception: Option<String>,
    /// Remplace une fiche existante du même nom au lieu d'échouer.
    #[serde(default)]
    pub remplacer: bool,
}

#[derive(Debug, Serialize)]
pub struct SortieCreation {
    pub personnage: Personnage,
    pub pp_max: u32,
    pub avertissements: Vec<String>,
    pub exceptions_acceptees: Vec<String>,
}

pub fn creer_personnage(c: &Campagne, p: CreationParams) -> Result<SortieCreation, String> {
    let mut perso = Personnage {
        id: String::new(),
        nom: p.nom.trim().to_string(),
        espece: p.espece,
        role: p.role,
        caracteristiques: p.caracteristiques,
        talents: p.talents,
        pouvoirs: p.pouvoirs,
        patron: p.patron,
        pp: None,
        dommages: 0,
        equipement: p.equipement,
        notes: p.notes,
    };
    perso.finaliser();
    let problemes = controler(&perso);
    let par = |g: Gravite| -> Vec<String> {
        problemes.iter().filter(|x: &&Probleme| x.gravite == g).map(|x| x.message.clone()).collect()
    };

    let structure = par(Gravite::Structure);
    if !structure.is_empty() {
        return Err(format!("fiche incohérente : {}", structure.join(" ; ")));
    }
    let budget = par(Gravite::Budget);
    if !budget.is_empty() {
        match p.exception.as_deref().map(str::trim).filter(|e| !e.is_empty()) {
            None => {
                return Err(format!(
                    "dépassement des budgets de création : {} — corriger la fiche, ou fournir \
                     `exception` (justification) pour une exception assumée du MJ",
                    budget.join(" ; ")
                ))
            }
            Some(motif) => {
                if !perso.notes.is_empty() {
                    perso.notes.push('\n');
                }
                perso.notes.push_str(&format!(
                    "Exception de création : {motif} (dépassements : {})",
                    budget.join(" ; ")
                ));
            }
        }
    }

    if p.remplacer {
        c.enregistrer(&perso).map_err(erreur)?;
    } else {
        c.creer(&perso).map_err(erreur)?;
    }
    Ok(SortieCreation {
        pp_max: perso.pp_max(),
        avertissements: par(Gravite::Avertissement),
        exceptions_acceptees: budget,
        personnage: perso,
    })
}

// ---- Consultation ---------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct DetailPouvoir {
    pub code: Option<u16>,
    pub nom: String,
    pub niveau: u8,
    pub limitation: bool,
    pub famille: String,
    pub categorie: String,
    pub attaque: Option<String>,
    pub defense: Option<String>,
    pub cout: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VueFiche {
    pub fiche: Personnage,
    pub pp: u32,
    pub pp_max: u32,
    pub budget_espece: Budget,
    pub pouvoirs_detail: Vec<DetailPouvoir>,
}

pub fn voir_personnage(c: &Campagne, nom: &str) -> Result<VueFiche, String> {
    let fiche = c.charger(nom).map_err(erreur)?;
    let livre = fiche.espece.livre_pouvoirs();
    let pouvoirs_detail = fiche
        .pouvoirs
        .iter()
        .map(|p| {
            let cat: Option<&Pouvoir> =
                livre.zip(p.code).and_then(|(l, code)| catalogue::pouvoir_par_code(l, code));
            DetailPouvoir {
                code: p.code,
                nom: p.nom.clone(),
                niveau: p.niveau,
                limitation: p.limitation,
                famille: cat.map(|x| x.famille.clone()).unwrap_or_default(),
                categorie: cat.map(|x| x.categorie.clone()).unwrap_or_default(),
                attaque: cat.and_then(|x| x.attaque.clone()),
                defense: cat.and_then(|x| x.defense.clone()),
                cout: cat.and_then(|x| x.cout.clone()),
            }
        })
        .collect();
    Ok(VueFiche {
        pp: fiche.pp_courants(),
        pp_max: fiche.pp_max(),
        budget_espece: fiche.espece.budget(),
        pouvoirs_detail,
        fiche,
    })
}

#[derive(Debug, Serialize)]
pub struct Resume {
    pub nom: String,
    pub espece: Espece,
    pub role: Role,
    pub pp: String,
    pub dommages: u32,
}

pub fn lister_personnages(c: &Campagne, role: Option<Role>) -> Result<Vec<Resume>, String> {
    Ok(c.lister(role)
        .map_err(erreur)?
        .into_iter()
        .map(|p| Resume {
            pp: format!("{}/{}", p.pp_courants(), p.pp_max()),
            nom: p.nom,
            espece: p.espece,
            role: p.role,
            dommages: p.dommages,
        })
        .collect())
}

pub fn supprimer_personnage(c: &Campagne, nom: &str) -> Result<String, String> {
    c.supprimer(nom).map_err(erreur)?;
    Ok(format!("« {nom} » supprimé"))
}

// ---- État ------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct EtatPersonnage {
    pub nom: String,
    pub pp: u32,
    pub pp_max: u32,
    pub dommages: u32,
}

impl From<&Personnage> for EtatPersonnage {
    fn from(p: &Personnage) -> Self {
        Self { nom: p.nom.clone(), pp: p.pp_courants(), pp_max: p.pp_max(), dommages: p.dommages }
    }
}

/// `delta` > 0 récupère des PP (plafonnés au maximum), < 0 en dépense.
pub fn ajuster_pp(c: &Campagne, nom: &str, delta: i32) -> Result<EtatPersonnage, String> {
    c.modifier(nom, |p| {
        if delta < 0 {
            p.depenser_pp(delta.unsigned_abs())?;
        } else {
            p.recuperer_pp(delta.unsigned_abs());
        }
        Ok(())
    })
    .map_err(erreur)?
    .map(|(p, ())| EtatPersonnage::from(&p))
}

/// `delta` > 0 inflige des dommages, < 0 en soigne (jamais en dessous de 0).
pub fn ajuster_dommages(c: &Campagne, nom: &str, delta: i32) -> Result<EtatPersonnage, String> {
    c.modifier(nom, |p| {
        p.ajuster_dommages(delta);
        Ok(())
    })
    .map_err(erreur)?
    .map(|(p, ())| EtatPersonnage::from(&p))
}

// ---- Tests sur fiche ------------------------------------------------------------------

fn un() -> u8 {
    1
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TestParams {
    /// Nom du personnage.
    pub nom: String,
    /// Caractéristique utilisée : FO, PR, VO, PE, AG, AP (ou le nom complet).
    pub caracteristique: String,
    /// Talent utilisé. S'il n'est pas sur la fiche : ligne 0 avec le malus du
    /// catalogue des talents (ou `malus_sans_talent`).
    #[serde(default)]
    pub talent: Option<String>,
    /// Pouvoir utilisé (doit être sur la fiche). Exclusif avec `talent`.
    #[serde(default)]
    pub pouvoir: Option<String>,
    /// Sans talent ni pouvoir : difficulté 0 (difficile), 1 (moyen), 2 (facile). Défaut : 1.
    #[serde(default = "un")]
    pub difficulte: u8,
    /// Décalage de colonnes du MJ : positif = bonus, négatif = malus.
    #[serde(default)]
    pub modificateur: i32,
    /// Malus volontaire du joueur, en colonnes (ajouté aux unités).
    #[serde(default)]
    pub malus_volontaire: u8,
    /// Malus en colonnes si le talent est absent de la fiche et du catalogue,
    /// ou pour remplacer celui du catalogue.
    #[serde(default)]
    pub malus_sans_talent: Option<u8>,
    /// PP dépensés pour cette action (ex. coût d'un pouvoir), prélevés avant le jet.
    #[serde(default)]
    pub cout_pp: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SortieTest {
    pub description: String,
    pub pp_depenses: u32,
    pub pp_restants: u32,
    #[serde(flatten)]
    pub action: SortieAction,
}

/// Prépare un test (sans effet de bord) : l'action de table UM et sa description.
fn preparer(c: &Campagne, t: &TestParams) -> Result<(insmv_regles::Action, String), String> {
    let perso = c.charger(&t.nom).map_err(erreur)?;
    let carac = Carac::analyser(&t.caracteristique)
        .ok_or_else(|| format!("caractéristique inconnue : « {} » (FO, PR, VO, PE, AG, AP)", t.caracteristique))?;
    let prepare = perso.preparer_test(&DemandeTest {
        carac,
        talent: t.talent.clone(),
        pouvoir: t.pouvoir.clone(),
        difficulte: t.difficulte,
        modificateur: t.modificateur,
        malus_volontaire: t.malus_volontaire,
        malus_sans_talent: t.malus_sans_talent,
    })?;
    if let Some(cout) = t.cout_pp {
        if cout > perso.pp_courants() {
            return Err(format!("{} n'a que {} PP, {cout} demandés", perso.nom, perso.pp_courants()));
        }
    }
    Ok((prepare.action, prepare.description))
}

fn prelever_pp(c: &Campagne, t: &TestParams) -> Result<(u32, u32), String> {
    let cout = t.cout_pp.unwrap_or(0);
    let (p, ()) = c
        .modifier(&t.nom, |p| p.depenser_pp(cout).map(|_| ()))
        .map_err(erreur)??;
    Ok((cout, p.pp_courants()))
}

pub fn tester_personnage(
    c: &Campagne,
    t: &TestParams,
    source: &mut impl SourceDes,
) -> Result<SortieTest, String> {
    let (action, description) = preparer(c, t)?;
    let (pp_depenses, pp_restants) = prelever_pp(c, t)?;
    let action = decrire_action(evaluer(&action, D666::tirer(source)));
    Ok(SortieTest { description, pp_depenses, pp_restants, action })
}

#[derive(Debug, Serialize)]
pub struct SortieConflitPersonnages {
    pub attaquant: SortieTest,
    pub defenseur: SortieTest,
    #[serde(flatten)]
    pub issue: Issue,
    pub resume: String,
}

pub fn conflit_personnages(
    c: &Campagne,
    attaquant: &TestParams,
    defenseur: &TestParams,
    source: &mut impl SourceDes,
) -> Result<SortieConflitPersonnages, String> {
    // Tout est validé avant de toucher aux PP : un conflit invalide ne consomme rien.
    let (action_a, desc_a) = preparer(c, attaquant)?;
    let (action_d, desc_d) = preparer(c, defenseur)?;
    let mut sorties = Vec::new();
    for (t, action, description) in [(attaquant, action_a, desc_a), (defenseur, action_d, desc_d)] {
        let (pp_depenses, pp_restants) = prelever_pp(c, t)?;
        let jet = decrire_action(evaluer(&action, D666::tirer(source)));
        sorties.push(SortieTest { description, pp_depenses, pp_restants, action: jet });
    }
    let defenseur = sorties.pop().expect("deux sorties");
    let attaquant = sorties.pop().expect("deux sorties");
    let issue = arbitrer(&attaquant.action.jet, &defenseur.action.jet);
    Ok(SortieConflitPersonnages { resume: decrire_issue(&issue), attaquant, defenseur, issue })
}

// ---- Catalogues ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LivrePouvoirs {
    /// Table de Magna Veritas (Anges, Serviteurs, Soldats).
    Ange,
    /// Table d'In Nomine Satanis (Démons, Familiers, Morts-vivants).
    Demon,
}

impl From<LivrePouvoirs> for Livre {
    fn from(l: LivrePouvoirs) -> Self {
        match l {
            LivrePouvoirs::Ange => Livre::Ange,
            LivrePouvoirs::Demon => Livre::Demon,
        }
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChercherPouvoirParams {
    pub livre: LivrePouvoirs,
    /// Code à trois chiffres (ex. 321).
    #[serde(default)]
    pub code: Option<u16>,
    /// Texte contenu dans le nom, la catégorie ou la famille (ex. « télépathie », « attaque à distance »).
    #[serde(default)]
    pub requete: Option<String>,
}

/// Plafond de résultats renvoyés par une recherche libre.
const MAX_RESULTATS: usize = 40;

#[derive(Debug, Serialize)]
pub struct SortieRecherche<T> {
    pub total: usize,
    pub resultats: Vec<T>,
}

fn borner<T: Clone>(v: Vec<&T>) -> SortieRecherche<T> {
    SortieRecherche { total: v.len(), resultats: v.into_iter().take(MAX_RESULTATS).cloned().collect() }
}

pub fn chercher_pouvoir(p: &ChercherPouvoirParams) -> Result<SortieRecherche<Pouvoir>, String> {
    let livre = Livre::from(p.livre);
    match (p.code, &p.requete) {
        (Some(code), _) => Ok(borner(catalogue::pouvoir_par_code(livre, code).into_iter().collect())),
        (None, Some(q)) => Ok(borner(catalogue::chercher_pouvoirs(livre, q))),
        (None, None) => Err("indiquer `code` ou `requete`".into()),
    }
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TirerPouvoirParams {
    pub livre: LivrePouvoirs,
    /// Méthode semi-aléatoire : le joueur choisit le chiffre des centaines (1 à 6),
    /// un D66 donne le reste. Sans cela, méthode aléatoire : un D666 complet.
    #[serde(default)]
    pub centaines: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct SortieTirage {
    pub methode: &'static str,
    pub code: u16,
    pub pouvoir: Option<Pouvoir>,
}

pub fn tirer_pouvoir(p: &TirerPouvoirParams, source: &mut impl SourceDes) -> Result<SortieTirage, String> {
    let (methode, code) = match p.centaines {
        Some(x) if (1..=6).contains(&x) => {
            ("semi_aleatoire", u16::from(x) * 100 + u16::from(insmv_regles::des::d66(source)))
        }
        Some(x) => return Err(format!("centaines hors bornes (1 à 6) : {x}")),
        None => ("aleatoire", D666::tirer(source).valeur()),
    };
    Ok(SortieTirage {
        methode,
        code,
        pouvoir: catalogue::pouvoir_par_code(p.livre.into(), code).cloned(),
    })
}

pub fn chercher_talent(requete: &str) -> SortieRecherche<Talent> {
    borner(catalogue::chercher_talents(requete))
}

// ---- Journal ------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NoterParams {
    pub texte: String,
    /// Séance ou chapitre, ex. « Séance 3 ».
    #[serde(default)]
    pub seance: String,
    /// scene, pnj, lieu, intrigue, decision, regle… (libre).
    #[serde(default)]
    pub categorie: String,
    /// Noms des personnages concernés.
    #[serde(default)]
    pub personnages: Vec<String>,
}

pub fn noter_journal(c: &Campagne, p: NoterParams) -> Result<EntreeJournal, String> {
    if p.texte.trim().is_empty() {
        return Err("le texte de l'entrée est vide".into());
    }
    c.noter(EntreeJournal {
        id: 0,
        horodatage: 0,
        seance: p.seance,
        categorie: p.categorie,
        texte: p.texte,
        personnages: p.personnages.iter().map(|n| insmv_regles::personnage::identifiant(n)).collect(),
    })
    .map_err(erreur)
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct LireJournalParams {
    #[serde(default)]
    pub seance: Option<String>,
    #[serde(default)]
    pub categorie: Option<String>,
    /// Texte à rechercher dans les entrées (insensible à la casse).
    #[serde(default)]
    pub contient: Option<String>,
    /// Ne garder que les entrées concernant ce personnage.
    #[serde(default)]
    pub personnage: Option<String>,
    /// Nombre maximal d'entrées (les plus récentes). Défaut : 20.
    #[serde(default)]
    pub limite: Option<usize>,
}

pub fn lire_journal(c: &Campagne, p: LireJournalParams) -> Result<Vec<EntreeJournal>, String> {
    c.lire_journal(&FiltreJournal {
        seance: p.seance,
        categorie: p.categorie,
        contient: p.contient,
        personnage: p.personnage,
        limite: p.limite.unwrap_or(20).clamp(1, 200),
    })
    .map_err(erreur)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insmv_regles::SequenceDes;

    fn carac(fo: u8, vo: u8, ag: u8, ap: u8, pr: u8, pe: u8) -> Caracteristiques {
        Caracteristiques { fo, pr, vo, pe, ag, ap }
    }

    fn demon(nom: &str) -> CreationParams {
        CreationParams {
            nom: nom.into(),
            espece: Espece::Demon,
            role: Role::Joueur,
            // 18 points, VO 4.
            caracteristiques: carac(3, 4, 3, 2, 3, 3),
            talents: vec![
                TalentPossede { nom: "Arme de poing".into(), niveau: 2 },
                TalentPossede { nom: "Esquive".into(), niveau: 2 },
                TalentPossede { nom: "Baratin".into(), niveau: 1 },
            ],
            pouvoirs: [321, 112, 211, 311]
                .map(|code| PouvoirPossede { code: Some(code), nom: String::new(), niveau: 0, limitation: false })
                .into(),
            patron: Some("Belial".into()),
            equipement: vec![],
            notes: String::new(),
            exception: None,
            remplacer: false,
        }
    }

    fn campagne_avec(noms: &[&str]) -> Campagne {
        let c = Campagne::en_memoire().unwrap();
        for n in noms {
            creer_personnage(&c, demon(n)).unwrap();
        }
        c
    }

    fn test(nom: &str, carac: &str) -> TestParams {
        TestParams {
            nom: nom.into(),
            caracteristique: carac.into(),
            talent: None,
            pouvoir: None,
            difficulte: 1,
            modificateur: 0,
            malus_volontaire: 0,
            malus_sans_talent: None,
            cout_pp: None,
        }
    }

    // ---- création ----

    #[test]
    fn creation_valide_renvoie_pp_et_persiste() {
        let c = Campagne::en_memoire().unwrap();
        let s = creer_personnage(&c, demon("Mnofriytkgh")).unwrap();
        assert_eq!(s.pp_max, 8);
        assert!(s.avertissements.is_empty() && s.exceptions_acceptees.is_empty());
        assert_eq!(s.personnage.pouvoirs[0].nom, "Télépathie");
        assert_eq!(c.charger("mnofriytkgh").unwrap().pp_courants(), 8);
    }

    #[test]
    fn creation_refusee_en_cas_de_depassement_sans_exception() {
        let c = Campagne::en_memoire().unwrap();
        let mut p = demon("Gros bill");
        p.caracteristiques = carac(5, 5, 5, 5, 5, 5);
        let err = creer_personnage(&c, p).unwrap_err();
        assert!(err.contains("30 points") && err.contains("exception"), "{err}");
        assert!(c.lister(None).unwrap().is_empty(), "rien n'est enregistré");
    }

    #[test]
    fn creation_acceptee_avec_exception_motivee_et_tracee() {
        let c = Campagne::en_memoire().unwrap();
        let mut p = demon("Prince mineur");
        p.caracteristiques = carac(5, 5, 5, 5, 5, 5);
        p.exception = Some("Boss de fin de scénario".into());
        let s = creer_personnage(&c, p).unwrap();
        assert!(!s.exceptions_acceptees.is_empty());
        assert!(s.personnage.notes.contains("Boss de fin de scénario"));
        assert!(c.charger("Prince mineur").unwrap().notes.contains("Exception de création"));
    }

    #[test]
    fn exception_vide_ne_vaut_pas_justification() {
        let c = Campagne::en_memoire().unwrap();
        let mut p = demon("Triche");
        p.caracteristiques = carac(5, 5, 5, 5, 5, 5);
        p.exception = Some("   ".into());
        assert!(creer_personnage(&c, p).is_err());
    }

    #[test]
    fn une_fiche_incoherente_n_est_jamais_acceptee_meme_avec_exception() {
        let c = Campagne::en_memoire().unwrap();
        let mut p = demon("Bug");
        p.pouvoirs[0].code = Some(999);
        p.exception = Some("peu importe".into());
        let err = creer_personnage(&c, p).unwrap_err();
        assert!(err.contains("incohérente") && err.contains("999"), "{err}");
    }

    #[test]
    fn doublon_refuse_sauf_remplacement() {
        let c = campagne_avec(&["Paulo"]);
        assert!(creer_personnage(&c, demon("Paulo")).unwrap_err().contains("existe déjà"));
        let mut p = demon("Paulo");
        p.remplacer = true;
        p.patron = Some("Baal".into());
        creer_personnage(&c, p).unwrap();
        assert_eq!(c.charger("Paulo").unwrap().patron.as_deref(), Some("Baal"));
    }

    // ---- consultation et état ----

    #[test]
    fn voir_detaille_les_pouvoirs_avec_le_catalogue() {
        let c = campagne_avec(&["Paulo"]);
        let v = voir_personnage(&c, "paulo").unwrap();
        assert_eq!((v.pp, v.pp_max), (8, 8));
        let tele = v.pouvoirs_detail.iter().find(|d| d.code == Some(321)).unwrap();
        assert_eq!(tele.nom, "Télépathie");
        assert!(!tele.categorie.is_empty());
        assert_eq!(v.budget_espece.points_caracteristiques, 18);
        assert!(voir_personnage(&c, "inconnu").unwrap_err().contains("introuvable"));
    }

    #[test]
    fn lister_resume_pp_et_dommages() {
        let c = campagne_avec(&["Paulo", "Albert"]);
        ajuster_dommages(&c, "Albert", 2).unwrap();
        let l = lister_personnages(&c, None).unwrap();
        assert_eq!(l[0].nom, "Albert");
        assert_eq!((l[0].pp.as_str(), l[0].dommages), ("8/8", 2));
        assert!(lister_personnages(&c, Some(Role::Pnj)).unwrap().is_empty());
    }

    #[test]
    fn ajuster_pp_depense_recupere_et_refuse_l_impossible() {
        let c = campagne_avec(&["Paulo"]);
        assert_eq!(ajuster_pp(&c, "Paulo", -3).unwrap().pp, 5);
        assert_eq!(ajuster_pp(&c, "Paulo", 10).unwrap().pp, 8, "plafonné");
        assert!(ajuster_pp(&c, "Paulo", -9).unwrap_err().contains("n'a que 8 PP"));
        assert_eq!(c.charger("Paulo").unwrap().pp_courants(), 8);
    }

    #[test]
    fn ajuster_dommages_infliger_et_soigner() {
        let c = campagne_avec(&["Paulo"]);
        assert_eq!(ajuster_dommages(&c, "Paulo", 5).unwrap().dommages, 5);
        assert_eq!(ajuster_dommages(&c, "Paulo", -9).unwrap().dommages, 0);
    }

    #[test]
    fn supprimer() {
        let c = campagne_avec(&["Paulo"]);
        supprimer_personnage(&c, "Paulo").unwrap();
        assert!(supprimer_personnage(&c, "Paulo").is_err());
    }

    // ---- tests sur fiche ----

    #[test]
    fn test_avec_talent_de_la_fiche() {
        // PE 3, Arme de poing +2 → colonne 5 ligne 2 → 62 ; jet 123 : réussi, z = 3.
        let c = campagne_avec(&["Paulo"]);
        let mut t = test("Paulo", "PE");
        t.talent = Some("arme de poing".into());
        let s = tester_personnage(&c, &t, &mut SequenceDes::new([1, 2, 3])).unwrap();
        assert!(s.action.jet.reussi);
        assert_eq!(s.action.jet.unites, 3);
        assert!(s.description.contains("Arme de poing +2"), "{}", s.description);
    }

    #[test]
    fn test_sans_le_talent_applique_le_malus_du_catalogue() {
        // AG 3 → colonne 5, Acrobatie absente (-3) → colonne 2 ligne 0 → 21.
        let c = campagne_avec(&["Paulo"]);
        let mut t = test("Paulo", "AG");
        t.talent = Some("Acrobatie".into());
        let s = tester_personnage(&c, &t, &mut SequenceDes::new([2, 1, 4])).unwrap();
        assert_eq!(s.action.jet.colonne, 2);
        assert!(s.action.jet.reussi, "21 ≤ 21");
        let s = tester_personnage(&c, &t, &mut SequenceDes::new([2, 2, 4])).unwrap();
        assert!(!s.action.jet.reussi, "22 > 21");
    }

    #[test]
    fn test_prelevement_de_pp_avant_le_jet() {
        let c = campagne_avec(&["Paulo"]);
        let mut t = test("Paulo", "PR");
        t.pouvoir = Some("Télépathie".into());
        t.cout_pp = Some(2);
        let s = tester_personnage(&c, &t, &mut SequenceDes::new([6, 6, 6])).unwrap();
        assert_eq!((s.pp_depenses, s.pp_restants), (2, 6));
        assert!(!s.action.jet.reussi, "le coût est payé même en cas d'échec");
        assert_eq!(c.charger("Paulo").unwrap().pp_courants(), 6);
    }

    #[test]
    fn test_invalide_ne_consomme_rien() {
        let c = campagne_avec(&["Paulo"]);
        let mut t = test("Paulo", "PR");
        t.cout_pp = Some(99);
        assert!(tester_personnage(&c, &t, &mut SequenceDes::new([])).unwrap_err().contains("PP"));
        t.cout_pp = Some(1);
        t.caracteristique = "charisme".into();
        assert!(tester_personnage(&c, &t, &mut SequenceDes::new([])).unwrap_err().contains("inconnue"));
        t.caracteristique = "PR".into();
        t.pouvoir = Some("Feu".into());
        assert!(tester_personnage(&c, &t, &mut SequenceDes::new([])).is_err());
        assert_eq!(c.charger("Paulo").unwrap().pp_courants(), 8);
    }

    #[test]
    fn conflit_entre_deux_fiches() {
        // Paulo (AG 3, Esquive +2 → 62) attaque, Albert se défend de même.
        let c = campagne_avec(&["Paulo", "Albert"]);
        let mut a = test("Paulo", "AG");
        a.talent = Some("Esquive".into());
        let mut d = test("Albert", "AG");
        d.talent = Some("Esquive".into());
        // Paulo 111 → z=1 ; Albert 214 → z=4 : le défenseur l'emporte, écart 3.
        let s = conflit_personnages(&c, &a, &d, &mut SequenceDes::new([1, 1, 1, 2, 1, 4])).unwrap();
        assert_eq!(s.attaquant.action.jet.jet.valeur(), 111);
        assert_eq!(s.defenseur.action.jet.jet.valeur(), 214);
        assert!(s.resume.contains("défenseur l'emporte"), "{}", s.resume);
    }

    #[test]
    fn conflit_preleve_les_pp_des_deux_camps_seulement_si_tout_est_valide() {
        let c = campagne_avec(&["Paulo", "Albert"]);
        let mut a = test("Paulo", "PR");
        a.pouvoir = Some("Télépathie".into());
        a.cout_pp = Some(3);
        let mut d = test("Albert", "VO");
        d.cout_pp = Some(1);
        conflit_personnages(&c, &a, &d, &mut SequenceDes::new([1, 1, 1, 1, 1, 1])).unwrap();
        assert_eq!(c.charger("Paulo").unwrap().pp_courants(), 5);
        assert_eq!(c.charger("Albert").unwrap().pp_courants(), 7);

        d.nom = "Fantôme".into();
        assert!(conflit_personnages(&c, &a, &d, &mut SequenceDes::new([])).is_err());
        assert_eq!(c.charger("Paulo").unwrap().pp_courants(), 5, "rien de prélevé");
    }

    // ---- catalogues ----

    #[test]
    fn chercher_pouvoir_par_code_et_par_texte() {
        let p = ChercherPouvoirParams { livre: LivrePouvoirs::Demon, code: Some(321), requete: None };
        assert_eq!(chercher_pouvoir(&p).unwrap().resultats[0].nom, "Télépathie");
        let p = ChercherPouvoirParams { livre: LivrePouvoirs::Ange, code: None, requete: Some("charme".into()) };
        assert_eq!(chercher_pouvoir(&p).unwrap().total, 2);
        let vide = ChercherPouvoirParams { livre: LivrePouvoirs::Ange, code: None, requete: None };
        assert!(chercher_pouvoir(&vide).is_err());
    }

    #[test]
    fn recherche_libre_plafonnee() {
        let p = ChercherPouvoirParams { livre: LivrePouvoirs::Demon, code: None, requete: Some("a".into()) };
        let s = chercher_pouvoir(&p).unwrap();
        assert!(s.total > MAX_RESULTATS && s.resultats.len() == MAX_RESULTATS);
    }

    #[test]
    fn tirer_pouvoir_aleatoire_et_semi_aleatoire() {
        // D666 = 3 2 1 → pouvoir 321.
        let p = TirerPouvoirParams { livre: LivrePouvoirs::Demon, centaines: None };
        let t = tirer_pouvoir(&p, &mut SequenceDes::new([3, 2, 1])).unwrap();
        assert_eq!((t.methode, t.code), ("aleatoire", 321));
        assert_eq!(t.pouvoir.unwrap().nom, "Télépathie");
        // Centaines choisies = 3, D66 = 2 1 → 321.
        let p = TirerPouvoirParams { livre: LivrePouvoirs::Demon, centaines: Some(3) };
        let t = tirer_pouvoir(&p, &mut SequenceDes::new([2, 1])).unwrap();
        assert_eq!((t.methode, t.code), ("semi_aleatoire", 321));
        let p = TirerPouvoirParams { livre: LivrePouvoirs::Demon, centaines: Some(7) };
        assert!(tirer_pouvoir(&p, &mut SequenceDes::new([])).is_err());
    }

    #[test]
    fn chercher_talent_par_categorie() {
        assert!(chercher_talent("combat").total >= 7);
        assert_eq!(chercher_talent("esquive").resultats[0].malus, Some(-1));
    }

    // ---- journal ----

    #[test]
    fn journal_ajout_et_lecture_filtree() {
        let c = campagne_avec(&["Paulo"]);
        let e = noter_journal(&c, NoterParams {
            texte: "Le concierge est un familier".into(),
            seance: "Séance 1".into(),
            categorie: "pnj".into(),
            personnages: vec!["Paulo".into()],
        })
        .unwrap();
        assert_eq!(e.personnages, ["paulo"]);
        let l = lire_journal(&c, LireJournalParams { personnage: Some("Paulo".into()), ..Default::default() }).unwrap();
        assert_eq!(l.len(), 1);
        let l = lire_journal(&c, LireJournalParams { contient: Some("zzz".into()), ..Default::default() }).unwrap();
        assert!(l.is_empty());
        let vide = NoterParams { texte: " ".into(), seance: String::new(), categorie: String::new(), personnages: vec![] };
        assert!(noter_journal(&c, vide).is_err());
    }
}
