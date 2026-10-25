//! Fiche de personnage et règles de création (`livres/commun-06-…`, `ange-03-…`, `demon-03-…`).
//!
//! Les budgets de création sont ceux des tableaux « Résumé » des deux livres.
//! Les exceptions (« il existe bien sûr des exceptions aux valeurs indiquées »)
//! sont du ressort du MJ : [`controler`] distingue les dépassements de budget
//! (Gravite::Budget, contournables par une exception motivée) des fiches
//! incohérentes (Gravite::Structure, jamais acceptées).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::catalogue::{self, Livre, Pouvoir};
use crate::table_um::{Action, Caracteristique, NiveauTalent};

// ---- Caractéristiques -------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Carac {
    FO,
    PR,
    VO,
    PE,
    AG,
    AP,
}

impl Carac {
    pub const TOUTES: [Carac; 6] = [Carac::FO, Carac::PR, Carac::VO, Carac::PE, Carac::AG, Carac::AP];

    /// Accepte l'abréviation (« FO ») ou le nom (« Force »), sans casse ni accent.
    pub fn analyser(texte: &str) -> Option<Self> {
        match catalogue::normaliser(texte).as_str() {
            "fo" | "force" => Some(Self::FO),
            "pr" | "precision" => Some(Self::PR),
            "vo" | "volonte" => Some(Self::VO),
            "pe" | "perception" => Some(Self::PE),
            "ag" | "agilite" => Some(Self::AG),
            "ap" | "apparence" => Some(Self::AP),
            _ => None,
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Self::FO => "Force",
            Self::PR => "Précision",
            Self::VO => "Volonté",
            Self::PE => "Perception",
            Self::AG => "Agilité",
            Self::AP => "Apparence",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Caracteristiques {
    pub fo: u8,
    pub pr: u8,
    pub vo: u8,
    pub pe: u8,
    pub ag: u8,
    pub ap: u8,
}

impl Caracteristiques {
    pub fn get(&self, c: Carac) -> u8 {
        match c {
            Carac::FO => self.fo,
            Carac::PR => self.pr,
            Carac::VO => self.vo,
            Carac::PE => self.pe,
            Carac::AG => self.ag,
            Carac::AP => self.ap,
        }
    }

    pub fn total(&self) -> u32 {
        Carac::TOUTES.iter().map(|&c| u32::from(self.get(c))).sum()
    }
}

// ---- Espèces et budgets -----------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Espece {
    Humain,
    Serviteur,
    Soldat,
    Ange,
    MortVivant,
    Familier,
    Demon,
}

/// Budget de création d'une espèce (tableaux « Résumé »).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Budget {
    pub points_caracteristiques: u32,
    pub max_caracteristique: u8,
    pub points_talents: u32,
    pub max_niveau_talent: u8,
    pub nb_pouvoirs: u32,
    pub max_niveau_pouvoir: u8,
    /// Récupération : 1 PP toutes les n heures (hors repos complet : 1 PP / 24 h).
    pub heures_par_pp: Option<u32>,
}

impl Espece {
    pub fn budget(self) -> Budget {
        let b = |pc, mc, pt, np, mp, h| Budget {
            points_caracteristiques: pc,
            max_caracteristique: mc,
            points_talents: pt,
            max_niveau_talent: 2,
            nb_pouvoirs: np,
            max_niveau_pouvoir: mp,
            heures_par_pp: h,
        };
        match self {
            Self::Humain => b(10, 3, 6, 0, 0, None),
            Self::Serviteur => b(12, 3, 10, 1, 0, Some(6)),
            Self::Soldat => b(14, 3, 12, 2, 1, Some(3)),
            Self::Ange => b(20, 5, 8, 4, 2, Some(24)),
            Self::MortVivant => b(12, 4, 4, 2, 0, Some(12)),
            Self::Familier => b(14, 4, 4, 3, 1, Some(6)),
            Self::Demon => b(18, 5, 10, 4, 2, Some(24)),
        }
    }

    /// Table de pouvoirs applicable ; `None` pour les humains (aucun pouvoir).
    pub fn livre_pouvoirs(self) -> Option<Livre> {
        match self {
            Self::Humain => None,
            Self::Serviteur | Self::Soldat | Self::Ange => Some(Livre::Ange),
            Self::MortVivant | Self::Familier | Self::Demon => Some(Livre::Demon),
        }
    }

    /// Les personnages-joueurs peuvent prendre une limitation contre un pouvoir de plus.
    pub fn accepte_limitation(self) -> bool {
        matches!(self, Self::Ange | Self::Demon)
    }

