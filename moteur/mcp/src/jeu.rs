//! Outils de campagne : fiches, état des personnages, tests sur fiche, catalogues, journal.
//! Comme [`crate::outils`], cette couche est indépendante du protocole MCP.

use insmv_campagne::{ActionCombat, Campagne, EntreeJournal, ErreurCampagne, Evenement, FiltreJournal, JetUm};
use insmv_regles::catalogue::{self, Livre, Pouvoir, Talent};
use insmv_regles::blessures::{EtatSante, Sante};
use insmv_regles::personnage::{
    controler, Budget, Carac, Caracteristiques, DemandeTest, Espece, Gravite, Personnage,
    PouvoirPossede, Probleme, Role, TalentPossede,
};
use insmv_regles::combat::{
    initiative as ordre_initiative, resoudre_attaque, Circonstance, Initiative, ParamsAttaque, ParamsDefense,
    ResultatAttaque,
};
use insmv_regles::{evaluer, resoudre_conflit as arbitrer, Issue, SourceDes, D666};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::outils::{decrire_action, decrire_issue, SortieAction};

fn erreur(e: ErreurCampagne) -> String {
    e.to_string()
}

/// Trace un événement pour l'interface Web. L'action de jeu a déjà eu lieu et son résultat est
/// en base : un échec d'affichage ne doit pas la faire passer pour ratée.
fn emettre(c: &Campagne, evenement: Evenement) {
    let _ = c.emettre(&evenement);
}

/// Affiche des jets (déjà tirés) sur la table UM de l'interface Web.
pub fn afficher_jets(c: &Campagne, jets: Vec<JetUm>) {
    emettre(c, Evenement::Jets { jets });
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
    pub sante: Sante,
    /// Durée d'inconscience ou de coma tirée au moment où l'état vient d'être atteint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duree_tiree: Option<String>,
}

impl From<&Personnage> for EtatPersonnage {
    fn from(p: &Personnage) -> Self {
        Self {
            nom: p.nom.clone(),
            pp: p.pp_courants(),
            pp_max: p.pp_max(),
            dommages: p.dommages,
            sante: p.sante(),
            duree_tiree: None,
        }
    }
}

