//! Combat physique (`livres/commun-09-confrontation-armes-combat.md`).
//!
//! Une attaque est un conflit entre le talent d'attaque et un talent de défense. Les
//! dommages sont : unités finales du conflit + puissance de l'arme - protection
//! (+ bonus de Force au contact). Ce module est pur : les fiches sont passées en
//! paramètre et le résultat décrit ce qui s'est passé, sans rien modifier.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::blessures::EtatSante;
use crate::catalogue::{self, Arme, CategorieArme, Protection};
use crate::conflit::{resoudre_conflit, Issue};
use crate::des::{SourceDes, D666};
use crate::personnage::{Carac, DemandeTest, Personnage};
use crate::table_um::{evaluer, ResultatJet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeAttaque {
    Contact,
    Distance,
}

/// Modificateurs d'attaque de la table « Modificateurs au jet d'attaque ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Circonstance {
    /// Cible surprise : +1 au contact, et elle ne peut pas esquiver.
    CibleSurprise,
    /// Cible de dos : +1 au contact, et elle ne peut pas esquiver.
    CibleDeDos,
    /// Cible mobile : -1 à distance.
    CibleMobile,
    /// Attaquant mobile : -1 au contact et à distance.
    AttaquantMobile,
    /// Cible dans la portée de l'arme : +1 à distance.
    DansLaPortee,
    /// Entre une et deux fois la portée : aucun modificateur (à distance).
    PorteeDouble,
    /// Entre deux et cinq fois la portée : -1 à distance.
    PorteeLointaine,
    /// Avoir visé un tour : +1.
    VisePendantUnTour,
    /// Avoir visé deux tours : +2 à distance.
    VisePendantDeuxTours,
    /// Avoir visé trois tours ou plus : +3 à distance.
    ViseTroisToursEtPlus,
    /// Nuit : -1 au contact, -2 à distance.
    Nuit,
    /// Tir depuis une voiture : +1 au contact, +2 à distance.
    Voiture,
    /// Attaquant en contrebas : -1 au contact.
    AttaquantPlusBas,
    /// Attaquant en surplomb : +1 au contact.
    AttaquantPlusHaut,
}

impl Circonstance {
    /// Décalage en colonnes de la table UM.
    pub fn colonnes(self, type_attaque: TypeAttaque) -> i32 {
        use Circonstance::*;
        use TypeAttaque::{Contact, Distance};
        match (self, type_attaque) {
            (CibleSurprise | CibleDeDos, Contact) => 1,
            (CibleSurprise | CibleDeDos, Distance) => 0,
            (CibleMobile, Distance) => -1,
            (AttaquantMobile, _) => -1,
            (DansLaPortee, Distance) => 1,
            (PorteeLointaine, Distance) => -1,
            (VisePendantUnTour, _) => 1,
            (VisePendantDeuxTours, Distance) => 2,
            (ViseTroisToursEtPlus, Distance) => 3,
            (Nuit, Contact) => -1,
            (Nuit, Distance) => -2,
            (Voiture, Contact) => 1,
            (Voiture, Distance) => 2,
            (AttaquantPlusBas, Contact) => -1,
            (AttaquantPlusHaut, Contact) => 1,
            _ => 0,
        }
    }

    /// La cible ne peut pas esquiver (« Pas d'esquive »).
    pub fn interdit_la_defense(self) -> bool {
        matches!(self, Circonstance::CibleSurprise | Circonstance::CibleDeDos)
    }
}

// ---- Talents de combat ----------------------------------------------------------

const TALENTS_DE_CONTACT: [&str; 3] = ["corps a corps", "arme de contact", "arme de contact lourde"];
const TALENTS_A_FEU: [&str; 3] = ["arme de poing", "arme d epaule", "arme lourde"];

fn est_talent_de_contact(talent: &str) -> bool {
    TALENTS_DE_CONTACT.contains(&catalogue::normaliser(talent).as_str())
}

/// Caractéristique associée à un talent de combat (attaque ou défense), d'après la
/// liste des talents. `None` pour une spécialisation : le MJ doit la préciser.
pub fn carac_talent_combat(talent: &str) -> Option<Carac> {
    match catalogue::normaliser(talent).as_str() {
        "arme de poing" | "arme d epaule" | "arme lourde" | "mitrailleuse" | "lance grenade"
        | "lance flammes" => Some(Carac::PE),
        "lancer" => Some(Carac::PR),
        "corps a corps" | "arme de contact" | "esquive" => Some(Carac::AG),
        "arme de contact lourde" => Some(Carac::FO),
        _ => None,
    }
}

