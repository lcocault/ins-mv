//! Catalogues des pouvoirs et des talents, générés depuis `livres/` par
//! `moteur/outils/generer_catalogues.py` (données dans `donnees/`).

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// Livre de règles d'où provient une entrée de catalogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Livre {
    /// Magna Veritas.
    Ange,
    /// In Nomine Satanis.
    Demon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pouvoir {
    pub livre: Livre,
    /// Numéro à trois chiffres de la table des pouvoirs ; absent pour les pouvoirs « spéciaux ».
    pub code: Option<u16>,
    pub nom: String,
    pub famille: String,
    pub categorie: String,
    /// Caractéristique d'attaque.
    pub attaque: Option<String>,
    /// Caractéristique de défense.
    pub defense: Option<String>,
    /// Coût en points de pouvoir, tel qu'imprimé.
    pub cout: Option<String>,
    /// Caractéristique du jet de pouvoir quand le livre l'indique (ex. « (Volonté) » → VO).
    #[serde(default)]
    pub jet: Option<String>,
    /// Texte de la règle dans le livre (ou introduction de sa catégorie).
    #[serde(default)]
    pub description: Option<String>,
}

impl Pouvoir {
    /// Coût en PP d'une utilisation quand il est fixe : « 4 », « 2/attaque », « 1/défense ».
    /// `None` pour un coût variable (« 1, 2 ou 3/att. », « 1/minute », « La moitié... »).
    /// Un pouvoir sans coût imprimé est gratuit (0).
    pub fn cout_fixe(&self) -> Option<u32> {
        let texte = match self.cout.as_deref().map(str::trim) {
            None | Some("") => return Some(0),
            Some(t) => t,
        };
        let (nombre, suite) = match texte.split_once('/') {
            Some((n, s)) => (n.trim(), Some(s.trim())),
            None => (texte, None),
        };
        let valeur: u32 = nombre.parse().ok()?;
        match suite {
            None | Some("attaque") | Some("att.") | Some("défense") => Some(valeur),
            _ => None,
        }
    }

    /// Les pouvoirs 6xx sont des limitations (défauts), qui comptent quand même pour les PP.
    pub fn est_limitation(&self) -> bool {
        self.code.is_some_and(|c| (600..700).contains(&c))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talent {
    pub nom: String,
    pub categorie: String,
    /// Malus en colonnes quand on ne possède pas le talent ; `None` si variable (à fixer par le MJ).
    pub malus: Option<i32>,
    /// Variante du malus dans le livre des Démons, quand elle diffère.
    #[serde(default)]
    pub malus_demon: Option<i32>,
    /// Caractéristiques conseillées (abréviations FO, PR, VO, PE, AG, AP).
    pub caracteristiques: Vec<String>,
}

impl Talent {
    pub fn malus_pour(&self, livre: Livre) -> Option<i32> {
        match livre {
            Livre::Demon => self.malus_demon.or(self.malus),
            Livre::Ange => self.malus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CategorieArme {
    Contact,
    Distance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteArme {
    pub id: String,
    pub texte: String,
}

/// Arme des tableaux de `commun-09-confrontation-armes-combat.md`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arme {
    pub nom: String,
    pub categorie: CategorieArme,
    /// Ajustement aux unités pour les dommages ; `None` si « Spécial » ou rafale seule.
    pub puissance: Option<i32>,
    pub puissance_speciale: bool,
    /// Puissance en rafale, par cible touchée.
    pub puissance_rafale: Option<i32>,
    /// Ajustement en colonnes du jet de toucher.
    pub precision: i32,
    pub portee: Option<u32>,
    pub coups: Option<String>,
    pub talent: String,
    pub notes: Vec<NoteArme>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protection {
    pub nom: String,
    /// Retranchée aux dommages.
    pub protection: i32,
    /// Malus en colonnes pour les talents d'agilité du porteur (négatif).
    pub ajustement: i32,
    pub notes: Vec<NoteArme>,
}

static ARMES: OnceLock<Vec<Arme>> = OnceLock::new();
static PROTECTIONS: OnceLock<Vec<Protection>> = OnceLock::new();

pub fn armes() -> &'static [Arme] {
    ARMES.get_or_init(|| serde_json::from_str(include_str!("../donnees/armes.json")).expect("armes.json valide"))
}

pub fn protections() -> &'static [Protection] {
    PROTECTIONS.get_or_init(|| {
        serde_json::from_str(include_str!("../donnees/protections.json")).expect("protections.json valide")
    })
}

/// Arme par nom (insensible aux accents et à la casse). Deux armes portent le même
/// nom (« Hachette » au contact et à distance) : `categorie` lève l'ambiguïté.
pub fn arme_par_nom(nom: &str, categorie: Option<CategorieArme>) -> Vec<&'static Arme> {
    let cible = normaliser(nom);
    armes()
        .iter()
        .filter(|a| normaliser(&a.nom) == cible && categorie.is_none_or(|c| c == a.categorie))
        .collect()
}

pub fn protection_par_nom(nom: &str) -> Option<&'static Protection> {
    let cible = normaliser(nom);
    protections().iter().find(|p| normaliser(&p.nom) == cible)
}

static POUVOIRS: OnceLock<Vec<Pouvoir>> = OnceLock::new();
static TALENTS: OnceLock<Vec<Talent>> = OnceLock::new();

pub fn pouvoirs() -> &'static [Pouvoir] {
    POUVOIRS.get_or_init(|| {
        serde_json::from_str(include_str!("../donnees/pouvoirs.json")).expect("pouvoirs.json valide")
    })
}

pub fn talents() -> &'static [Talent] {
    TALENTS.get_or_init(|| {
        serde_json::from_str(include_str!("../donnees/talents.json")).expect("talents.json valide")
    })
}

