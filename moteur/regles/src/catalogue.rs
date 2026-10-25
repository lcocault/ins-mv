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
}

impl Pouvoir {
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
    fn talents_a_malus_variable() {
        assert_eq!(talent_par_nom("Spécialisation").unwrap().malus, None);
    }
}