/// Table « Talent ou pouvoir / Défense » : talents de défense permis face à une attaque.
pub fn defense_autorisee(talent_attaque: &str, talent_defense: &str) -> bool {
    let defense = catalogue::normaliser(talent_defense);
    let permis: &[&str] = match catalogue::normaliser(talent_attaque).as_str() {
        "corps a corps" => &["corps a corps", "arme de contact", "arme de contact lourde", "esquive"],
        "arme de contact" => &["arme de contact", "arme de contact lourde", "esquive"],
        "arme de contact lourde" => &["arme de contact lourde", "esquive"],
        // Toute arme à distance : seule l'esquive.
        _ => &["esquive"],
    };
    permis.contains(&defense.as_str())
}

/// Bonus aux dommages d'une force surhumaine : Force - 3 à partir de 4, en attaque
/// de contact seulement (jamais en parade).
pub fn bonus_force(fo: u8, type_attaque: TypeAttaque) -> i32 {
    if type_attaque == TypeAttaque::Contact && fo >= 4 {
        i32::from(fo) - 3
    } else {
        0
    }
}

/// Protection naturelle du pouvoir « Armure corporelle » (211) : 2 au niveau 0, +1 par niveau.
pub fn protection_naturelle(p: &Personnage) -> i32 {
    p.pouvoirs
        .iter()
        .find(|pv| pv.code == Some(211))
        .map_or(0, |pv| 2 + i32::from(pv.niveau))
}

/// Protection d'une armure face à ce type d'attaque (le Kevlar protège de 4 contre les armes à feu).
pub fn protection_contre(protection: &Protection, talent_attaque: &str) -> i32 {
    let a_feu = TALENTS_A_FEU.contains(&catalogue::normaliser(talent_attaque).as_str());
    if protection.nom == "Kevlar" && a_feu {
        4
    } else {
        protection.protection
    }
}