/// Minuscules sans accents ni ponctuation : « Hélicoptère » ≈ « helicoptere ».
pub fn normaliser(texte: &str) -> String {
    texte
        .chars()
        .flat_map(char::to_lowercase)
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            '\'' | '’' | '-' => ' ',
            c => c,
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pouvoir_par_code(livre: Livre, code: u16) -> Option<&'static Pouvoir> {
    pouvoirs().iter().find(|p| p.livre == livre && p.code == Some(code))
}

/// Recherche exacte (insensible aux accents et à la casse) d'un pouvoir par son nom.
/// Plusieurs pouvoirs peuvent porter le même nom (ex. « Charme » en 131 et 161).
pub fn pouvoirs_par_nom(livre: Livre, nom: &str) -> Vec<&'static Pouvoir> {
    let cible = normaliser(nom);
    pouvoirs().iter().filter(|p| p.livre == livre && normaliser(&p.nom) == cible).collect()
}

/// Recherche libre : nom, catégorie ou famille contenant la requête.
pub fn chercher_pouvoirs(livre: Livre, requete: &str) -> Vec<&'static Pouvoir> {
    let cible = normaliser(requete);
    pouvoirs()
        .iter()
        .filter(|p| p.livre == livre)
        .filter(|p| {
            [&p.nom, &p.categorie, &p.famille].iter().any(|t| normaliser(t).contains(&cible))
        })
        .collect()
}

pub fn talent_par_nom(nom: &str) -> Option<&'static Talent> {
    let cible = normaliser(nom);
    talents().iter().find(|t| normaliser(&t.nom) == cible)
}

