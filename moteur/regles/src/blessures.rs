//! Effets des blessures (`regles/ange-12-blessures.md`, `regles/demon-12-blessures.md`).
//!
//! Les dommages s'accumulent sur la fiche (`Personnage::dommages`). Selon l'espèce,
//! un certain nombre de points de dommage retire un point de Force ; la Force
//! courante détermine l'évanouissement, le coma et la mort. Les jets de table UM
//! qui utilisent la Force prennent en compte la valeur d'origine : la Force
//! courante ne sert qu'à ces seuils.

use serde::Serialize;

use crate::personnage::{Espece, Personnage};

/// Règles de blessure d'une espèce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RegimeBlessures {
    /// Points de dommage nécessaires pour perdre un point de Force.
    pub dommages_par_point_de_force: u32,
    /// Force à laquelle le personnage est assommé ou évanoui (`None` : jamais).
    pub seuil_assomme: Option<i32>,
    /// Force à laquelle il tombe dans le coma (humains seulement).
    pub seuil_coma: Option<i32>,
    /// Force à laquelle il meurt.
    pub seuil_mort: i32,
    /// Un blessé subit un malus de colonnes égal à ses dommages non guéris.
    pub malus_de_blessure: bool,
}

impl Espece {
    pub fn regime_blessures(self) -> RegimeBlessures {
        let r = |dommages, assomme, coma, mort, malus| RegimeBlessures {
            dommages_par_point_de_force: dommages,
            seuil_assomme: assomme,
            seuil_coma: coma,
            seuil_mort: mort,
            malus_de_blessure: malus,
        };
        match self {
            // Évanoui D6 minutes à 0, coma D6 jours à -1, mort à -2.
            Self::Humain | Self::Serviteur | Self::Soldat => r(1, Some(0), Some(-1), -2, true),
            // Aucune douleur : ni assommé ni malus.
            Self::MortVivant => r(1, None, None, -2, false),
            Self::Familier => r(2, Some(-1), None, -2, true),
            Self::Demon => r(2, Some(-2), None, -3, false),
            Self::Ange => r(3, Some(-2), None, -3, false),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EtatSante {
    Valide,
    /// Évanoui ou assommé ; la durée est donnée par `duree_des`.
    Inconscient,
    Coma,
    Mort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Sante {
    pub force_perdue: u32,
    /// Force d'origine moins la Force perdue (peut être négative).
    pub force_courante: i32,
    pub etat: EtatSante,
    /// Malus en colonnes sur toutes les actions.
    pub malus_colonnes: u32,
    /// Durée de l'inconscience à tirer (« D6 secondes », « D6 minutes »…) quand elle s'applique.
    pub duree: Option<&'static str>,
}

impl Personnage {
    pub fn force_perdue(&self) -> u32 {
        self.dommages / self.espece.regime_blessures().dommages_par_point_de_force
    }

    pub fn sante(&self) -> Sante {
        let regime = self.espece.regime_blessures();
        let perdue = self.force_perdue();
        let courante = i32::from(self.caracteristiques.fo) - perdue as i32;
        let (etat, duree) = if courante <= regime.seuil_mort {
            (EtatSante::Mort, None)
        } else if regime.seuil_coma.is_some_and(|s| courante <= s) {
            (EtatSante::Coma, Some("D6 jours"))
        } else if regime.seuil_assomme.is_some_and(|s| courante <= s) {
            let duree = match self.espece {
                Espece::Humain | Espece::Serviteur | Espece::Soldat => "D6 minutes",
                _ => "D6 secondes",
            };
            (EtatSante::Inconscient, Some(duree))
        } else {
            (EtatSante::Valide, None)
        };
        Sante {
            force_perdue: perdue,
            force_courante: courante,
            etat,
            malus_colonnes: if regime.malus_de_blessure { self.dommages } else { 0 },
            duree,
        }
    }

    /// Soins naturels : chaque semaine de repos rend un point de Force, soit
    /// `dommages_par_point_de_force` points de dommage. Les points de dommage
    /// restants qui n'atteignent pas un point de Force disparaissent la semaine
    /// où il n'y a plus de Force perdue.
    pub fn soigner_semaines(&mut self, semaines: u32) -> u32 {
        let pas = self.espece.regime_blessures().dommages_par_point_de_force;
        for _ in 0..semaines {
            self.dommages = if self.dommages >= pas { self.dommages - pas } else { 0 };
        }
        self.dommages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::personnage::{Caracteristiques, Role};

    fn perso(espece: Espece, fo: u8) -> Personnage {
        Personnage {
            id: "t".into(),
            nom: "T".into(),
            espece,
            role: Role::Pnj,
            caracteristiques: Caracteristiques { fo, pr: 2, vo: 2, pe: 2, ag: 2, ap: 2 },
            talents: vec![],
            pouvoirs: vec![],
            patron: None,
            pp: None,
            dommages: 0,
            equipement: vec![],
            notes: String::new(),
        }
    }

    fn avec(espece: Espece, fo: u8, dommages: u32) -> Sante {
        let mut p = perso(espece, fo);
        p.dommages = dommages;
        p.sante()
    }

    #[test]
    fn regimes_conformes_aux_livres() {
        let r = |e: Espece| {
            let r = e.regime_blessures();
            (r.dommages_par_point_de_force, r.seuil_assomme, r.seuil_coma, r.seuil_mort, r.malus_de_blessure)
        };
        assert_eq!(r(Espece::Humain), (1, Some(0), Some(-1), -2, true));
        assert_eq!(r(Espece::Serviteur), r(Espece::Humain));
        assert_eq!(r(Espece::Soldat), r(Espece::Humain));
        assert_eq!(r(Espece::MortVivant), (1, None, None, -2, false));
        assert_eq!(r(Espece::Familier), (2, Some(-1), None, -2, true));
        assert_eq!(r(Espece::Demon), (2, Some(-2), None, -3, false));
        assert_eq!(r(Espece::Ange), (3, Some(-2), None, -3, false));
    }

    #[test]
    fn humain_de_force_2_evanoui_coma_mort() {
        // Force 2 : 2 dommages → 0 (évanoui), 3 → -1 (coma), 4 → -2 (mort).
        assert_eq!(avec(Espece::Humain, 2, 1).etat, EtatSante::Valide);
        let s = avec(Espece::Humain, 2, 2);
        assert_eq!((s.etat, s.force_courante, s.duree), (EtatSante::Inconscient, 0, Some("D6 minutes")));
        let s = avec(Espece::Humain, 2, 3);
        assert_eq!((s.etat, s.duree), (EtatSante::Coma, Some("D6 jours")));
        assert_eq!(avec(Espece::Humain, 2, 4).etat, EtatSante::Mort);
        assert_eq!(avec(Espece::Humain, 2, 4).duree, None);
    }

    #[test]
    fn humain_blesse_subit_un_malus_egal_aux_dommages() {
        assert_eq!(avec(Espece::Humain, 3, 2).malus_colonnes, 2);
        assert_eq!(avec(Espece::Soldat, 3, 1).malus_colonnes, 1);
        assert_eq!(avec(Espece::Familier, 4, 3).malus_colonnes, 3);
    }

    #[test]
    fn mort_vivant_sans_malus_ni_evanouissement() {
        // Force 4 : meurt à -2, soit 6 points de Force perdus.
        let s = avec(Espece::MortVivant, 4, 5);
        assert_eq!((s.etat, s.malus_colonnes, s.force_courante), (EtatSante::Valide, 0, -1));
        assert_eq!(avec(Espece::MortVivant, 4, 6).etat, EtatSante::Mort);
    }

    #[test]
    fn familier_perd_un_point_de_force_pour_deux_dommages() {
        // Force 3 : 2 dommages → 2, 8 → -1 (assommé D6 secondes), 10 → -2 (mort).
        let s = avec(Espece::Familier, 3, 3);
        assert_eq!((s.force_perdue, s.force_courante), (1, 2));
        let s = avec(Espece::Familier, 3, 8);
        assert_eq!((s.etat, s.duree), (EtatSante::Inconscient, Some("D6 secondes")));
        assert_eq!(avec(Espece::Familier, 3, 10).etat, EtatSante::Mort);
    }

    #[test]
    fn demon_force_4() {
        // Perd 1 point pour 2 dommages : -2 à 12 dommages (assommé), -3 à 14 (mort).
        assert_eq!(avec(Espece::Demon, 4, 11).etat, EtatSante::Valide);
        assert_eq!(avec(Espece::Demon, 4, 12).etat, EtatSante::Inconscient);
        assert_eq!(avec(Espece::Demon, 4, 13).etat, EtatSante::Inconscient);
        assert_eq!(avec(Espece::Demon, 4, 14).etat, EtatSante::Mort);
    }

    #[test]
    fn ange_jean_de_lavenue_force_4() {
        // Perd 1 point de Force pour 3 dommages.
        let s = avec(Espece::Ange, 4, 8);
        assert_eq!((s.force_perdue, s.force_courante, s.malus_colonnes), (2, 2, 0));
        assert_eq!(avec(Espece::Ange, 4, 17).etat, EtatSante::Valide, "force -1");
        let s = avec(Espece::Ange, 4, 18);
        assert_eq!((s.etat, s.force_courante, s.duree), (EtatSante::Inconscient, -2, Some("D6 secondes")));
        assert_eq!(avec(Espece::Ange, 4, 20).etat, EtatSante::Inconscient);
        assert_eq!(avec(Espece::Ange, 4, 21).etat, EtatSante::Mort);
    }

    #[test]
    fn soins_naturels_une_force_par_semaine() {
        let mut p = perso(Espece::Ange, 4);
        p.dommages = 7; // 2 Force perdue + 1 de reste
        assert_eq!(p.soigner_semaines(1), 4);
        assert_eq!(p.soigner_semaines(1), 1);
        assert_eq!(p.soigner_semaines(1), 0);
        assert_eq!(p.soigner_semaines(5), 0);

        let mut h = perso(Espece::Humain, 2);
        h.dommages = 3;
        assert_eq!(h.soigner_semaines(2), 1);
    }

    #[test]
    fn la_force_des_jets_reste_celle_d_origine() {
        let mut p = perso(Espece::Humain, 3);
        p.dommages = 2;
        assert_eq!(p.sante().force_courante, 1);
        assert_eq!(p.caracteristiques.fo, 3, "la fiche garde la valeur d'origine");
    }
}