// ---- Paramètres et résultat ------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ParamsDefense {
    /// Talent de défense ; « Esquive » par défaut.
    pub talent: Option<String>,
    pub caracteristique: Option<Carac>,
    /// Arme utilisée pour parer : sa précision modifie le jet et sa puissance s'ajoute aux unités.
    pub arme: Option<String>,
    pub modificateur: i32,
    pub malus_sans_talent: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct ParamsAttaque {
    pub talent: String,
    pub caracteristique: Option<Carac>,
    pub arme: Option<String>,
    /// Remplace la puissance de l'arme (armes spéciales, objets improvisés).
    pub puissance: Option<i32>,
    /// S'ajoute à la puissance (ex. +1 du pouvoir Arme de contact intelligente).
    pub bonus_puissance: i32,
    /// Remplace la précision de l'arme.
    pub precision: Option<i32>,
    pub circonstances: Vec<Circonstance>,
    pub modificateur: i32,
    pub malus_volontaire: u8,
    pub malus_sans_talent: Option<u8>,
    pub armure_attaquant: Option<String>,
    pub defense: ParamsDefense,
    /// Le défenseur ne voit pas venir l'attaque ou ne peut pas répondre.
    pub defense_impossible: bool,
    pub armure_defenseur: Option<String>,
    /// Protection supplémentaire (bouclier, pouvoir...), en points.
    pub protection_supplementaire: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetailJet {
    pub description: String,
    pub jet: ResultatJet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetailDommages {
    /// Unités finales du conflit (attaque, moins la défense si elle a réussi).
    pub unites: u8,
    pub puissance: i32,
    pub bonus_force: i32,
    pub protection: i32,
    /// Dommages infligés, jamais négatifs.
    pub total: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatAttaque {
    pub type_attaque: TypeAttaque,
    pub attaque: DetailJet,
    pub defense: Option<DetailJet>,
    /// Pourquoi aucune défense n'a été lancée, le cas échéant.
    pub defense_absente: Option<String>,
    pub issue: Issue,
    /// Dommages si l'attaque aboutit. En cas d'unités égales, ce sont ceux de
    /// l'hypothèse « l'attaquant l'emporte » et `arbitrage_mj` est vrai.
    pub dommages: Option<DetailDommages>,
    pub arbitrage_mj: bool,
    /// Notes du livre sur l'arme et l'armure, à appliquer par le MJ.
    pub notes: Vec<String>,
}

impl ResultatAttaque {
    /// Dommages à appliquer tout de suite (aucun si le MJ doit arbitrer).
    pub fn dommages_a_appliquer(&self) -> u32 {
        if self.arbitrage_mj {
            0
        } else {
            self.dommages.as_ref().map_or(0, |d| d.total)
        }
    }
}

fn arme_du_catalogue(nom: &str, talent: &str) -> Result<&'static Arme, String> {
    let categorie = if est_talent_de_contact(talent) { CategorieArme::Contact } else { CategorieArme::Distance };
    catalogue::arme_par_nom(nom, Some(categorie))
        .first()
        .copied()
        .or_else(|| catalogue::arme_par_nom(nom, None).first().copied())
        .ok_or_else(|| format!("arme « {nom} » inconnue du catalogue (fournir `puissance` et `precision`)"))
}

fn armure(nom: &Option<String>) -> Result<Option<&'static Protection>, String> {
    nom.as_deref()
        .map(|n| catalogue::protection_par_nom(n).ok_or_else(|| format!("protection « {n} » inconnue du catalogue")))
        .transpose()
}

/// Ajustement d'armure : il ne pénalise que les talents d'Agilité.
fn ajustement_armure(armure: Option<&Protection>, carac: Carac) -> i32 {
    match (armure, carac) {
        (Some(p), Carac::AG) => p.ajustement,
        _ => 0,
    }
}

pub fn resoudre_attaque(
    att: &Personnage,
    def: &Personnage,
    p: &ParamsAttaque,
    source: &mut impl SourceDes,
) -> Result<ResultatAttaque, String> {
    let mut notes = Vec::new();

    // Arme, puissance, précision, type d'attaque.
    let arme = p.arme.as_deref().map(|n| arme_du_catalogue(n, &p.talent)).transpose()?;
    let type_attaque = match arme.map(|a| a.categorie) {
        Some(CategorieArme::Contact) => TypeAttaque::Contact,
        Some(CategorieArme::Distance) => TypeAttaque::Distance,
        None if est_talent_de_contact(&p.talent) => TypeAttaque::Contact,
        None => TypeAttaque::Distance,
    };
    if let Some(a) = arme {
        if catalogue::normaliser(&a.talent) != catalogue::normaliser(&p.talent) {
            notes.push(format!("{} s'utilise normalement avec le talent {}, pas {}", a.nom, a.talent, p.talent));
        }
        notes.extend(a.notes.iter().map(|n| format!("{} ({}) : {}", a.nom, n.id, n.texte)));
    }
    let puissance_base = match (p.puissance, arme) {
        (Some(v), _) => v,
        (None, Some(a)) => match (a.puissance, a.puissance_rafale, a.puissance_speciale) {
            (Some(v), _, _) => v,
            (None, Some(rafale), false) => {
                notes.push(format!("{} : tir en rafale uniquement", a.nom));
                rafale
            }
            _ => return Err(format!("{} a une puissance spéciale : fournir `puissance` après avoir lu sa note", a.nom)),
        },
        (None, None) => return Err("indiquer une `arme` du catalogue ou une `puissance`".into()),
    };
    let puissance = puissance_base + p.bonus_puissance;
    let precision = p.precision.or(arme.map(|a| a.precision)).unwrap_or(0);

    // Jet d'attaque.
    let carac_att = p
        .caracteristique
        .or_else(|| carac_talent_combat(&p.talent))
        .ok_or_else(|| format!("caractéristique de « {} » inconnue : la préciser", p.talent))?;
    let armure_att = armure(&p.armure_attaquant)?;
    let colonnes_circonstances: i32 = p.circonstances.iter().map(|c| c.colonnes(type_attaque)).sum();
    let modificateur_att =
        colonnes_circonstances + precision + ajustement_armure(armure_att, carac_att) + p.modificateur;
    let demande_att = DemandeTest {
        carac: carac_att,
        talent: Some(p.talent.clone()),
        pouvoir: None,
        difficulte: 1,
        modificateur: modificateur_att,
        malus_volontaire: p.malus_volontaire,
        malus_sans_talent: p.malus_sans_talent,
    };
    let prepare_att = att.preparer_test(&demande_att)?;
    let jet_att = evaluer(&prepare_att.action, D666::tirer(source));
    let attaque = DetailJet { description: prepare_att.description, jet: jet_att };

    // Défense.
    let raison = if p.defense_impossible {
        Some("la cible ne voit pas venir l'attaque ou ne peut pas répondre".to_string())
    } else if p.circonstances.iter().any(|c| c.interdit_la_defense()) {
        Some("cible surprise ou attaquée de dos : pas d'esquive".to_string())
    } else if def.sante().etat != EtatSante::Valide {
        Some(format!("{} est hors d'état de se défendre", def.nom))
    } else {
        None
    };
    let mut defense = None;
    if raison.is_none() {
        let talent_def = p.defense.talent.clone().unwrap_or_else(|| "Esquive".to_string());
        if !defense_autorisee(&p.talent, &talent_def) {
            return Err(format!("{talent_def} ne permet pas de se défendre contre une attaque de {}", p.talent));
        }
        let carac_def = p
            .defense
            .caracteristique
            .or_else(|| carac_talent_combat(&talent_def))
            .ok_or_else(|| format!("caractéristique de « {talent_def} » inconnue : la préciser"))?;
        let arme_parade = p.defense.arme.as_deref().map(|n| arme_du_catalogue(n, &talent_def)).transpose()?;
        let armure_def = armure(&p.armure_defenseur)?;
        let modificateur_def = p.defense.modificateur
            + arme_parade.map_or(0, |a| a.precision)
            + ajustement_armure(armure_def, carac_def);
        let puissance_parade = arme_parade.and_then(|a| a.puissance).unwrap_or(0);
        let prepare_def = def.preparer_test(&DemandeTest {
            carac: carac_def,
            talent: Some(talent_def),
            pouvoir: None,
            difficulte: 1,
            modificateur: modificateur_def,
            malus_volontaire: 0,
            malus_sans_talent: p.defense.malus_sans_talent,
        })?;
        let mut jet_def = evaluer(&prepare_def.action, D666::tirer(source));
        // La puissance de l'arme de parade s'ajoute aux unités de la défense.
        jet_def.unites = jet_def.unites.saturating_add_signed(puissance_parade as i8);
        defense = Some(DetailJet { description: prepare_def.description, jet: jet_def });
    }

    let issue = match &defense {
        Some(d) => resoudre_conflit(&attaque.jet, &d.jet),
        None if attaque.jet.reussi => Issue::DefenseSansEffet { unites: attaque.jet.unites },
        None => Issue::RienNeSePasse,
    };

    // Dommages.
    let unites_finales = match issue {
        Issue::RienNeSePasse => None,
        Issue::DefenseSansEffet { unites } => Some(unites),
        Issue::ConflitReel { vainqueur, unites_finales } => match vainqueur {
            crate::conflit::Vainqueur::Defenseur => None,
            _ => Some(unites_finales),
        },
    };
    let armure_def = armure(&p.armure_defenseur)?;
    let protection = armure_def
        .map_or(0, |a| protection_contre(a, &p.talent))
        .max(protection_naturelle(def))
        + p.protection_supplementaire;
    if let Some(a) = armure_def {
        notes.extend(a.notes.iter().map(|n| format!("{} ({}) : {}", a.nom, n.id, n.texte)));
    }
    let bonus = bonus_force(att.caracteristiques.fo, type_attaque);
    let dommages = unites_finales.map(|unites| DetailDommages {
        unites,
        puissance,
        bonus_force: bonus,
        protection,
        total: (i32::from(unites) + puissance + bonus - protection).max(0) as u32,
    });
    let arbitrage_mj = issue.action_reussie().is_none();

    Ok(ResultatAttaque { type_attaque, attaque, defense, defense_absente: raison, issue, dommages, arbitrage_mj, notes })
}

// ---- Initiative ------------------------------------------------------------------

/// Un rang d'initiative : les personnages d'un même rang agissent simultanément.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Rang {
    pub valeur: u8,
    pub personnages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Initiative {
    /// Phase 1 : pouvoirs psychiques, par Volonté décroissante (égalités départagées au D66).
    pub psychique: Vec<Rang>,
    /// Phase 2 : actions physiques, par Agilité décroissante (mêmes valeurs : simultané).
    pub physique: Vec<Rang>,
    /// Personnages ne pouvant pas agir (inconscients, dans le coma, morts).
    pub hors_combat: Vec<String>,
    /// Tirages de D66 qui ont départagé des égalités en phase psychique : (nom, D66).
    pub departages: Vec<(String, u8)>,
}

fn grouper(mut valeurs: Vec<(String, u8)>) -> Vec<Rang> {
    valeurs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let mut rangs: Vec<Rang> = Vec::new();
    for (nom, valeur) in valeurs {
        match rangs.last_mut() {
            Some(r) if r.valeur == valeur => r.personnages.push(nom),
            _ => rangs.push(Rang { valeur, personnages: vec![nom] }),
        }
    }
    rangs
}

pub fn initiative(personnages: &[Personnage], source: &mut impl SourceDes) -> Initiative {
    let (actifs, hors): (Vec<&Personnage>, Vec<&Personnage>) =
        personnages.iter().partition(|p| p.sante().etat == EtatSante::Valide);
    let physique = grouper(actifs.iter().map(|p| (p.nom.clone(), p.caracteristiques.ag)).collect());

    // Phase psychique : seuls ceux qui ont des pouvoirs peuvent attaquer ; égalités départagées au D66.
    let psychiques: Vec<&&Personnage> = actifs.iter().filter(|p| p.pp_max() > 0).collect();
    let mut departages = Vec::new();
    let mut psychique = Vec::new();
    for rang in grouper(psychiques.iter().map(|p| (p.nom.clone(), p.caracteristiques.vo)).collect()) {
        if rang.personnages.len() == 1 {
            psychique.push(rang);
            continue;
        }
        let mut tirages: Vec<(String, u8)> = rang
            .personnages
            .into_iter()
            .map(|nom| {
                let d66 = crate::des::d66(source);
                departages.push((nom.clone(), d66));
                (nom, d66)
            })
            .collect();
        tirages.sort_by(|a, b| b.1.cmp(&a.1));
        let mut sous_rangs: Vec<Rang> = Vec::new();
        for (nom, d66) in tirages {
            match sous_rangs.last_mut() {
                Some(r) if r.valeur == d66 => r.personnages.push(nom),
                _ => sous_rangs.push(Rang { valeur: rang.valeur, personnages: vec![nom] }),
            }
        }
        psychique.extend(sous_rangs);
    }

    Initiative {
        psychique,
        physique,
        hors_combat: hors.iter().map(|p| p.nom.clone()).collect(),
        departages,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::des::SequenceDes;
    use crate::personnage::{Caracteristiques, Espece, PouvoirPossede, Role, TalentPossede};

    fn perso(nom: &str, espece: Espece, c: (u8, u8, u8, u8, u8, u8), talents: &[(&str, u8)]) -> Personnage {
        let (fo, pr, vo, pe, ag, ap) = c;
        let mut p = Personnage {
            id: String::new(),
            nom: nom.into(),
            espece,
            role: Role::Pnj,
            caracteristiques: Caracteristiques { fo, pr, vo, pe, ag, ap },
            talents: talents.iter().map(|(n, l)| TalentPossede { nom: n.to_string(), niveau: *l }).collect(),
            pouvoirs: vec![],
            patron: None,
            pp: None,
            dommages: 0,
            equipement: vec![],
            notes: String::new(),
        };
        p.finaliser();
        p
    }

    /// Jean : Ange FO4 AG5, Arme de contact +2 → 66 en attaque.
    fn jean() -> Personnage {
        perso("Jean", Espece::Ange, (4, 2, 4, 3, 5, 2), &[("Arme de contact", 2), ("Esquive", 1)])
    }

    /// Victime : humain FO2 AG2, Esquive +0 → colonne 2, seuil 21.
    fn victime() -> Personnage {
        perso("Victime", Espece::Humain, (2, 2, 2, 2, 2, 2), &[("Esquive", 0)])
    }

    fn epee() -> ParamsAttaque {
        ParamsAttaque {
            talent: "Arme de contact".into(),
            arme: Some("Epée longue".into()),
            bonus_puissance: 1,
            ..Default::default()
        }
    }

    fn lancer(att: &Personnage, def: &Personnage, p: &ParamsAttaque, des: &[u8]) -> ResultatAttaque {
        resoudre_attaque(att, def, p, &mut SequenceDes::new(des.iter().copied())).unwrap()
    }

    // ---- tables ----

    #[test]
    fn circonstances_conformes_au_tableau() {
        use Circonstance::*;
        use TypeAttaque::{Contact as C, Distance as D};
        let attendu = [
            (CibleSurprise, 1, 0),
            (CibleDeDos, 1, 0),
            (CibleMobile, 0, -1),
            (AttaquantMobile, -1, -1),
            (DansLaPortee, 0, 1),
            (PorteeDouble, 0, 0),
            (PorteeLointaine, 0, -1),
            (VisePendantUnTour, 1, 1),
            (VisePendantDeuxTours, 0, 2),
            (ViseTroisToursEtPlus, 0, 3),
            (Nuit, -1, -2),
            (Voiture, 1, 2),
            (AttaquantPlusBas, -1, 0),
            (AttaquantPlusHaut, 1, 0),
        ];
        for (c, contact, distance) in attendu {
            assert_eq!((c.colonnes(C), c.colonnes(D)), (contact, distance), "{c:?}");
        }
        assert!(CibleSurprise.interdit_la_defense() && CibleDeDos.interdit_la_defense());
        assert!(!Nuit.interdit_la_defense());
    }

    #[test]
    fn caracteristiques_des_talents_de_combat() {
        assert_eq!(carac_talent_combat("Arme de contact"), Some(Carac::AG));
        assert_eq!(carac_talent_combat("Arme de contact lourde"), Some(Carac::FO));
        assert_eq!(carac_talent_combat("Corps à corps"), Some(Carac::AG));
        assert_eq!(carac_talent_combat("Arme d'épaule"), Some(Carac::PE));
        assert_eq!(carac_talent_combat("Lancer"), Some(Carac::PR));
        assert_eq!(carac_talent_combat("esquive"), Some(Carac::AG));
        assert_eq!(carac_talent_combat("Spécialisation"), None);
    }

    #[test]
    fn defenses_permises_selon_le_tableau() {
        assert!(defense_autorisee("Corps à corps", "Arme de contact lourde"));
        assert!(defense_autorisee("Arme de contact", "Arme de contact"));
        assert!(!defense_autorisee("Arme de contact", "Corps à corps"));
        assert!(defense_autorisee("Arme de contact lourde", "Esquive"));
        assert!(!defense_autorisee("Arme de contact lourde", "Arme de contact"));
        assert!(defense_autorisee("Arme de poing", "Esquive"));
        assert!(!defense_autorisee("Arme de poing", "Arme de contact"));
        assert!(!defense_autorisee("Lancer", "Corps à corps"));
    }

    #[test]
    fn bonus_de_force_au_contact_seulement() {
        assert_eq!(bonus_force(3, TypeAttaque::Contact), 0);
        assert_eq!(bonus_force(4, TypeAttaque::Contact), 1);
        assert_eq!(bonus_force(5, TypeAttaque::Contact), 2);
        assert_eq!(bonus_force(6, TypeAttaque::Contact), 3);
        assert_eq!(bonus_force(6, TypeAttaque::Distance), 0);
    }

    // ---- attaques de bout en bout ----

    #[test]
    fn attaque_gagnante_dommages_calcules_comme_dans_le_livre() {
        // Jean (66) : 435 → touche, unités 5. Victime esquive (≤21) : 113 → réussit, unités 3.
        // Conflit : écart 2. Dommages = 2 + puissance 3 (épée +2, intelligente +1) + Force (4-3) = 6.
        let r = lancer(&jean(), &victime(), &epee(), &[4, 3, 5, 1, 1, 3]);
        assert_eq!(r.type_attaque, TypeAttaque::Contact);
        assert!(r.attaque.jet.reussi && r.defense.as_ref().unwrap().jet.reussi);
        let d = r.dommages.clone().unwrap();
        assert_eq!((d.unites, d.puissance, d.bonus_force, d.protection, d.total), (2, 3, 1, 0, 6));
        assert_eq!(r.dommages_a_appliquer(), 6);
        assert!(!r.arbitrage_mj);
    }

    #[test]
    fn defense_ratee_les_unites_de_l_attaque_comptent_en_entier() {
        // Défense 666 : ratée. Dommages = 5 + 3 + 1 = 9.
        let r = lancer(&jean(), &victime(), &epee(), &[4, 3, 5, 6, 6, 6]);
        assert_eq!(r.issue, Issue::DefenseSansEffet { unites: 5 });
        assert_eq!(r.dommages.unwrap().total, 9);
    }

    #[test]
    fn defense_victorieuse_pas_de_dommages() {
        // Attaque unités 1, défense réussie unités 4 : le défenseur l'emporte.
        let r = lancer(&jean(), &victime(), &epee(), &[4, 3, 1, 1, 1, 4]);
        assert_eq!(r.dommages, None);
        assert_eq!(r.dommages_a_appliquer(), 0);
    }

    #[test]
    fn attaque_ratee_rien_ne_se_passe() {
        let mut j = jean();
        j.talents[0].niveau = 0;
        j.caracteristiques.ag = 1; // colonne 1 ligne 0 → 13
        let r = lancer(&j, &victime(), &epee(), &[6, 6, 2, 1, 1, 1]);
        assert_eq!(r.issue, Issue::RienNeSePasse);
        assert_eq!(r.dommages, None);
    }

    #[test]
    fn egalite_des_unites_arbitrage_du_mj_sans_dommages_appliques() {
        let r = lancer(&jean(), &victime(), &epee(), &[4, 3, 3, 1, 1, 3]);
        assert!(r.arbitrage_mj);
        assert_eq!(r.dommages.as_ref().unwrap().unites, 0);
        assert_eq!(r.dommages_a_appliquer(), 0);
    }

    #[test]
    fn cible_surprise_pas_de_defense_et_bonus_de_colonne() {
        let mut p = epee();
        p.circonstances = vec![Circonstance::CibleSurprise];
        // Un seul D666 est lancé : celui de l'attaque.
        let r = lancer(&jean(), &victime(), &p, &[4, 3, 5]);
        assert!(r.defense.is_none());
        assert!(r.defense_absente.as_ref().unwrap().contains("pas d'esquive"));
        assert_eq!(r.dommages.unwrap().total, 5 + 3 + 1);
    }

    #[test]
    fn defenseur_inconscient_ne_se_defend_pas() {
        let mut v = victime();
        v.dommages = 2; // Force 2 → 0 : évanoui
        let r = lancer(&jean(), &v, &epee(), &[4, 3, 5]);
        assert!(r.defense_absente.unwrap().contains("hors d'état"));
        assert_eq!(r.dommages.unwrap().total, 9);
    }

    #[test]
    fn protection_de_l_armure_et_ajustement_d_agilite() {
        // Cotte de mailles : protection 3, -1 colonne sur l'Esquive (Agilité).
        let mut p = epee();
        p.armure_defenseur = Some("Cotte de maille".into());
        // Esquive : colonne 2 - 1 = 1 → seuil 13. Dé 113 : 11 ≤ 13 réussit, unités 3 ; attaque 5 → écart 2.
        let r = lancer(&jean(), &victime(), &p, &[4, 3, 5, 1, 1, 3]);
        assert_eq!(r.defense.as_ref().unwrap().jet.seuil, crate::table_um::Seuil::Valeur(13));
        let d = r.dommages.unwrap();
        assert_eq!((d.protection, d.total), (3, 2 + 3 + 1 - 3));
    }

    #[test]
    fn dommages_jamais_negatifs() {
        let mut p = epee();
        p.armure_defenseur = Some("Acier".into()); // protection 6
        p.bonus_puissance = 0;
        let r = lancer(&jean(), &victime(), &p, &[4, 3, 2, 6, 6, 6]);
        assert_eq!(r.dommages.unwrap().total, 0, "2 + 2 + 1 - 6 < 0");
    }

    #[test]
    fn armure_corporelle_naturelle_pouvoir_211() {
        let mut v = victime();
        v.pouvoirs = vec![PouvoirPossede { code: Some(211), nom: String::new(), niveau: 1, limitation: false }];
        v.espece = Espece::Ange; // pouvoirs de la table des Anges
        v.finaliser();
        assert_eq!(protection_naturelle(&v), 3);
        let r = lancer(&jean(), &v, &epee(), &[4, 3, 5, 6, 6, 6]);
        assert_eq!(r.dommages.unwrap().protection, 3);
        // On prend la meilleure des deux protections, elles ne s'additionnent pas.
        let mut p = epee();
        p.armure_defenseur = Some("Cuir".into());
        let r = lancer(&jean(), &v, &p, &[4, 3, 5, 6, 6, 6]);
        assert_eq!(r.dommages.unwrap().protection, 3);
    }

    #[test]
    fn kevlar_protege_de_4_contre_les_armes_a_feu_et_de_2_sinon() {
        let kevlar = catalogue::protection_par_nom("Kevlar").unwrap();
        assert_eq!(protection_contre(kevlar, "Arme de poing"), 4);
        assert_eq!(protection_contre(kevlar, "Arme d'épaule"), 4);
        assert_eq!(protection_contre(kevlar, "Arme de contact"), 2);
        assert_eq!(protection_contre(catalogue::protection_par_nom("Cuir").unwrap(), "Arme de poing"), 1);
    }

    #[test]
    fn parade_avec_une_arme_ajoute_sa_puissance_et_sa_precision() {
        // Victime avec Arme de contact +2 et une hachette (+1 puissance, -1 précision).
        let mut v = perso("Garde", Espece::Humain, (2, 2, 2, 2, 3, 2), &[("Arme de contact", 2)]);
        v.finaliser();
        let mut p = epee();
        p.defense = ParamsDefense {
            talent: Some("Arme de contact".into()),
            arme: Some("Hachette".into()),
            ..Default::default()
        };
        // AG 3 → colonne 5, ligne 2, précision -1 → colonne 4 → 54. Dé 223 : 22 ≤ 54, unités 3 + 1 = 4.
        let r = lancer(&jean(), &v, &p, &[4, 3, 5, 2, 2, 3]);
        let d = r.defense.unwrap();
        assert_eq!(d.jet.seuil, crate::table_um::Seuil::Valeur(54));
        assert_eq!(d.jet.unites, 4);
        assert_eq!(r.dommages.unwrap().unites, 1, "écart 5 - 4");
    }

    #[test]
    fn arme_a_distance_perception_circonstances_et_pas_de_bonus_de_force() {
        let tireur = perso("Tireur", Espece::Demon, (5, 2, 3, 4, 2, 2), &[("Arme d'épaule", 1)]);
        let mut cible = victime();
        cible.talents = vec![TalentPossede { nom: "Esquive".into(), niveau: 0 }];
        let p = ParamsAttaque {
            talent: "Arme d'épaule".into(),
            arme: Some("Carabine".into()),
            circonstances: vec![Circonstance::DansLaPortee, Circonstance::VisePendantUnTour, Circonstance::Nuit],
            ..Default::default()
        };
        // PE 4 → colonne 8, ligne +1 ; carabine précision +1 ; portée +1, visée +1, nuit -2 → +1 → colonne 9 → 55.
        let r = lancer(&tireur, &cible, &p, &[5, 5, 1, 6, 6, 6]);
        assert_eq!(r.type_attaque, TypeAttaque::Distance);
        assert_eq!(r.attaque.jet.seuil, crate::table_um::Seuil::Valeur(55));
        let d = r.dommages.unwrap();
        assert_eq!((d.bonus_force, d.puissance), (0, 1), "Force 5 ne compte pas à distance");
        assert_eq!(d.total, 1 + 1);
    }

    #[test]
    fn malus_de_blessure_de_l_attaquant_applique() {
        let mut gardien = perso("Gardien", Espece::Soldat, (3, 2, 2, 2, 5, 2), &[("Arme de contact", 2)]);
        gardien.dommages = 2;
        let r = lancer(&gardien, &victime(), &epee(), &[1, 1, 1, 6, 6, 6]);
        assert!(r.attaque.description.contains("blessures : -2"));
        assert_eq!(r.attaque.jet.colonne, 13 - 2);
    }

    #[test]
    fn armes_speciales_et_erreurs() {
        let j = jean();
        let v = victime();
        let mut des = SequenceDes::new([]);
        // Ni arme ni puissance.
        let p = ParamsAttaque { talent: "Corps à corps".into(), ..Default::default() };
        assert!(resoudre_attaque(&j, &v, &p, &mut des).unwrap_err().contains("arme"));
        // Puissance spéciale : coup de boule.
        let p = ParamsAttaque { talent: "Corps à corps".into(), arme: Some("Coup de boule".into()), ..Default::default() };
        assert!(resoudre_attaque(&j, &v, &p, &mut des).unwrap_err().contains("spéciale"));
        // Arme inconnue, défense interdite, protection inconnue.
        let p = ParamsAttaque { arme: Some("Sabre laser".into()), ..epee() };
        assert!(resoudre_attaque(&j, &v, &p, &mut des).unwrap_err().contains("inconnue"));
        let mut p = epee();
        p.defense.talent = Some("Corps à corps".into());
        assert!(resoudre_attaque(&j, &v, &p, &mut SequenceDes::new([4, 3, 5])).unwrap_err().contains("ne permet pas"));
        let mut p = epee();
        p.armure_defenseur = Some("Carton".into());
        assert!(resoudre_attaque(&j, &v, &p, &mut SequenceDes::new([4, 3, 5])).unwrap_err().contains("Carton"));
    }

    #[test]
    fn puissance_fournie_pour_une_arme_speciale() {
        let p = ParamsAttaque {
            talent: "Corps à corps".into(),
            arme: Some("Coup de boule".into()),
            puissance: Some(2),
            ..Default::default()
        };
        let mut j = jean();
        j.talents.push(TalentPossede { nom: "Corps à corps".into(), niveau: 1 });
        let r = lancer(&j, &victime(), &p, &[4, 3, 5, 6, 6, 6]);
        assert_eq!(r.dommages.unwrap().puissance, 2);
        assert!(r.notes.iter().any(|n| n.contains("sonné")), "la note du livre est transmise : {:?}", r.notes);
    }

    #[test]
    fn un_attaquant_inconscient_ne_peut_pas_attaquer() {
        let mut j = jean();
        j.dommages = 18; // Ange Force 4 : -2 → assommé
        let err = resoudre_attaque(&j, &victime(), &epee(), &mut SequenceDes::new([])).unwrap_err();
        assert!(err.contains("inconscient"), "{err}");
    }

    // ---- initiative ----

    #[test]
    fn initiative_physique_par_agilite_et_egalites_simultanees() {
        let a = perso("A", Espece::Humain, (2, 2, 2, 2, 3, 2), &[]);
        let b = perso("B", Espece::Humain, (2, 2, 2, 2, 5, 2), &[]);
        let c = perso("C", Espece::Humain, (2, 2, 2, 2, 3, 2), &[]);
        let i = initiative(&[a, b, c], &mut SequenceDes::new([]));
        assert_eq!(i.physique.len(), 2);
        assert_eq!(i.physique[0].personnages, ["B"]);
        assert_eq!((i.physique[1].valeur, i.physique[1].personnages.clone()), (3, vec!["A".to_string(), "C".to_string()]));
        assert!(i.psychique.is_empty(), "sans pouvoirs, pas de phase psychique");
    }

    #[test]
    fn initiative_psychique_par_volonte_avec_depart_au_d66() {
        let mut j = jean();
        j.pouvoirs = vec![PouvoirPossede { code: Some(166), nom: String::new(), niveau: 0, limitation: false }];
        j.finaliser();
        let mut k = perso("Kora", Espece::Ange, (3, 3, 4, 3, 3, 3), &[]);
        k.pouvoirs = j.pouvoirs.clone();
        k.finaliser();
        let mut m = perso("Max", Espece::Ange, (3, 3, 6, 3, 3, 3), &[]);
        m.pouvoirs = j.pouvoirs.clone();
        m.finaliser();
        // Max (VO 6) d'abord ; Jean et Kora (VO 4) égalité : D66 → Jean 23, Kora 51 → Kora avant.
        let i = initiative(&[j, k, m], &mut SequenceDes::new([2, 3, 5, 1]));
        let ordre: Vec<&str> = i.psychique.iter().flat_map(|r| r.personnages.iter().map(String::as_str)).collect();
        assert_eq!(ordre, ["Max", "Kora", "Jean"]);
        assert_eq!(i.departages, vec![("Jean".to_string(), 23), ("Kora".to_string(), 51)]);
    }

    #[test]
    fn initiative_ecarte_les_personnages_hors_combat() {
        let a = perso("Valide", Espece::Humain, (2, 2, 2, 2, 3, 2), &[]);
        let mut b = perso("Mort", Espece::Humain, (2, 2, 2, 2, 5, 2), &[]);
        b.dommages = 9;
        let i = initiative(&[a, b], &mut SequenceDes::new([]));
        assert_eq!(i.hors_combat, ["Mort"]);
        assert_eq!(i.physique.len(), 1);
    }
}