    fn livre_talents(self) -> Livre {
        self.livre_pouvoirs().unwrap_or(Livre::Ange)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Joueur,
    /// PNJ qui accompagne le joueur pour équilibrer la difficulté.
    Partenaire,
    #[default]
    Pnj,
}

// ---- Fiche -------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TalentPossede {
    pub nom: String,
    /// 0 à +2 à la création (+3 réservé à l'évolution).
    pub niveau: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PouvoirPossede {
    /// Numéro de la table des pouvoirs (ex. 321). À défaut, `nom` identifie le pouvoir.
    #[serde(default)]
    pub code: Option<u16>,
    #[serde(default)]
    pub nom: String,
    #[serde(default)]
    pub niveau: u8,
    /// Renseigné par [`Personnage::finaliser`] : vrai pour les limitations (codes 6xx).
    #[serde(default)]
    pub limitation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Personnage {
    /// Identifiant stable, dérivé du nom par [`Personnage::finaliser`].
    #[serde(default)]
    pub id: String,
    pub nom: String,
    pub espece: Espece,
    #[serde(default)]
    pub role: Role,
    pub caracteristiques: Caracteristiques,
    #[serde(default)]
    pub talents: Vec<TalentPossede>,
    #[serde(default)]
    pub pouvoirs: Vec<PouvoirPossede>,
    /// Archange, Prince-Démon ou supérieur dont dépend le personnage.
    #[serde(default)]
    pub patron: Option<String>,
    /// PP courants ; absent à la création (= maximum).
    #[serde(default)]
    pub pp: Option<u32>,
    /// Points de dommages subis et non encore guéris (effets : voir blessures).
    #[serde(default)]
    pub dommages: u32,
    #[serde(default)]
    pub equipement: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

pub fn identifiant(nom: &str) -> String {
    catalogue::normaliser(nom).replace(' ', "-")
}

impl Personnage {
    /// Complète la fiche : identifiant, noms et indicateurs de limitation tirés du
    /// catalogue, PP courants. Les pouvoirs inconnus sont laissés tels quels
    /// (signalés ensuite par [`controler`]).
    pub fn finaliser(&mut self) {
        self.id = identifiant(&self.nom);
        if let Some(livre) = self.espece.livre_pouvoirs() {
            for p in &mut self.pouvoirs {
                if let Ok(trouve) = trouver_pouvoir(livre, p) {
                    p.code = trouve.code;
                    p.nom = trouve.nom.clone();
                    p.limitation = trouve.est_limitation();
                }
            }
        }
        if self.pp.is_none() {
            self.pp = Some(self.pp_max());
        }
    }

    /// Tirages sur la table des pouvoirs : un par niveau acquis (niveau + 1) et un par limitation.
    pub fn tirages(&self) -> u32 {
        self.pouvoirs.iter().map(|p| if p.limitation { 1 } else { u32::from(p.niveau) + 1 }).sum()
    }

    /// Volonté + un point par tirage sur la table des pouvoirs (limitations comprises).
    pub fn pp_max(&self) -> u32 {
        if self.pouvoirs.is_empty() {
            0
        } else {
            u32::from(self.caracteristiques.vo) + self.tirages()
        }
    }

    pub fn pp_courants(&self) -> u32 {
        self.pp.unwrap_or_else(|| self.pp_max())
    }

    pub fn depenser_pp(&mut self, n: u32) -> Result<u32, String> {
        let actuels = self.pp_courants();
        if n > actuels {
            return Err(format!("{} n'a que {actuels} PP, {n} demandés", self.nom));
        }
        self.pp = Some(actuels - n);
        Ok(actuels - n)
    }

    /// Récupère des PP sans dépasser le maximum ; renvoie les PP courants.
    pub fn recuperer_pp(&mut self, n: u32) -> u32 {
        let nouveau = (self.pp_courants() + n).min(self.pp_max());
        self.pp = Some(nouveau);
        nouveau
    }

    /// Ajoute (delta > 0) ou soigne (delta < 0) des dommages ; jamais en dessous de zéro.
    pub fn ajuster_dommages(&mut self, delta: i32) -> u32 {
        self.dommages = self.dommages.saturating_add_signed(delta);
        self.dommages
    }

    pub fn talent(&self, nom: &str) -> Option<&TalentPossede> {
        let cible = catalogue::normaliser(nom);
        self.talents.iter().find(|t| catalogue::normaliser(&t.nom) == cible)
    }

    pub fn pouvoir(&self, nom: &str) -> Option<&PouvoirPossede> {
        let cible = catalogue::normaliser(nom);
        self.pouvoirs.iter().find(|p| catalogue::normaliser(&p.nom) == cible)
    }

    /// Construit l'action à résoudre sur la table UM à partir de la fiche.
    pub fn preparer_test(&self, demande: &DemandeTest) -> Result<TestPrepare, String> {
        let carac = Caracteristique::new(self.caracteristiques.get(demande.carac))
            .map_err(|e| e.to_string())?;
        let (base, origine) = match (&demande.talent, &demande.pouvoir) {
            (Some(_), Some(_)) => return Err("indiquer un talent ou un pouvoir, pas les deux".into()),
            (Some(nom), None) => match self.talent(nom) {
                Some(t) => (
                    Action::new(carac, niveau(t.niveau)?),
                    format!("talent {} +{}", t.nom, t.niveau),
                ),
                None => self.sans_talent(carac, nom, demande.malus_sans_talent)?,
            },
            (None, Some(nom)) => match self.pouvoir(nom) {
                Some(p) => (
                    Action::new(carac, niveau(p.niveau)?),
                    format!("pouvoir {} +{}", p.nom, p.niveau),
                ),
                None => return Err(format!("{} ne possède pas le pouvoir « {nom} »", self.nom)),
            },
            (None, None) => (
                Action::new(carac, niveau(demande.difficulte)?),
                format!("caractéristique seule, difficulté +{}", demande.difficulte),
            ),
        };
        let action = base
            .avec_modificateur(demande.modificateur)
            .avec_malus_volontaire(demande.malus_volontaire);
        let description = format!(
            "{} : {} {} ({origine})",
            self.nom,
            demande.carac.nom(),
            self.caracteristiques.get(demande.carac)
        );
        Ok(TestPrepare { action, description })
    }

    fn sans_talent(
        &self,
        carac: Caracteristique,
        nom: &str,
        malus_impose: Option<u8>,
    ) -> Result<(Action, String), String> {
        let malus = match malus_impose {
            Some(m) => m,
            None => {
                let connu = catalogue::talent_par_nom(nom)
                    .and_then(|t| t.malus_pour(self.espece.livre_talents()));
                match connu {
                    Some(m) => u8::try_from(-m).map_err(|_| format!("malus incohérent pour « {nom} »"))?,
                    None => {
                        return Err(format!(
                            "talent « {nom} » absent de la fiche et sans malus connu : \
                             préciser malus_sans_talent"
                        ))
                    }
                }
            }
        };
        Ok((Action::sans_talent(carac, malus), format!("sans le talent {nom}, malus {malus} colonnes")))
    }
}

fn niveau(n: u8) -> Result<NiveauTalent, String> {
    NiveauTalent::new(n).map_err(|e| e.to_string())
}

/// Ce que l'on veut tester sur une fiche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandeTest {
    pub carac: Carac,
    pub talent: Option<String>,
    pub pouvoir: Option<String>,
    /// Sans talent ni pouvoir : 0 difficile, 1 moyen, 2 facile.
    pub difficulte: u8,
    pub modificateur: i32,
    pub malus_volontaire: u8,
    /// Remplace le malus du catalogue (ou le fournit s'il est variable).
    pub malus_sans_talent: Option<u8>,
}

impl DemandeTest {
    pub fn caracteristique(carac: Carac, difficulte: u8) -> Self {
        Self {
            carac,
            talent: None,
            pouvoir: None,
            difficulte,
            modificateur: 0,
            malus_volontaire: 0,
            malus_sans_talent: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPrepare {
    pub action: Action,
    pub description: String,
}

// ---- Contrôle de création ------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Gravite {
    /// Fiche incohérente : jamais acceptée.
    Structure,
    /// Dépassement des budgets de création : le MJ peut le valider comme exception.
    Budget,
    /// Information (points non dépensés, etc.).
    Avertissement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Probleme {
    pub gravite: Gravite,
    pub message: String,
}

fn probleme(gravite: Gravite, message: impl Into<String>) -> Probleme {
    Probleme { gravite, message: message.into() }
}

/// Coût d'un niveau de talent : 1 point pour 0, 2 pour +1, 4 pour +2.
pub fn cout_talent(niveau: u8) -> Option<u32> {
    match niveau {
        0 => Some(1),
        1 => Some(2),
        2 => Some(4),
        _ => None,
    }
}

fn trouver_pouvoir(livre: Livre, p: &PouvoirPossede) -> Result<&'static Pouvoir, String> {
    if let Some(code) = p.code {
        return catalogue::pouvoir_par_code(livre, code)
            .ok_or_else(|| format!("pouvoir {code} inconnu dans la table du livre {livre:?}"));
    }
    let trouves = catalogue::pouvoirs_par_nom(livre, &p.nom);
    match trouves.as_slice() {
        [] => Err(format!("pouvoir « {} » inconnu (utiliser le code à trois chiffres)", p.nom)),
        [un] => Ok(un),
        plusieurs => {
            let codes: Vec<String> =
                plusieurs.iter().filter_map(|p| p.code).map(|c| c.to_string()).collect();
            Err(format!("pouvoir « {} » ambigu : codes possibles {}", p.nom, codes.join(", ")))
        }
    }
}

/// Vérifie une fiche finalisée par rapport aux règles de création.
pub fn controler(p: &Personnage) -> Vec<Probleme> {
    use Gravite::*;
    let budget = p.espece.budget();
    let mut r = Vec::new();

    if p.nom.trim().is_empty() {
        r.push(probleme(Structure, "le nom est vide"));
    }

    // Caractéristiques.
    for c in Carac::TOUTES {
        let v = p.caracteristiques.get(c);
        if !(1..=6).contains(&v) {
            r.push(probleme(Structure, format!("{} hors bornes (1 à 6) : {v}", c.nom())));
        } else if v > budget.max_caracteristique {
            r.push(probleme(
                Budget,
                format!("{} = {v} dépasse le maximum {} de l'espèce", c.nom(), budget.max_caracteristique),
            ));
        }
    }
    let total = p.caracteristiques.total();
    match total.cmp(&budget.points_caracteristiques) {
        std::cmp::Ordering::Greater => r.push(probleme(
            Budget,
            format!("caractéristiques : {total} points répartis pour un budget de {}", budget.points_caracteristiques),
        )),
        std::cmp::Ordering::Less => r.push(probleme(
            Avertissement,
            format!("caractéristiques : {total} points répartis sur {}", budget.points_caracteristiques),
        )),
        std::cmp::Ordering::Equal => {}
    }

    // Talents.
    let mut depense = 0;
    let mut vus = std::collections::HashSet::new();
    for t in &p.talents {
        if !vus.insert(catalogue::normaliser(&t.nom)) {
            r.push(probleme(Structure, format!("talent « {} » en double", t.nom)));
        }
        match cout_talent(t.niveau) {
            Some(c) if t.niveau <= budget.max_niveau_talent => depense += c,
            Some(c) => {
                depense += c;
                r.push(probleme(Budget, format!("talent {} +{} : maximum +{}", t.nom, t.niveau, budget.max_niveau_talent)));
            }
            None => r.push(probleme(Structure, format!("talent {} : niveau {} invalide (0 à 3)", t.nom, t.niveau))),
        }
        if catalogue::talent_par_nom(&t.nom).is_none() {
            r.push(probleme(Avertissement, format!("talent « {} » absent du catalogue (création du MJ ?)", t.nom)));
        }
    }
    match depense.cmp(&budget.points_talents) {
        std::cmp::Ordering::Greater => r.push(probleme(
            Budget,
            format!("talents : {depense} points dépensés pour un budget de {}", budget.points_talents),
        )),
        std::cmp::Ordering::Less => r.push(probleme(
            Avertissement,
            format!("talents : {depense} points dépensés sur {}", budget.points_talents),
        )),
        std::cmp::Ordering::Equal => {}
    }

    // Pouvoirs.
    match p.espece.livre_pouvoirs() {
        None if !p.pouvoirs.is_empty() => {
            r.push(probleme(Budget, "les humains ne peuvent posséder de pouvoirs"));
        }
        None => {}
        Some(livre) => controler_pouvoirs(p, livre, &budget, &mut r),
    }

    if let Some(pp) = p.pp {
        if pp > p.pp_max() {
            r.push(probleme(Structure, format!("PP courants {pp} supérieurs au maximum {}", p.pp_max())));
        }
    }
    r
}

fn controler_pouvoirs(p: &Personnage, livre: Livre, budget: &Budget, r: &mut Vec<Probleme>) {
    use Gravite::*;
    let mut limitations = 0;
    let mut tirages = 0;
    for pv in &p.pouvoirs {
        let Ok(trouve) = trouver_pouvoir(livre, pv) else {
            r.push(probleme(Structure, trouver_pouvoir(livre, pv).unwrap_err()));
            continue;
        };
        if trouve.est_limitation() {
            limitations += 1;
            if pv.niveau != 0 {
                r.push(probleme(Structure, format!("limitation « {} » : pas de niveau", trouve.nom)));
            }
        } else {
            tirages += u32::from(pv.niveau) + 1;
            if pv.niveau > budget.max_niveau_pouvoir {
                r.push(probleme(
                    Budget,
                    format!("pouvoir {} niveau +{} : maximum +{}", trouve.nom, pv.niveau, budget.max_niveau_pouvoir),
                ));
            }
        }
    }
    if limitations > 1 || (limitations == 1 && !p.espece.accepte_limitation()) {
        r.push(probleme(
            Budget,
            "une seule limitation, réservée aux Anges et aux Démons (contre un pouvoir de plus)",
        ));
    }
    let autorises = budget.nb_pouvoirs + u32::from(limitations == 1 && p.espece.accepte_limitation());
    if tirages > autorises {
        r.push(probleme(
            Budget,
            format!("pouvoirs : {tirages} tirages (un par niveau acquis) pour {autorises} autorisés"),
        ));
    } else if tirages < autorises {
        r.push(probleme(Avertissement, format!("pouvoirs : {tirages} tirages sur {autorises} autorisés")));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carac(fo: u8, vo: u8, ag: u8, ap: u8, pr: u8, pe: u8) -> Caracteristiques {
        Caracteristiques { fo, pr, vo, pe, ag, ap }
    }

    fn talent(nom: &str, niveau: u8) -> TalentPossede {
        TalentPossede { nom: nom.into(), niveau }
    }

    fn pouvoir(code: u16, niveau: u8) -> PouvoirPossede {
        PouvoirPossede { code: Some(code), nom: String::new(), niveau, limitation: false }
    }

    fn fiche(espece: Espece, c: Caracteristiques) -> Personnage {
        Personnage {
            id: String::new(),
            nom: "Test".into(),
            espece,
            role: Role::Pnj,
            caracteristiques: c,
            talents: vec![],
            pouvoirs: vec![],
            patron: None,
            pp: None,
            dommages: 0,
            equipement: vec![],
            notes: String::new(),
        }
    }

    fn gravites(p: &Personnage, g: Gravite) -> Vec<String> {
        controler(p).into_iter().filter(|x| x.gravite == g).map(|x| x.message).collect()
    }

    // ---- Budgets (tableaux « Résumé ») ----

    #[test]
    fn budgets_conformes_aux_tableaux_resume() {
        let attendu = [
            (Espece::Humain, 10, 3, 6, 0, 0),
            (Espece::Serviteur, 12, 3, 10, 1, 0),
            (Espece::Soldat, 14, 3, 12, 2, 1),
            (Espece::Ange, 20, 5, 8, 4, 2),
            (Espece::MortVivant, 12, 4, 4, 2, 0),
            (Espece::Familier, 14, 4, 4, 3, 1),
            (Espece::Demon, 18, 5, 10, 4, 2),
        ];
        for (e, pc, mc, pt, np, mp) in attendu {
            let b = e.budget();
            assert_eq!(
                (b.points_caracteristiques, b.max_caracteristique, b.points_talents, b.nb_pouvoirs, b.max_niveau_pouvoir),
                (pc, mc, pt, np, mp),
                "{e:?}"
            );
            assert_eq!(b.max_niveau_talent, 2);
        }
    }

    #[test]
    fn recuperation_des_pp() {
        let h = |e: Espece| e.budget().heures_par_pp;
        assert_eq!(h(Espece::Humain), None);
        assert_eq!(h(Espece::Serviteur), Some(6));
        assert_eq!(h(Espece::Soldat), Some(3));
        assert_eq!(h(Espece::MortVivant), Some(12));
        assert_eq!(h(Espece::Familier), Some(6));
        assert_eq!(h(Espece::Ange), Some(24));
        assert_eq!(h(Espece::Demon), Some(24));
    }

    #[test]
    fn cout_des_talents() {
        assert_eq!([0, 1, 2].map(cout_talent), [Some(1), Some(2), Some(4)]);
        assert_eq!(cout_talent(3), None);
    }

    // ---- Humains du livre ----

    #[test]
    fn humains_du_livre_et_leurs_depassements_de_budget() {
        // (nom, caracs FO VO AG AP PR PE, talents)
        let exemples = [
            ("Normal", carac(2, 2, 1, 2, 1, 2), vec![("Esquive", 0), ("Baratin", 0), ("Mécanique", 1)]),
            ("Scientifique", carac(1, 3, 1, 1, 2, 2), vec![("Chimie", 2), ("Informatique", 0), ("Discussion", 1)]),
            ("Militaire", carac(2, 1, 2, 1, 2, 3), vec![("Arme de poing", 2), ("Arme d'épaule", 0), ("Tactique", 1)]),
            ("Policier", carac(2, 2, 1, 2, 2, 2), vec![("Arme de poing", 1), ("Corps à corps", 1), ("Voiture", 0), ("Esquive", 0)]),
            ("Skinhead", carac(3, 1, 2, 2, 1, 1), vec![("Corps à corps", 1), ("Esquive", 0), ("Arme de poing", 0)]),
            ("Chrétien", carac(1, 3, 1, 2, 2, 1), vec![("Discussion", 1), ("Baratin", 0), ("Esquive", 0)]),
            ("Loubard", carac(3, 1, 2, 2, 1, 1), vec![("Discrétion", 0), ("Manipulation", 0), ("Crochetage", 0), ("Corps à corps", 1)]),
            ("Truand", carac(2, 2, 1, 2, 2, 2), vec![("Discrétion", 0), ("Crochetage", 1), ("Corps à corps", 0), ("Arme de poing", 1)]),
        ];
        let mut hors_budget = vec![];
        for (nom, c, talents) in exemples {
            let mut p = fiche(Espece::Humain, c);
            p.nom = nom.into();
            p.talents = talents.iter().map(|(n, l)| talent(n, *l)).collect();
            p.finaliser();
            assert!(gravites(&p, Gravite::Structure).is_empty(), "{nom}");
            let budget = gravites(&p, Gravite::Budget);
            if !budget.is_empty() {
                hors_budget.push((nom, budget.len(), budget.join(" ; ")));
            }
        }
        // Erratum E-002 : ces exemples du livre dépassent le budget de création
        // (11 points de caractéristiques et/ou 7 points de talents).
        let noms: Vec<_> = hors_budget.iter().map(|(n, _, _)| *n).collect();
        assert_eq!(noms, ["Scientifique", "Militaire", "Policier", "Truand"]);
        assert!(hors_budget[0].2.contains("7 points dépensés"));
        assert!(hors_budget[1].2.contains("11 points répartis") && hors_budget[1].2.contains("7 points dépensés"));
        assert!(hors_budget[2].2.contains("11 points répartis"));
        assert!(hors_budget[3].2.contains("11 points répartis"));
    }

    // ---- Caractéristiques ----

    #[test]
    fn depassement_du_budget_de_caracteristiques() {
        let mut p = fiche(Espece::Humain, carac(3, 3, 2, 2, 1, 2)); // 13 points
        p.finaliser();
        assert_eq!(gravites(&p, Gravite::Budget).len(), 1);
        assert!(gravites(&p, Gravite::Budget)[0].contains("13 points"));
    }

    #[test]
    fn caracteristique_au_dessus_du_maximum_de_l_espece() {
        let mut p = fiche(Espece::Humain, carac(4, 2, 1, 1, 1, 1)); // 10 points, FO 4
        p.finaliser();
        let b = gravites(&p, Gravite::Budget);
        assert_eq!(b.len(), 1);
        assert!(b[0].contains("Force"));
    }

    #[test]
    fn caracteristique_hors_bornes_est_une_erreur_de_structure() {
        let mut p = fiche(Espece::Ange, carac(0, 5, 5, 5, 3, 2));
        p.finaliser();
        assert!(!gravites(&p, Gravite::Structure).is_empty());
        let mut p = fiche(Espece::Ange, carac(7, 5, 3, 2, 2, 1));
        p.finaliser();
        assert!(!gravites(&p, Gravite::Structure).is_empty());
    }

    #[test]
    fn points_non_depenses_sont_un_avertissement() {
        let mut p = fiche(Espece::Humain, carac(2, 2, 1, 1, 1, 1));
        p.finaliser();
        assert!(gravites(&p, Gravite::Budget).is_empty());
        assert!(gravites(&p, Gravite::Avertissement).iter().any(|m| m.contains("caractéristiques")));
    }

    // ---- Talents ----

    #[test]
    fn talents_depassement_et_niveau_trop_haut() {
        let mut p = fiche(Espece::Humain, carac(2, 2, 2, 2, 1, 1));
        p.talents = vec![talent("Esquive", 2), talent("Baratin", 1), talent("Nage", 1)]; // 4+2+2 = 8 > 6
        p.finaliser();
        assert!(gravites(&p, Gravite::Budget).iter().any(|m| m.contains("8 points")));

        p.talents = vec![talent("Esquive", 3)];
        assert!(!gravites(&p, Gravite::Structure).is_empty(), "+3 est invalide");
    }

    #[test]
    fn talent_en_double_et_talent_hors_catalogue() {
        let mut p = fiche(Espece::Humain, carac(2, 2, 2, 2, 1, 1));
        p.talents = vec![talent("Esquive", 0), talent("esquive", 0), talent("Vol à la tire", 0)];
        p.finaliser();
        assert!(gravites(&p, Gravite::Structure).iter().any(|m| m.contains("en double")));
        assert!(gravites(&p, Gravite::Avertissement).iter().any(|m| m.contains("absent du catalogue")));
    }

    // ---- Pouvoirs et PP ----

    #[test]
    fn humain_sans_pouvoir() {
        let mut p = fiche(Espece::Humain, carac(2, 2, 2, 2, 1, 1));
        p.pouvoirs = vec![pouvoir(112, 0)];
        p.finaliser();
        assert!(gravites(&p, Gravite::Budget).iter().any(|m| m.contains("humains")));
    }

    fn demon_type() -> Personnage {
        let mut p = fiche(Espece::Demon, carac(3, 4, 3, 2, 3, 3)); // 18 points
        p.talents = vec![talent("Arme de poing", 2), talent("Esquive", 2), talent("Baratin", 1)]; // 10
        p.pouvoirs = vec![pouvoir(321, 0), pouvoir(112, 0), pouvoir(211, 0), pouvoir(311, 0)];
        p.finaliser();
        p
    }

    #[test]
    fn demon_conforme_au_budget_et_pp() {
        let p = demon_type();
        assert!(controler(&p).is_empty(), "{:?}", controler(&p));
        // Volonté 4 + 4 tirages.
        assert_eq!(p.pp_max(), 8);
        assert_eq!(p.pp_courants(), 8);
        assert_eq!(p.pouvoirs[0].nom, "Télépathie");
    }

    #[test]
    fn niveau_de_pouvoir_compte_comme_un_tirage_supplementaire() {
        let mut p = demon_type();
        p.pouvoirs = vec![pouvoir(321, 1), pouvoir(112, 1)]; // 2 + 2 tirages
        p.finaliser();
        assert!(controler(&p).is_empty(), "{:?}", controler(&p));
        assert_eq!(p.tirages(), 4);
        p.pouvoirs[0].niveau = 3; // niveau 3 > max +2 ; 4 + 2 = 6 tirages > 4
        let b = gravites(&p, Gravite::Budget);
        assert!(b.iter().any(|m| m.contains("maximum +2")), "{b:?}");
        assert!(b.iter().any(|m| m.contains("tirages")), "{b:?}");
    }

    #[test]
    fn limitation_donne_un_pouvoir_et_deux_pp() {
        let mut p = demon_type();
        p.pouvoirs.push(pouvoir(613, 0)); // limitation : Cicatrice
        p.pouvoirs.push(pouvoir(121, 0)); // pouvoir supplémentaire
        p.finaliser();
        assert!(controler(&p).is_empty(), "{:?}", controler(&p));
        assert_eq!(p.pp_max(), 4 + 4 + 2, "VO + 4 pouvoirs + limitation + pouvoir de plus");
    }

    #[test]
    fn pouvoir_supplementaire_sans_limitation_depasse_le_budget() {
        let mut p = demon_type();
        p.pouvoirs.push(pouvoir(121, 0));
        p.finaliser();
        assert!(gravites(&p, Gravite::Budget).iter().any(|m| m.contains("5 tirages")));
    }

    #[test]
    fn deux_limitations_ou_limitation_hors_pj_sont_refusees() {
        let mut p = demon_type();
        p.pouvoirs.push(pouvoir(613, 0));
        p.pouvoirs.push(pouvoir(614, 0));
        p.finaliser();
        assert!(gravites(&p, Gravite::Budget).iter().any(|m| m.contains("limitation")));

        let mut f = fiche(Espece::Familier, carac(3, 3, 3, 2, 2, 1));
        f.pouvoirs = vec![pouvoir(321, 0), pouvoir(613, 0)];
        f.finaliser();
        assert!(gravites(&f, Gravite::Budget).iter().any(|m| m.contains("limitation")));
    }

    #[test]
    fn pouvoir_inconnu_ou_ambigu_est_une_erreur_de_structure() {
        let mut p = demon_type();
        p.pp = None;
        p.pouvoirs = vec![
            pouvoir(999, 0),
            PouvoirPossede { code: None, nom: "Peur".into(), niveau: 0, limitation: false },
            PouvoirPossede { code: None, nom: "Lévitation quantique".into(), niveau: 0, limitation: false },
        ];
        p.finaliser();
        let s = gravites(&p, Gravite::Structure);
        assert_eq!(s.len(), 3, "{s:?}");
        assert!(s.iter().any(|m| m.contains("ambigu")));
    }

    #[test]
    fn pouvoir_designe_par_son_nom() {
        let mut p = demon_type();
        p.pouvoirs[0] = PouvoirPossede { code: None, nom: "telepathie".into(), niveau: 0, limitation: false };
        p.finaliser();
        assert_eq!(p.pouvoirs[0].code, Some(321));
        assert_eq!(p.pouvoirs[0].nom, "Télépathie");
    }

    #[test]
    fn familier_et_serviteur_utilisent_la_bonne_table() {
        // 321 existe côté Démons (Télépathie). Un Serviteur lit la table des Anges.
        let mut f = fiche(Espece::Familier, carac(3, 3, 3, 2, 2, 1));
        f.pouvoirs = vec![pouvoir(321, 0)];
        f.finaliser();
        assert_eq!(f.pouvoirs[0].nom, "Télépathie");
        assert_eq!(Espece::Serviteur.livre_pouvoirs(), Some(Livre::Ange));
        assert_eq!(Espece::Soldat.livre_pouvoirs(), Some(Livre::Ange));
        assert_eq!(Espece::MortVivant.livre_pouvoirs(), Some(Livre::Demon));
    }

    #[test]
    fn pp_courants_ne_depassent_pas_le_maximum() {
        let mut p = demon_type();
        p.pp = Some(99);
        assert!(gravites(&p, Gravite::Structure).iter().any(|m| m.contains("PP courants")));
    }

    #[test]
    fn creature_sans_pouvoir_a_zero_pp() {
        let mut p = fiche(Espece::Humain, carac(2, 2, 2, 2, 1, 1));
        p.finaliser();
        assert_eq!((p.pp_max(), p.pp_courants()), (0, 0));
    }

    // ---- Évolution de l'état ----

    #[test]
    fn depenser_et_recuperer_des_pp() {
        let mut p = demon_type(); // 8 PP
        assert_eq!(p.depenser_pp(3), Ok(5));
        assert!(p.depenser_pp(6).is_err());
        assert_eq!(p.pp_courants(), 5, "un échec ne consomme rien");
        assert_eq!(p.recuperer_pp(2), 7);
        assert_eq!(p.recuperer_pp(10), 8, "plafonné au maximum");
    }

    #[test]
    fn dommages_ne_descendent_pas_sous_zero() {
        let mut p = demon_type();
        assert_eq!(p.ajuster_dommages(3), 3);
        assert_eq!(p.ajuster_dommages(-1), 2);
        assert_eq!(p.ajuster_dommages(-10), 0);
    }

    #[test]
    fn identifiant_stable_et_sans_accent() {
        assert_eq!(identifiant("Mnofriytkgh l'Éveillé"), "mnofriytkgh l eveille".replace(' ', "-"));
        let mut p = demon_type();
        p.nom = "Gustave Dupré".into();
        p.finaliser();
        assert_eq!(p.id, "gustave-dupre");
    }

    // ---- Préparation de tests depuis la fiche ----

    #[test]
    fn test_avec_talent_de_la_fiche() {
        let p = demon_type(); // PE 3, Arme de poing +2
        let mut d = DemandeTest::caracteristique(Carac::PE, 1);
        d.talent = Some("arme de poing".into());
        let t = p.preparer_test(&d).unwrap();
        assert_eq!(t.action.niveau.valeur(), 2);
        assert_eq!(t.action.caracteristique.valeur(), 3);
        assert!(t.description.contains("Arme de poing +2"), "{}", t.description);
    }

    #[test]
    fn test_sans_le_talent_utilise_le_malus_du_catalogue() {
        let p = demon_type();
        let mut d = DemandeTest::caracteristique(Carac::AG, 1);
        d.talent = Some("Acrobatie".into()); // malus -3
        let t = p.preparer_test(&d).unwrap();
        assert_eq!(t.action.niveau.valeur(), 0);
        assert_eq!(t.action.modificateur, -3);
    }

    #[test]
    fn test_talent_inconnu_demande_un_malus() {
        let p = demon_type();
        let mut d = DemandeTest::caracteristique(Carac::AG, 1);
        d.talent = Some("Apiculture".into());
        assert!(p.preparer_test(&d).unwrap_err().contains("malus_sans_talent"));
        d.malus_sans_talent = Some(4);
        assert_eq!(p.preparer_test(&d).unwrap().action.modificateur, -4);
        // Talent à malus variable du catalogue : même exigence.
        d.malus_sans_talent = None;
        d.talent = Some("Spécialisation".into());
        assert!(p.preparer_test(&d).is_err());
    }

    #[test]
    fn test_avec_pouvoir_caracteristique_seule_et_modificateurs() {
        let p = demon_type(); // PR 3, pouvoir Télépathie +0
        let mut d = DemandeTest::caracteristique(Carac::PR, 1);
        d.pouvoir = Some("Télépathie".into());
        d.modificateur = -1;
        d.malus_volontaire = 2;
        let t = p.preparer_test(&d).unwrap();
        assert_eq!((t.action.niveau.valeur(), t.action.modificateur, t.action.malus_volontaire), (0, -1, 2));

        d.pouvoir = Some("Feu".into());
        assert!(p.preparer_test(&d).unwrap_err().contains("ne possède pas"));

        let seule = DemandeTest::caracteristique(Carac::FO, 2);
        let t = p.preparer_test(&seule).unwrap();
        assert_eq!(t.action.niveau, NiveauTalent::FACILE);
        assert!(p.preparer_test(&DemandeTest::caracteristique(Carac::FO, 4)).is_err());
    }

    #[test]
    fn test_ne_cumule_pas_talent_et_pouvoir() {
        let p = demon_type();
        let mut d = DemandeTest::caracteristique(Carac::PE, 1);
        d.talent = Some("Esquive".into());
        d.pouvoir = Some("Télépathie".into());
        assert!(p.preparer_test(&d).is_err());
    }

    #[test]
    fn analyse_des_noms_de_caracteristiques() {
        assert_eq!(Carac::analyser("Volonté"), Some(Carac::VO));
        assert_eq!(Carac::analyser("agilite"), Some(Carac::AG));
        assert_eq!(Carac::analyser("pe"), Some(Carac::PE));
        assert_eq!(Carac::analyser("charisme"), None);
    }

    #[test]
    fn serialisation_round_trip() {
        let p = demon_type();
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Personnage>(&json).unwrap(), p);
    }
}