/// Tire la durée (« D6 secondes »…) quand un personnage vient de basculer dans un état
/// qui en comporte une.
fn tirer_duree(avant: EtatSante, apres: &Sante, source: &mut impl SourceDes) -> Option<String> {
    let modele = apres.duree?;
    if avant == apres.etat {
        return None;
    }
    let tirage = i32::from(source.d6());
    let unite = modele.trim_start_matches("D6 ");
    Some(format!("{tirage} {unite}"))
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

/// `delta` > 0 inflige des dommages, < 0 en soigne (jamais en dessous de 0). Renvoie
/// l'état de santé (Force perdue, évanouissement, coma, mort) et tire la durée de
/// l'inconscience quand le personnage vient d'y basculer.
pub fn ajuster_dommages(
    c: &Campagne,
    nom: &str,
    delta: i32,
    source: &mut impl SourceDes,
) -> Result<EtatPersonnage, String> {
    c.modifier(nom, |p| {
        let avant = p.sante().etat;
        p.ajuster_dommages(delta);
        Ok(avant)
    })
    .map_err(erreur)?
    .map(|(p, avant)| {
        let mut etat = EtatPersonnage::from(&p);
        etat.duree_tiree = tirer_duree(avant, &etat.sante, source);
        etat
    })
}

/// Soins naturels : une semaine de repos total rend un point de Force.
pub fn soigner_semaines(c: &Campagne, nom: &str, semaines: u32) -> Result<EtatPersonnage, String> {
    c.modifier(nom, |p| {
        p.soigner_semaines(semaines);
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
    let jet = evaluer(&action, D666::tirer(source));
    afficher_jets(c, vec![JetUm { libelle: format!("{} : {description}", t.nom), action, jet }]);
    Ok(SortieTest { description, pp_depenses, pp_restants, action: decrire_action(jet) })
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
    let mut jets_affiches = Vec::new();
    for (t, action, description) in [(attaquant, action_a, desc_a), (defenseur, action_d, desc_d)] {
        let (pp_depenses, pp_restants) = prelever_pp(c, t)?;
        let jet = evaluer(&action, D666::tirer(source));
        jets_affiches.push(JetUm { libelle: format!("{} : {description}", t.nom), action, jet });
        sorties.push(SortieTest { description, pp_depenses, pp_restants, action: decrire_action(jet) });
    }
    afficher_jets(c, jets_affiches);
    let defenseur = sorties.pop().expect("deux sorties");
    let attaquant = sorties.pop().expect("deux sorties");
    let issue = arbitrer(&attaquant.action.jet, &defenseur.action.jet);
    Ok(SortieConflitPersonnages { resume: decrire_issue(&issue), attaquant, defenseur, issue })
}

// ---- Pouvoirs ---------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UsagePouvoirParams {
    /// Nom du personnage qui utilise le pouvoir.
    pub nom: String,
    /// Pouvoir de la fiche : son nom ou son code à trois chiffres.
    pub pouvoir: String,
    /// Cible, si le pouvoir est une attaque ou un effet contesté : elle se défend alors avec la
    /// caractéristique de défense du pouvoir (Volonté au niveau 0 pour un pouvoir psychique).
    #[serde(default)]
    pub cible: Option<String>,
    /// Caractéristique du jet quand le livre ne l'indique pas (FO, PR, VO, PE, AG, AP).
    #[serde(default)]
    pub caracteristique: Option<String>,
    /// Coût en PP quand il est variable (« 1, 2 ou 3/att. », « 1/minute »...). Sinon déduit du livre.
    #[serde(default)]
    pub cout_pp: Option<u32>,
    /// Décalage de colonnes du MJ pour l'utilisateur.
    #[serde(default)]
    pub modificateur: i32,
    /// Malus volontaire du joueur (colonnes), ajouté aux unités.
    #[serde(default)]
    pub malus_volontaire: u8,
    /// Caractéristique de défense de la cible quand le livre ne l'indique pas.
    #[serde(default)]
    pub defense: Option<String>,
    /// Décalage de colonnes du MJ pour la défense de la cible.
    #[serde(default)]
    pub modificateur_defense: i32,
}

#[derive(Debug, Serialize)]
pub struct SortieUsagePouvoir {
    pub pouvoir: String,
    pub code: Option<u16>,
    /// Règle du livre : à appliquer par le MJ en interprétant les unités.
    pub regle: Option<String>,
    pub cout_paye: u32,
    pub pp_restants: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test: Option<SortieTest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflit: Option<SortieConflitPersonnages>,
}

pub fn utiliser_pouvoir(
    c: &Campagne,
    p: &UsagePouvoirParams,
    source: &mut impl SourceDes,
) -> Result<SortieUsagePouvoir, String> {
    let perso = c.charger(&p.nom).map_err(erreur)?;
    let livre = perso
        .espece
        .livre_pouvoirs()
        .ok_or_else(|| format!("{} ne possède pas de pouvoirs", perso.nom))?;
    let possede = match p.pouvoir.trim().parse::<u16>() {
        Ok(code) => perso.pouvoirs.iter().find(|pv| pv.code == Some(code)),
        Err(_) => perso.pouvoir(&p.pouvoir),
    }
    .ok_or_else(|| format!("{} ne possède pas le pouvoir « {} »", perso.nom, p.pouvoir))?;
    if possede.limitation {
        return Err(format!("« {} » est une limitation, pas un pouvoir utilisable", possede.nom));
    }
    let entree = possede.code.and_then(|code| catalogue::pouvoir_par_code(livre, code));

    let cout = match (p.cout_pp, entree.and_then(|e| e.cout_fixe())) {
        (Some(explicite), _) => explicite,
        (None, Some(fixe)) => fixe,
        (None, None) => {
            return Err(format!(
                "le coût de « {} » est variable ({}) : préciser `cout_pp`",
                possede.nom,
                entree.and_then(|e| e.cout.as_deref()).unwrap_or("non imprimé")
            ))
        }
    };
    let carac = p
        .caracteristique
        .clone()
        .or_else(|| entree.and_then(|e| e.jet.clone().or_else(|| e.attaque.clone())))
        .ok_or_else(|| {
            format!("« {} » n'a pas de caractéristique de jet dans le livre : préciser `caracteristique`", possede.nom)
        })?;
    let attaque = TestParams {
        nom: perso.nom.clone(),
        caracteristique: carac,
        talent: None,
        pouvoir: Some(possede.nom.clone()),
        difficulte: 1,
        modificateur: p.modificateur,
        malus_volontaire: p.malus_volontaire,
        malus_sans_talent: None,
        cout_pp: Some(cout),
    };
    let regle = entree.and_then(|e| e.description.clone());
    let (code, nom) = (possede.code, possede.nom.clone());

    if let Some(cible) = &p.cible {
        let carac_def = p
            .defense
            .clone()
            .or_else(|| entree.and_then(|e| e.defense.clone()))
            .ok_or_else(|| {
                format!("« {nom} » n'a pas de caractéristique de défense dans le livre : préciser `defense`")
            })?;
        let defense = TestParams {
            nom: cible.clone(),
            caracteristique: carac_def,
            talent: None,
            pouvoir: None,
            // Défense « au niveau 0 » (jet de Volonté, ou de Force pour certains pouvoirs).
            difficulte: 0,
            modificateur: p.modificateur_defense,
            malus_volontaire: 0,
            malus_sans_talent: None,
            cout_pp: None,
        };
        let conflit = conflit_personnages(c, &attaque, &defense, source)?;
        let pp_restants = conflit.attaquant.pp_restants;
        tracer_pouvoir(c, &perso.nom, Some(cible), &nom, cout, &conflit.resume);
        Ok(SortieUsagePouvoir { pouvoir: nom, code, regle, cout_paye: cout, pp_restants, test: None, conflit: Some(conflit) })
    } else {
        let test = tester_personnage(c, &attaque, source)?;
        let pp_restants = test.pp_restants;
        tracer_pouvoir(c, &perso.nom, None, &nom, cout, &test.action.resume);
        Ok(SortieUsagePouvoir { pouvoir: nom, code, regle, cout_paye: cout, pp_restants, test: Some(test), conflit: None })
    }
}

/// Un pouvoir utilisé pendant un combat figure dans le fil des actions ; hors combat, il ne
/// doit pas en ouvrir un.
fn tracer_pouvoir(c: &Campagne, acteur: &str, cible: Option<&str>, pouvoir: &str, cout: u32, resultat: &str) {
    if c.combat_en_cours().ok().flatten().is_none() {
        return;
    }
    emettre(
        c,
        Evenement::Action(ActionCombat {
            acteur: acteur.to_string(),
            cible: cible.map(str::to_string),
            resume: format!("{acteur} utilise {pouvoir} ({cout} PP) : {resultat}"),
        }),
    );
}

// ---- Combat ---------------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
pub struct DefenseParams {
    /// Talent de défense. Défaut : Esquive. Contre une arme à distance seule l'Esquive est permise ;
    /// au contact : Corps à corps, Arme de contact, Arme de contact lourde ou Esquive selon l'attaque.
    #[serde(default)]
    pub talent: Option<String>,
    /// Caractéristique, si le talent n'a pas de caractéristique usuelle.
    #[serde(default)]
    pub caracteristique: Option<String>,
    /// Arme de parade (catalogue) : sa précision modifie le jet, sa puissance s'ajoute aux unités.
    #[serde(default)]
    pub arme: Option<String>,
    #[serde(default)]
    pub modificateur: i32,
    #[serde(default)]
    pub malus_sans_talent: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AttaqueParams {
    /// Nom de l'attaquant.
    pub attaquant: String,
    /// Nom du défenseur.
    pub defenseur: String,
    /// Talent d'attaque : Corps à corps, Arme de contact, Arme de contact lourde, Arme de poing,
    /// Arme d'épaule, Arme lourde, Lancer...
    pub talent: String,
    /// Caractéristique, si le talent n'a pas de caractéristique usuelle (Spécialisation).
    #[serde(default)]
    pub caracteristique: Option<String>,
    /// Arme du catalogue (ex. « Epée longue », « Carabine »). Donne puissance et précision.
    #[serde(default)]
    pub arme: Option<String>,
    /// Remplace la puissance de l'arme (armes spéciales, objets improvisés).
    #[serde(default)]
    pub puissance: Option<i32>,
    /// Ajoute à la puissance (ex. +1 du pouvoir 413 Arme de contact intelligente).
    #[serde(default)]
    pub bonus_puissance: i32,
    /// Remplace la précision de l'arme (en colonnes).
    #[serde(default)]
    pub precision: Option<i32>,
    /// Circonstances de la table des modificateurs d'attaque.
    #[serde(default)]
    pub circonstances: Vec<Circonstance>,
    /// Décalage de colonnes supplémentaire du MJ (bonus > 0, malus < 0).
    #[serde(default)]
    pub modificateur: i32,
    /// Malus volontaire du joueur (colonnes), ajouté aux unités donc aux dommages.
    #[serde(default)]
    pub malus_volontaire: u8,
    #[serde(default)]
    pub malus_sans_talent: Option<u8>,
    /// Armure de l'attaquant (Cuir, Cotte de maille, Kevlar, Acier) : malus d'Agilité.
    #[serde(default)]
    pub armure_attaquant: Option<String>,
    #[serde(default)]
    pub defense: DefenseParams,
    /// La cible ne voit pas venir l'attaque : aucune défense.
    #[serde(default)]
    pub defense_impossible: bool,
    /// Armure du défenseur : protection retranchée aux dommages, malus d'Agilité à l'esquive.
    #[serde(default)]
    pub armure_defenseur: Option<String>,
    /// Protection supplémentaire (bouclier, pouvoir, objet) en points.
    #[serde(default)]
    pub protection_supplementaire: i32,
    /// Applique les dommages à la fiche du défenseur. Défaut : oui.
    #[serde(default = "vrai")]
    pub appliquer: bool,
}

fn vrai() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct SortieAttaque {
    pub attaquant: String,
    pub defenseur: String,
    #[serde(flatten)]
    pub resultat: ResultatAttaque,
    pub resume: String,
    /// État du défenseur après application des dommages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etat_defenseur: Option<EtatPersonnage>,
}

fn carac_optionnelle(texte: &Option<String>) -> Result<Option<Carac>, String> {
    texte
        .as_deref()
        .map(|t| Carac::analyser(t).ok_or_else(|| format!("caractéristique inconnue : « {t} »")))
        .transpose()
}

fn resume_attaque(r: &ResultatAttaque, att: &str, def: &str) -> String {
    let mut lignes = vec![format!(
        "{att} attaque {def} ({}) : {} → {}",
        match r.type_attaque {
            insmv_regles::combat::TypeAttaque::Contact => "contact",
            insmv_regles::combat::TypeAttaque::Distance => "distance",
        },
        r.attaque.jet.jet.valeur(),
        if r.attaque.jet.reussi { "touche" } else { "raté" },
    )];
    match (&r.defense, &r.defense_absente) {
        (Some(d), _) => lignes.push(format!(
            "{def} se défend : {} → {}",
            d.jet.jet.valeur(),
            if d.jet.reussi { "réussi" } else { "raté" }
        )),
        (None, Some(raison)) => lignes.push(format!("{def} ne se défend pas : {raison}")),
        _ => {}
    }
    lignes.push(decrire_issue(&r.issue));
    if let Some(d) = &r.dommages {
        let detail = format!(
            "{} unités + {} puissance + {} Force - {} protection",
            d.unites, d.puissance, d.bonus_force, d.protection
        );
        lignes.push(if r.arbitrage_mj {
            format!("Si le MJ donne l'avantage à l'attaquant : {} dommage(s) ({detail}).", d.total)
        } else {
            format!("Dommages : {} ({detail}).", d.total)
        });
    }
    lignes.join("\n")
}

fn jet_um(nom: &str, detail: &insmv_regles::combat::DetailJet) -> JetUm {
    JetUm { libelle: format!("{nom} : {}", detail.description), action: detail.action, jet: detail.jet }
}

pub fn attaquer(
    c: &Campagne,
    p: &AttaqueParams,
    source: &mut impl SourceDes,
) -> Result<SortieAttaque, String> {
    let att = c.charger(&p.attaquant).map_err(erreur)?;
    let def = c.charger(&p.defenseur).map_err(erreur)?;
    if att.id == def.id {
        return Err("l'attaquant et le défenseur sont le même personnage".into());
    }
    let params = ParamsAttaque {
        talent: p.talent.clone(),
        caracteristique: carac_optionnelle(&p.caracteristique)?,
        arme: p.arme.clone(),
        puissance: p.puissance,
        bonus_puissance: p.bonus_puissance,
        precision: p.precision,
        circonstances: p.circonstances.clone(),
        modificateur: p.modificateur,
        malus_volontaire: p.malus_volontaire,
        malus_sans_talent: p.malus_sans_talent,
        armure_attaquant: p.armure_attaquant.clone(),
        defense: ParamsDefense {
            talent: p.defense.talent.clone(),
            caracteristique: carac_optionnelle(&p.defense.caracteristique)?,
            arme: p.defense.arme.clone(),
            modificateur: p.defense.modificateur,
            malus_sans_talent: p.defense.malus_sans_talent,
        },
        defense_impossible: p.defense_impossible,
        armure_defenseur: p.armure_defenseur.clone(),
        protection_supplementaire: p.protection_supplementaire,
    };
    let resultat = resoudre_attaque(&att, &def, &params, source)?;
    let mut resume = resume_attaque(&resultat, &att.nom, &def.nom);

    let a_appliquer = resultat.dommages_a_appliquer();
    let etat_defenseur = if p.appliquer && a_appliquer > 0 {
        let delta = i32::try_from(a_appliquer).map_err(|_| "dommages démesurés".to_string())?;
        let etat = ajuster_dommages(c, &def.nom, delta, source)?;
        resume.push_str(&format!(
            "\n{} : {} dommage(s) au total, Force {} ({:?}){}.",
            etat.nom,
            etat.dommages,
            etat.sante.force_courante,
            etat.sante.etat,
            etat.duree_tiree.as_ref().map(|d| format!(", inconscient {d}")).unwrap_or_default()
        ));
        Some(etat)
    } else {
        None
    };
    let mut jets = vec![jet_um(&att.nom, &resultat.attaque)];
    jets.extend(resultat.defense.as_ref().map(|d| jet_um(&def.nom, d)));
    afficher_jets(c, jets);
    emettre(
        c,
        Evenement::Action(ActionCombat {
            acteur: att.nom.clone(),
            cible: Some(def.nom.clone()),
            resume: resume.clone(),
        }),
    );
    Ok(SortieAttaque { attaquant: att.nom, defenseur: def.nom, resultat, resume, etat_defenseur })
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct InitiativeParams {
    /// Noms des personnages engagés dans le combat.
    pub personnages: Vec<String>,
}

pub fn initiative(
    c: &Campagne,
    p: &InitiativeParams,
    source: &mut impl SourceDes,
) -> Result<Initiative, String> {
    let fiches = p
        .personnages
        .iter()
        .map(|n| c.charger(n).map_err(erreur))
        .collect::<Result<Vec<_>, _>>()?;
    if fiches.len() < 2 {
        return Err("l'initiative demande au moins deux personnages".into());
    }
    let ordre = ordre_initiative(&fiches, source);
    let numero = match c.combat_en_cours().map_err(erreur)? {
        Some(combat) => combat.tours.iter().map(|t| t.numero).max().unwrap_or(0) + 1,
        None => 1,
    };
    emettre(c, Evenement::Tour { numero, initiative: ordre.clone() });
    Ok(ordre)
}

#[derive(Debug, Serialize)]
pub struct SortieFinCombat {
    pub resume: String,
}

/// Clôt le combat affiché : le prochain `initiative` ouvre un nouveau combat au tour 1.
pub fn terminer_combat(c: &Campagne) -> Result<SortieFinCombat, String> {
    let en_cours = c.combat_en_cours().map_err(erreur)?;
    c.emettre(&Evenement::Fin).map_err(erreur)?;
    Ok(SortieFinCombat {
        resume: match en_cours {
            Some(combat) => format!("Combat terminé après {} tour(s).", combat.tours.iter().filter(|t| t.numero > 0).count()),
            None => "Aucun combat en cours.".to_string(),
        },
    })
}

/// Colonnes de décalage proposées au joueur : de -3 à +3 autour de la colonne de base.
const DECALAGES_PROPOSES: std::ops::RangeInclusive<i32> = -3..=3;

#[derive(Debug, Serialize)]
pub struct SeuilDecale {
    pub decalage: i32,
    pub colonne: i32,
    pub seuil: insmv_regles::Seuil,
    pub chances_pct: u8,
}

#[derive(Debug, Serialize)]
pub struct SortieAnnonce {
    pub description: String,
    pub colonne_base: i32,
    pub niveau: u8,
    pub seuils: Vec<SeuilDecale>,
    pub resume: String,
}

/// Annonce une action sans la lancer : la table UM de l'interface Web montre la colonne de base
/// et ce que donnerait chaque décalage, pour que le joueur puisse en décider.
pub fn annoncer_action(c: &Campagne, t: &TestParams) -> Result<SortieAnnonce, String> {
    let (action, description) = preparer(c, t)?;
    let seuils: Vec<SeuilDecale> = DECALAGES_PROPOSES
        .map(|d| {
            let decale = action.avec_modificateur(d);
            let seuil = decale.seuil();
            SeuilDecale { decalage: d, colonne: decale.colonne(), seuil, chances_pct: seuil.pourcentage() }
        })
        .collect();
    emettre(c, Evenement::Attente { libelle: format!("{} : {description}", t.nom), action });
    let resume = format!(
        "{description} : colonne {}, ligne {}. Avec décalage : {}.",
        action.colonne(),
        action.niveau.valeur(),
        seuils.iter().map(|s| format!("{:+} → {} %", s.decalage, s.chances_pct)).collect::<Vec<_>>().join(", ")
    );
    Ok(SortieAnnonce {
        description,
        colonne_base: action.colonne(),
        niveau: action.niveau.valeur(),
        seuils,
        resume,
    })
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

/// Au-delà de ce nombre de résultats, les textes du livre sont omis (rechercher par code pour les lire).
const MAX_RESULTATS_AVEC_TEXTE: usize = 5;

pub fn chercher_pouvoir(p: &ChercherPouvoirParams) -> Result<SortieRecherche<Pouvoir>, String> {
    let livre = Livre::from(p.livre);
    let mut sortie = match (p.code, &p.requete) {
        (Some(code), _) => borner(catalogue::pouvoir_par_code(livre, code).into_iter().collect()),
        (None, Some(q)) => borner(catalogue::chercher_pouvoirs(livre, q)),
        (None, None) => return Err("indiquer `code` ou `requete`".into()),
    };
    if sortie.total > MAX_RESULTATS_AVEC_TEXTE {
        for r in &mut sortie.resultats {
            r.description = None;
        }
    }
    Ok(sortie)
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
        ajuster_dommages(&c, "Albert", 2, &mut SequenceDes::new([])).unwrap();
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
        let mut des = SequenceDes::new([]);
        assert_eq!(ajuster_dommages(&c, "Paulo", 5, &mut des).unwrap().dommages, 5);
        assert_eq!(ajuster_dommages(&c, "Paulo", -9, &mut des).unwrap().dommages, 0);
    }

    #[test]
    fn dommages_etat_de_sante_et_duree_tiree_a_la_bascule() {
        // Démon de Force 3 : assommé à -2 (10 dommages), mort à -3 (12 dommages).
        let c = campagne_avec(&["Paulo"]);
        let e = ajuster_dommages(&c, "Paulo", 9, &mut SequenceDes::new([])).unwrap();
        assert_eq!((e.sante.etat, e.sante.force_courante), (EtatSante::Valide, -1));
        assert_eq!(e.duree_tiree, None);

        // 10e dommage : bascule en inconscience, durée D6 secondes tirée (4).
        let e = ajuster_dommages(&c, "Paulo", 1, &mut SequenceDes::new([4])).unwrap();
        assert_eq!(e.sante.etat, EtatSante::Inconscient);
        assert_eq!(e.duree_tiree.as_deref(), Some("4 secondes"));

        // Dommage de plus sans changement d'état : pas de nouveau tirage.
        let e = ajuster_dommages(&c, "Paulo", 1, &mut SequenceDes::new([])).unwrap();
        assert_eq!(e.duree_tiree, None);

        let e = ajuster_dommages(&c, "Paulo", 1, &mut SequenceDes::new([])).unwrap();
        assert_eq!(e.sante.etat, EtatSante::Mort);
    }

    #[test]
    fn soigner_des_semaines() {
        let c = campagne_avec(&["Paulo"]);
        ajuster_dommages(&c, "Paulo", 7, &mut SequenceDes::new([])).unwrap();
        let e = soigner_semaines(&c, "Paulo", 2).unwrap();
        assert_eq!(e.dommages, 3, "un Démon récupère 2 dommages par semaine");
        assert!(soigner_semaines(&c, "Inconnu", 1).is_err());
    }

    #[test]
    fn un_personnage_inconscient_ne_peut_pas_etre_teste() {
        let c = campagne_avec(&["Paulo"]);
        ajuster_dommages(&c, "Paulo", 10, &mut SequenceDes::new([2])).unwrap();
        let err = tester_personnage(&c, &test("Paulo", "PE"), &mut SequenceDes::new([])).unwrap_err();
        assert!(err.contains("inconscient"), "{err}");
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

    // ---- combat ----

    fn campagne_de_combat() -> Campagne {
        let c = Campagne::en_memoire().unwrap();
        creer_personnage(&c, CreationParams {
            nom: "Jean".into(),
            espece: Espece::Ange,
            role: Role::Joueur,
            caracteristiques: carac(4, 4, 5, 2, 2, 3),
            talents: vec![
                TalentPossede { nom: "Arme de contact".into(), niveau: 2 },
                TalentPossede { nom: "Esquive".into(), niveau: 1 },
                TalentPossede { nom: "Corps à corps".into(), niveau: 0 },
                TalentPossede { nom: "Discrétion".into(), niveau: 0 },
            ],
            pouvoirs: [413, 166, 243, 321, 263, 622]
                .map(|code| PouvoirPossede { code: Some(code), nom: String::new(), niveau: 0, limitation: false })
                .into(),
            patron: Some("Laurent".into()),
            equipement: vec![],
            notes: String::new(),
            exception: None,
            remplacer: false,
        })
        .unwrap();
        creer_personnage(&c, CreationParams {
            nom: "Zombi".into(),
            espece: Espece::Humain,
            role: Role::Pnj,
            caracteristiques: carac(2, 1, 2, 2, 2, 1),
            talents: vec![TalentPossede { nom: "Esquive".into(), niveau: 0 }],
            pouvoirs: vec![],
            patron: None,
            equipement: vec![],
            notes: String::new(),
            exception: None,
            remplacer: false,
        })
        .unwrap();
        c
    }

    fn coup_d_epee() -> AttaqueParams {
        AttaqueParams {
            attaquant: "Jean".into(),
            defenseur: "Zombi".into(),
            talent: "Arme de contact".into(),
            caracteristique: None,
            arme: Some("Epée longue".into()),
            puissance: None,
            bonus_puissance: 1,
            precision: None,
            circonstances: vec![],
            modificateur: 0,
            malus_volontaire: 0,
            malus_sans_talent: None,
            armure_attaquant: None,
            defense: DefenseParams::default(),
            defense_impossible: false,
            armure_defenseur: None,
            protection_supplementaire: 0,
            appliquer: true,
        }
    }

    #[test]
    fn attaque_applique_les_dommages_et_renvoie_l_etat() {
        let c = campagne_de_combat();
        // Attaque 435 (touche, unités 5), défense 666 (ratée) : 5 + 3 + 1 = 9 dommages.
        let s = attaquer(&c, &coup_d_epee(), &mut SequenceDes::new([4, 3, 5, 6, 6, 6])).unwrap();
        assert_eq!(s.resultat.dommages.as_ref().unwrap().total, 9);
        let etat = s.etat_defenseur.unwrap();
        assert_eq!((etat.dommages, etat.sante.etat), (9, EtatSante::Mort));
        assert_eq!(c.charger("Zombi").unwrap().dommages, 9, "persisté");
        assert!(s.resume.contains("Dommages : 9") && s.resume.contains("Mort"), "{}", s.resume);
    }

    #[test]
    fn attaque_sans_application_ne_touche_pas_a_la_fiche() {
        let c = campagne_de_combat();
        let mut p = coup_d_epee();
        p.appliquer = false;
        let s = attaquer(&c, &p, &mut SequenceDes::new([4, 3, 5, 6, 6, 6])).unwrap();
        assert!(s.etat_defenseur.is_none());
        assert_eq!(c.charger("Zombi").unwrap().dommages, 0);
    }

    #[test]
    fn egalite_des_unites_n_applique_rien() {
        let c = campagne_de_combat();
        // Attaque unités 3, défense Esquive ≤ 21 : 113 → unités 3. Égalité.
        let s = attaquer(&c, &coup_d_epee(), &mut SequenceDes::new([4, 3, 3, 1, 1, 3])).unwrap();
        assert!(s.resultat.arbitrage_mj && s.etat_defenseur.is_none());
        assert_eq!(c.charger("Zombi").unwrap().dommages, 0);
        assert!(s.resume.contains("le MJ"), "{}", s.resume);
    }

    #[test]
    fn dommages_a_la_bascule_tirent_la_duree_d_inconscience() {
        let c = campagne_de_combat();
        // Zombi (humain, Force 2) avec 2 dommages : évanoui. Dés : attaque 4 3 2 (unités 2), défense 666 ratée,
        // dommages = 2 + 3 + 1 = 6 → mort directement. Un coup léger : bonus négatif via protection.
        let mut p = coup_d_epee();
        p.protection_supplementaire = 4; // 2 + 3 + 1 - 4 = 2 dommages
        let s = attaquer(&c, &p, &mut SequenceDes::new([4, 3, 2, 6, 6, 6, 5])).unwrap();
        let etat = s.etat_defenseur.unwrap();
        assert_eq!(etat.sante.etat, EtatSante::Inconscient);
        assert_eq!(etat.duree_tiree.as_deref(), Some("5 minutes"));
    }

    #[test]
    fn attaquer_soi_meme_ou_un_inconnu_est_refuse() {
        let c = campagne_de_combat();
        let mut p = coup_d_epee();
        p.defenseur = "Jean".into();
        assert!(attaquer(&c, &p, &mut SequenceDes::new([])).unwrap_err().contains("même personnage"));
        p.defenseur = "Fantôme".into();
        assert!(attaquer(&c, &p, &mut SequenceDes::new([])).unwrap_err().contains("introuvable"));
        let mut p = coup_d_epee();
        p.caracteristique = Some("charisme".into());
        assert!(attaquer(&c, &p, &mut SequenceDes::new([])).unwrap_err().contains("inconnue"));
    }

    #[test]
    fn initiative_depuis_la_campagne() {
        let c = campagne_de_combat();
        let p = InitiativeParams { personnages: vec!["Jean".into(), "Zombi".into()] };
        let i = initiative(&c, &p, &mut SequenceDes::new([])).unwrap();
        assert_eq!(i.physique[0].personnages, ["Jean"], "Agilité 5 contre 2");
        assert_eq!(i.psychique.len(), 1, "seul Jean a des pouvoirs");
        let seul = InitiativeParams { personnages: vec!["Jean".into()] };
        assert!(initiative(&c, &seul, &mut SequenceDes::new([])).is_err());
    }

    // ---- affichage Web ----

    #[test]
    fn annoncer_une_action_n_affiche_que_l_attente() {
        let c = campagne_de_combat();
        let mut t = test("Jean", "AG");
        t.talent = Some("Esquive".into());
        let a = annoncer_action(&c, &t).unwrap();
        // AG 5 → colonne 13, ligne +1.
        assert_eq!((a.colonne_base, a.niveau), (13, 1));
        assert_eq!(a.seuils.len(), 7);
        assert_eq!((a.seuils[0].decalage, a.seuils[0].colonne), (-3, 10));
        assert_eq!(a.seuils[3].chances_pct, 100);
        assert!(matches!(c.derniere_resolution().unwrap().unwrap().evenement, Evenement::Attente { .. }));
        assert!(c.derniers_jets(5).unwrap().is_empty(), "aucun dé lancé");
        assert_eq!(c.charger("Jean").unwrap().pp_courants(), 10, "rien prélevé");
    }

    #[test]
    fn un_test_remplace_l_attente_par_son_jet() {
        let c = campagne_de_combat();
        let mut t = test("Jean", "AG");
        t.talent = Some("Esquive".into());
        annoncer_action(&c, &t).unwrap();
        t.modificateur = -2;
        tester_personnage(&c, &t, &mut SequenceDes::new([2, 3, 4])).unwrap();
        let Evenement::Jets { jets } = c.derniere_resolution().unwrap().unwrap().evenement else {
            panic!("le jet doit remplacer l'attente");
        };
        assert_eq!(jets.len(), 1);
        assert_eq!((jets[0].action.colonne_base(), jets[0].jet.colonne), (13, 11));
        assert!(jets[0].libelle.starts_with("Jean"), "{}", jets[0].libelle);
    }

    #[test]
    fn un_conflit_affiche_deux_jets_et_une_attaque_les_deux_plus_une_action() {
        let c = campagne_de_combat();
        let mut a = test("Jean", "AG");
        a.talent = Some("Esquive".into());
        let mut d = test("Zombi", "AG");
        d.difficulte = 1;
        conflit_personnages(&c, &a, &d, &mut SequenceDes::new([1, 1, 1, 1, 1, 1])).unwrap();
        let Evenement::Jets { jets } = c.derniere_resolution().unwrap().unwrap().evenement else { panic!() };
        assert_eq!(jets.len(), 2);
        assert!(c.combat_en_cours().unwrap().is_none(), "un conflit n'ouvre pas de combat");

        attaquer(&c, &coup_d_epee(), &mut SequenceDes::new([4, 3, 5, 6, 6, 6])).unwrap();
        let Evenement::Jets { jets } = c.derniere_resolution().unwrap().unwrap().evenement else { panic!() };
        assert_eq!(jets.len(), 2, "attaque et défense");
        let combat = c.combat_en_cours().unwrap().unwrap();
        assert_eq!(combat.tours[0].actions[0].acteur, "Jean");
        assert_eq!(combat.tours[0].actions[0].cible.as_deref(), Some("Zombi"));
    }

    #[test]
    fn initiative_numerote_les_tours_et_terminer_combat_repart_de_zero() {
        let c = campagne_de_combat();
        let p = InitiativeParams { personnages: vec!["Jean".into(), "Zombi".into()] };
        initiative(&c, &p, &mut SequenceDes::new([])).unwrap();
        initiative(&c, &p, &mut SequenceDes::new([])).unwrap();
        let numeros: Vec<u32> = c.combat_en_cours().unwrap().unwrap().tours.iter().map(|t| t.numero).collect();
        assert_eq!(numeros, [1, 2]);
        assert!(terminer_combat(&c).unwrap().resume.contains("2 tour(s)"));
        assert!(c.combat_en_cours().unwrap().is_none());
        assert!(terminer_combat(&c).unwrap().resume.contains("Aucun combat"));
        initiative(&c, &p, &mut SequenceDes::new([])).unwrap();
        assert_eq!(c.combat_en_cours().unwrap().unwrap().tours[0].numero, 1);
    }

    #[test]
    fn un_pouvoir_n_entre_dans_le_fil_que_pendant_un_combat() {
        let c = campagne_de_combat();
        utiliser_pouvoir(&c, &usage("Jean", "243"), &mut SequenceDes::new([1, 1, 1])).unwrap();
        assert!(c.combat_en_cours().unwrap().is_none());
        let p = InitiativeParams { personnages: vec!["Jean".into(), "Zombi".into()] };
        initiative(&c, &p, &mut SequenceDes::new([])).unwrap();
        utiliser_pouvoir(&c, &usage("Jean", "243"), &mut SequenceDes::new([1, 1, 1])).unwrap();
        let tour = &c.combat_en_cours().unwrap().unwrap().tours[0];
        assert!(tour.actions[0].resume.contains("Jean utilise"), "{}", tour.actions[0].resume);
    }

    // ---- pouvoirs ----

    fn usage(nom: &str, pouvoir: &str) -> UsagePouvoirParams {
        UsagePouvoirParams {
            nom: nom.into(),
            pouvoir: pouvoir.into(),
            cible: None,
            caracteristique: None,
            cout_pp: None,
            modificateur: 0,
            malus_volontaire: 0,
            defense: None,
            modificateur_defense: 0,
        }
    }

    #[test]
    fn utiliser_un_pouvoir_preleve_le_cout_du_livre_et_donne_la_regle() {
        let c = campagne_de_combat(); // Jean : VO 4, PP 10
        // Agilité (243) : jet de Volonté 4 niveau 0 → 32 ; coût 1. Jet 213 : 21 ≤ 32 réussi, unités 3.
        let s = utiliser_pouvoir(&c, &usage("Jean", "243"), &mut SequenceDes::new([2, 1, 3])).unwrap();
        assert_eq!((s.cout_paye, s.pp_restants), (1, 9));
        let t = s.test.unwrap();
        assert!(t.action.jet.reussi && t.action.jet.unites == 3);
        assert!(s.regle.unwrap().contains("(2+niveau"), "texte du livre transmis");
        assert_eq!(c.charger("Jean").unwrap().pp_courants(), 9);
    }

    #[test]
    fn pouvoir_designe_par_nom_et_gratuit_quand_aucun_cout_imprime() {
        let c = campagne_de_combat();
        let mut p = usage("Jean", "Arme de contact intelligente");
        p.caracteristique = Some("AG".into());
        let s = utiliser_pouvoir(&c, &p, &mut SequenceDes::new([1, 1, 1])).unwrap();
        assert_eq!((s.cout_paye, s.pp_restants), (0, 10));
    }

    #[test]
    fn pouvoir_contre_une_cible_defense_au_niveau_0() {
        let c = campagne_de_combat();
        // Calme (166) : VO contre VO. Jean (VO 4, niveau 0 → 32) : 213 réussi, unités 3.
        // Zombi (VO 1, niveau 0 → 13) : 666 raté. La défense échoue : le pouvoir agit pleinement.
        let mut p = usage("Jean", "Calme");
        p.cible = Some("Zombi".into());
        let s = utiliser_pouvoir(&c, &p, &mut SequenceDes::new([2, 1, 3, 6, 6, 6])).unwrap();
        let conflit = s.conflit.unwrap();
        assert_eq!(conflit.issue, Issue::DefenseSansEffet { unites: 3 });
        assert_eq!(conflit.defenseur.action.jet.seuil, insmv_regles::Seuil::Valeur(13));
        assert_eq!(s.pp_restants, 9);
        assert!(s.regle.unwrap().contains("discuter"));
    }

    #[test]
    fn pouvoir_non_possede_limitation_ou_informations_manquantes() {
        let c = campagne_de_combat();
        assert!(utiliser_pouvoir(&c, &usage("Jean", "Eclair"), &mut SequenceDes::new([])).unwrap_err().contains("ne possède pas"));
        assert!(utiliser_pouvoir(&c, &usage("Jean", "622"), &mut SequenceDes::new([])).unwrap_err().contains("limitation"));
        assert!(utiliser_pouvoir(&c, &usage("Zombi", "Calme"), &mut SequenceDes::new([])).unwrap_err().contains("pas de pouvoirs"));
        // Télépathie n'a pas de caractéristique de jet dans le livre.
        let err = utiliser_pouvoir(&c, &usage("Jean", "Télépathie"), &mut SequenceDes::new([])).unwrap_err();
        assert!(err.contains("caractéristique"), "{err}");
        // Agilité (243) n'a pas de caractéristique de défense.
        let mut p = usage("Jean", "243");
        p.cible = Some("Zombi".into());
        assert!(utiliser_pouvoir(&c, &p, &mut SequenceDes::new([])).unwrap_err().contains("défense"));
        assert_eq!(c.charger("Jean").unwrap().pp_courants(), 10, "aucun PP prélevé en cas d'erreur");
    }

    #[test]
    fn cout_explicite_pour_un_pouvoir_a_cout_variable() {
        let c = Campagne::en_memoire().unwrap();
        let mut p = demon("Mage");
        // Table des Démons : 121 = Feu, coût « 1, 2 ou 3/att. ».
        p.pouvoirs[0].code = Some(121);
        creer_personnage(&c, p).unwrap();
        let mut u = usage("Mage", "121");
        u.caracteristique = Some("PR".into());
        assert!(utiliser_pouvoir(&c, &u, &mut SequenceDes::new([])).unwrap_err().contains("cout_pp"));
        u.cout_pp = Some(3);
        let s = utiliser_pouvoir(&c, &u, &mut SequenceDes::new([1, 1, 1])).unwrap();
        assert_eq!(s.cout_paye, 3);
    }

    #[test]
    fn recherche_de_pouvoirs_omet_les_textes_quand_les_resultats_sont_nombreux() {
        let un = chercher_pouvoir(&ChercherPouvoirParams { livre: LivrePouvoirs::Ange, code: Some(166), requete: None }).unwrap();
        assert!(un.resultats[0].description.as_deref().unwrap().contains("discuter"));
        let beaucoup = chercher_pouvoir(&ChercherPouvoirParams { livre: LivrePouvoirs::Ange, code: None, requete: Some("a".into()) }).unwrap();
        assert!(beaucoup.resultats.iter().all(|r| r.description.is_none()));
    }
}