pub fn chercher_talents(requete: &str) -> Vec<&'static Talent> {
    let cible = normaliser(requete);
    talents()
        .iter()
        .filter(|t| normaliser(&t.nom).contains(&cible) || normaliser(&t.categorie).contains(&cible))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_livre_a_216_pouvoirs_numerotes_plus_des_speciaux() {
        for livre in [Livre::Ange, Livre::Demon] {
            let numerotes = pouvoirs().iter().filter(|p| p.livre == livre && p.code.is_some()).count();
            let speciaux = pouvoirs().iter().filter(|p| p.livre == livre && p.code.is_none()).count();
            assert_eq!(numerotes, 216, "{livre:?}");
            assert_eq!(speciaux, 10, "{livre:?}");
        }
    }

    #[test]
    fn codes_uniques_et_dans_la_table_d666() {
        for livre in [Livre::Ange, Livre::Demon] {
            let mut vus = std::collections::HashSet::new();
            for p in pouvoirs().iter().filter(|p| p.livre == livre) {
                if let Some(code) = p.code {
                    assert!(vus.insert(code), "doublon {code} ({livre:?})");
                    let (x, y, z) = (code / 100, code / 10 % 10, code % 10);
                    assert!([x, y, z].iter().all(|d| (1..=6).contains(d)), "code invalide {code}");
                }
            }
        }
    }

    #[test]
    fn recherche_par_code_et_par_nom() {
        assert_eq!(pouvoir_par_code(Livre::Ange, 112).unwrap().nom, "Coup de poing");
        assert_eq!(pouvoir_par_code(Livre::Demon, 321).unwrap().nom, "Télépathie");
        assert!(pouvoir_par_code(Livre::Ange, 999).is_none());
        let charme = pouvoirs_par_nom(Livre::Ange, "charme");
        assert_eq!(charme.iter().filter_map(|p| p.code).collect::<Vec<_>>(), vec![131, 161]);
    }

    #[test]
    fn limitations_sont_les_codes_6xx() {
        assert!(pouvoir_par_code(Livre::Ange, 613).unwrap().est_limitation());
        assert!(!pouvoir_par_code(Livre::Ange, 566).unwrap().est_limitation());
        assert!(pouvoirs().iter().filter(|p| p.est_limitation()).count() == 72);
    }

    #[test]
    fn recherche_libre_ignore_accents_et_casse() {
        assert!(!chercher_pouvoirs(Livre::Demon, "TELEPATHIE").is_empty());
        assert!(!chercher_pouvoirs(Livre::Ange, "attaque a distance").is_empty());
    }

    #[test]
    fn talents_malus_et_variante_comedie() {
        assert_eq!(talent_par_nom("esquive").unwrap().malus, Some(-1));
        assert_eq!(talent_par_nom("Hélicoptere").unwrap().malus, Some(-4));
        let comedie = talent_par_nom("Comédie").unwrap();
        assert_eq!(comedie.malus_pour(Livre::Ange), Some(-3));
        assert_eq!(comedie.malus_pour(Livre::Demon), Some(-4));
        assert_eq!(talent_par_nom("Arme d'épaule").unwrap().malus, Some(-2));
        assert_eq!(talent_par_nom("arme d epaule").unwrap().malus, Some(-2));
        assert!(talent_par_nom("Inexistant").is_none());
    }

    #[test]
    fn armes_du_livre() {
        assert_eq!(armes().len(), 28);
        let epee = arme_par_nom("epee longue", None);
        assert_eq!(epee.len(), 1);
        assert_eq!((epee[0].puissance, epee[0].precision, epee[0].talent.as_str()), (Some(2), 0, "Arme de contact"));
        // Précision en colonnes : hachette de contact -1, carabine +1.
        assert_eq!(arme_par_nom("Hachette", Some(CategorieArme::Contact))[0].precision, -1);
        assert_eq!(arme_par_nom("Hachette", Some(CategorieArme::Distance))[0].puissance, Some(0));
        assert_eq!(arme_par_nom("Hachette", None).len(), 2, "nom ambigu sans catégorie");
        assert_eq!(arme_par_nom("carabine", None)[0].precision, 1);
        assert_eq!(arme_par_nom("carabine", None)[0].portee, Some(100));
    }

    #[test]
    fn armes_speciales_et_rafales() {
        let uzi = &arme_par_nom("Uzi", None)[0];
        assert_eq!((uzi.puissance, uzi.puissance_rafale), (Some(1), Some(3)));
        let m60 = &arme_par_nom("M-60", None)[0];
        assert_eq!((m60.puissance, m60.puissance_rafale), (None, Some(6)));
        let boule = &arme_par_nom("Coup de boule", None)[0];
        assert!(boule.puissance_speciale && boule.puissance.is_none());
        assert_eq!(boule.notes[0].id, "b");
        assert!(arme_par_nom("Lance-grenade", None)[0].puissance_speciale);
    }

    #[test]
    fn protections_du_livre() {
        assert_eq!(protections().len(), 4);
        let acier = protection_par_nom("acier").unwrap();
        assert_eq!((acier.protection, acier.ajustement), (6, -3));
        assert_eq!(protection_par_nom("Cuir").unwrap().ajustement, 0);
        let kevlar = protection_par_nom("Kevlar").unwrap();
        assert_eq!((kevlar.protection, kevlar.ajustement), (2, -1));
        assert!(kevlar.notes[0].texte.contains("armes à feu"));
        assert!(protection_par_nom("Plastron en carton").is_none());
    }

    #[test]
    fn descriptions_et_caracteristiques_de_jet() {
        for livre in [Livre::Ange, Livre::Demon] {
            for p in pouvoirs().iter().filter(|p| p.livre == livre && p.code.is_some()) {
                assert!(p.description.as_deref().is_some_and(|d| d.len() > 5), "{livre:?} {:?} sans texte", p.code);
            }
        }
        let calme = pouvoir_par_code(Livre::Ange, 166).unwrap();
        assert_eq!(calme.jet.as_deref(), Some("VO"));
        assert!(calme.description.as_deref().unwrap().contains("discuter"));
        // Les augmentations temporaires de caractéristiques se jouent à la Volonté.
        assert_eq!(pouvoir_par_code(Livre::Ange, 243).unwrap().jet.as_deref(), Some("VO"));
        assert_eq!(pouvoir_par_code(Livre::Demon, 241).unwrap().jet.as_deref(), Some("VO"));
        // Variantes de livre résolues : « l'ange » (MV) contre « le Démon » (INS).
        assert!(pouvoir_par_code(Livre::Ange, 243).unwrap().description.as_deref().unwrap().contains("l'ange"));
        assert!(pouvoir_par_code(Livre::Demon, 243).unwrap().description.as_deref().unwrap().contains("Démon"));
    }

    #[test]
    fn couts_fixes_ou_variables() {
        let cout = |livre, code| pouvoir_par_code(livre, code).unwrap().cout_fixe();
        assert_eq!(cout(Livre::Ange, 243), Some(1), "« 1 »");
        assert_eq!(cout(Livre::Ange, 321), Some(1));
        assert_eq!(cout(Livre::Ange, 166), Some(1), "« 1/attaque »");
        assert_eq!(cout(Livre::Ange, 131), Some(4), "« 4/attaque »");
        assert_eq!(cout(Livre::Ange, 413), Some(0), "pas de coût imprimé");
        assert_eq!(cout(Livre::Ange, 121), None, "« 1, 2 ou 3/att. »");
        assert_eq!(cout(Livre::Ange, 113), None, "« 1/minute »");
        assert_eq!(cout(Livre::Ange, 151), None, "« La moitié (M. 4) »");
    }

    #[test]
    fn talents_a_malus_variable() {
        assert_eq!(talent_par_nom("Spécialisation").unwrap().malus, None);
    }
}
