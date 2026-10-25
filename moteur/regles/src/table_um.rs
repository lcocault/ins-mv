//! La table unique multiple (`livres/commun-05-table-um.md`).
//!
//! La table a 18 colonnes : les six caractéristiques y occupent des positions
//! inégales (les autres colonnes, notées « - », servent aux décalages) et les
//! lignes sont les niveaux de talent 0 à 3.

use serde::{Deserialize, Serialize};

use crate::des::{SourceDes, D666};
use crate::ErreurRegle;

const NB_COLONNES: i32 = 18;

/// Seuils de la table UM. Ligne = niveau de talent (0 à 3).
const TABLE: [[u8; NB_COLONNES as usize]; 4] = [
    [11, 13, 21, 22, 23, 24, 25, 26, 32, 33, 34, 35, 41, 43, 46, 53, 61, 66],
    [13, 21, 32, 34, 36, 43, 45, 51, 54, 55, 56, 62, 64, 66, 66, 66, 66, 66],
    [16, 24, 43, 46, 54, 62, 63, 64, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66],
    [31, 43, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66, 66],
];

/// Position de chaque caractéristique (1 à 6) dans l'en-tête de la table.
const COLONNE_CARACTERISTIQUE: [i32; 6] = [1, 2, 5, 8, 13, 17];

/// Valeur d'une caractéristique, de 1 à 6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Caracteristique(u8);

impl Caracteristique {
    pub fn new(valeur: u8) -> Result<Self, ErreurRegle> {
        if (1..=6).contains(&valeur) {
            Ok(Self(valeur))
        } else {
            Err(ErreurRegle::Caracteristique(valeur))
        }
    }

    pub fn valeur(self) -> u8 {
        self.0
    }

    fn colonne(self) -> i32 {
        COLONNE_CARACTERISTIQUE[usize::from(self.0) - 1]
    }
}

/// Niveau de talent, de 0 (amateur) à 3. Les difficultés sans talent suivent
/// le même barème : 0 difficile, +1 moyen, +2 facile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NiveauTalent(u8);

impl NiveauTalent {
    pub const DIFFICILE: Self = Self(0);
    pub const MOYEN: Self = Self(1);
    pub const FACILE: Self = Self(2);

    pub fn new(niveau: u8) -> Result<Self, ErreurRegle> {
        if niveau <= 3 {
            Ok(Self(niveau))
        } else {
            Err(ErreurRegle::NiveauTalent(niveau))
        }
    }

    pub fn valeur(self) -> u8 {
        self.0
    }
}

/// Seuil de réussite obtenu après lecture de la table et des décalages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "valeur", rename_all = "snake_case")]
pub enum Seuil {
    /// Décalé au-delà de la colonne « - » : l'action échoue obligatoirement.
    EchecAutomatique,
    /// Le D66 `xy` doit être inférieur ou égal à cette valeur.
    Valeur(u8),
    /// Décalé au-delà de la colonne « 6 » : l'action réussit obligatoirement.
    ReussiteAutomatique,
}

/// Lit le seuil pour une colonne (éventuellement hors table) et un niveau.
pub fn seuil(colonne: i32, niveau: NiveauTalent) -> Seuil {
    if colonne < 0 {
        Seuil::EchecAutomatique
    } else if colonne >= NB_COLONNES {
        Seuil::ReussiteAutomatique
    } else {
        Seuil::Valeur(TABLE[usize::from(niveau.0)][colonne as usize])
    }
}

/// Une action à résoudre sur la table UM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    pub caracteristique: Caracteristique,
    pub niveau: NiveauTalent,
    /// Décalage accordé par le maître de jeu, en colonnes (bonus > 0, malus < 0).
    pub modificateur: i32,
    /// Malus que le joueur s'inflige volontairement, en colonnes (≥ 0). Il
    /// décale vers la gauche et s'ajoute aux unités du jet.
    pub malus_volontaire: u8,
}

impl Action {
    pub fn new(caracteristique: Caracteristique, niveau: NiveauTalent) -> Self {
        Self { caracteristique, niveau, modificateur: 0, malus_volontaire: 0 }
    }

    /// Action tentée sans posséder le talent : ligne 0 avec un malus de `x`
    /// colonnes (le malus indiqué entre parenthèses après le nom du talent).
    pub fn sans_talent(caracteristique: Caracteristique, malus_colonnes: u8) -> Self {
        Self {
            modificateur: -i32::from(malus_colonnes),
            ..Self::new(caracteristique, NiveauTalent(0))
        }
    }

    pub fn avec_modificateur(mut self, colonnes: i32) -> Self {
        self.modificateur += colonnes;
        self
    }

    pub fn avec_malus_volontaire(mut self, colonnes: u8) -> Self {
        self.malus_volontaire = colonnes;
        self
    }

    /// Colonne finale sur la table, avant lecture.
    pub fn colonne(&self) -> i32 {
        self.caracteristique.colonne() + self.modificateur - i32::from(self.malus_volontaire)
    }
}

/// Résultat détaillé d'un jet sur la table UM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResultatJet {
    pub jet: D666,
    pub colonne: i32,
    pub seuil: Seuil,
    pub reussi: bool,
    /// Unités du jet (z) majorées du malus volontaire, à interpréter par le MJ.
    /// En cas d'échec, mesure l'ampleur de l'échec (ex. distance du tir raté).
    pub unites: u8,
}

/// Compare un D666 déjà tiré à l'action.
pub fn evaluer(action: &Action, jet: D666) -> ResultatJet {
    let colonne = action.colonne();
    let seuil = seuil(colonne, action.niveau);
    let reussi = match seuil {
        Seuil::EchecAutomatique => false,
        Seuil::ReussiteAutomatique => true,
        Seuil::Valeur(max) => jet.xy() <= max,
    };
    ResultatJet { jet, colonne, seuil, reussi, unites: jet.z + action.malus_volontaire }
}

/// Tire un D666 et résout l'action.
pub fn resoudre(action: &Action, source: &mut impl SourceDes) -> ResultatJet {
    evaluer(action, D666::tirer(source))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carac(v: u8) -> Caracteristique {
        Caracteristique::new(v).unwrap()
    }
    fn niveau(v: u8) -> NiveauTalent {
        NiveauTalent::new(v).unwrap()
    }
    fn jet(v: u16) -> D666 {
        D666::depuis_valeur(v).unwrap()
    }
    fn seuil_de(action: &Action) -> Seuil {
        seuil(action.colonne(), action.niveau)
    }

    // ---- structure de la table -------------------------------------------

    #[test]
    fn table_croissante_vers_la_droite_et_vers_le_bas() {
        for l in 0..4 {
            for c in 1..NB_COLONNES as usize {
                assert!(TABLE[l][c] >= TABLE[l][c - 1], "ligne {l}, colonne {c}");
            }
        }
        for l in 1..4 {
            for c in 0..NB_COLONNES as usize {
                assert!(TABLE[l][c] >= TABLE[l - 1][c], "ligne {l}, colonne {c}");
            }
        }
    }

    #[test]
    fn table_ne_contient_que_des_d66_valides() {
        for ligne in TABLE {
            for s in ligne {
                assert!((1..=6).contains(&(s / 10)) && (1..=6).contains(&(s % 10)), "{s}");
            }
        }
    }

    #[test]
    fn caracteristiques_aux_bonnes_colonnes() {
        let colonnes: Vec<i32> = (1..=6).map(|c| carac(c).colonne()).collect();
        assert_eq!(colonnes, vec![1, 2, 5, 8, 13, 17]);
    }

    #[test]
    fn bornes_des_valeurs() {
        assert!(Caracteristique::new(0).is_err());
        assert!(Caracteristique::new(7).is_err());
        assert!(NiveauTalent::new(4).is_err());
    }

    // ---- exemples du livre (une assertion par exemple) -------------------

    #[test]
    fn livre_escalier_agilite1_facile() {
        // Agilité 1, facile (+2) → 24 ; jet 146 : réussi, 6 mètres gagnés.
        let action = Action::new(carac(1), NiveauTalent::FACILE);
        let r = evaluer(&action, jet(146));
        assert_eq!(r.seuil, Seuil::Valeur(24));
        assert!(r.reussi);
        assert_eq!(r.unites, 6);
    }

    #[test]
    fn livre_mfm_force4_facile_reussite_certaine() {
        // Force 4, facile → 66 ; jet 244 : réussi, 4 morceaux.
        let action = Action::new(carac(4), NiveauTalent::FACILE);
        let r = evaluer(&action, jet(244));
        assert_eq!(r.seuil, Seuil::Valeur(66));
        assert!(r.reussi);
        assert_eq!(r.unites, 4);
    }

    #[test]
    fn livre_gustave_fusil_pompe_rate() {
        // Perception 2, Arme d'épaule+2 → 43 ; jet 545 : raté de 5 mètres.
        let action = Action::new(carac(2), niveau(2));
        let r = evaluer(&action, jet(545));
        assert_eq!(r.seuil, Seuil::Valeur(43));
        assert!(!r.reussi);
        assert_eq!(r.unites, 5);
    }

    #[test]
    fn livre_paulo_colt_rate() {
        // Perception 2, Arme de poing+1 → 32 ; jet 352 : raté de 2 mètres.
        let action = Action::new(carac(2), niveau(1));
        let r = evaluer(&action, jet(352));
        assert_eq!(r.seuil, Seuil::Valeur(32));
        assert!(!r.reussi);
        assert_eq!(r.unites, 2);
    }

    #[test]
    fn livre_piko_baratin_reussite_certaine() {
        // Perception 3, Baratin+3 → 66 ; jet 342 : réussi, unités 2.
        let action = Action::new(carac(3), niveau(3));
        let r = evaluer(&action, jet(342));
        assert_eq!(r.seuil, Seuil::Valeur(66));
        assert!(r.reussi);
        assert_eq!(r.unites, 2);
    }

    #[test]
    fn livre_mister_x_feu_rate() {
        // Précision 3, Feu+2 → 62 ; jet 641 : raté (64 > 62), 1 mètre carré.
        let action = Action::new(carac(3), niveau(2));
        let r = evaluer(&action, jet(641));
        assert_eq!(r.seuil, Seuil::Valeur(62));
        assert!(!r.reussi);
        assert_eq!(r.unites, 1);
    }

    #[test]
    fn livre_boka_lance_flammes_malus_de_une_colonne() {
        // Perception 4, Lance-flamme+2, malus -2 +1 = -1 → 64.
        let action = Action::new(carac(4), niveau(2)).avec_modificateur(-2).avec_modificateur(1);
        assert_eq!(seuil_de(&action), Seuil::Valeur(64));
    }

    #[test]
    fn livre_boka_malus_volontaire_de_quatre_colonnes() {
        // Même situation, malus volontaire de 4 en plus du -1 du MJ → 46 ;
        // jet 345 : réussi, unités 5 + 4 = 9.
        let action = Action::new(carac(4), niveau(2))
            .avec_modificateur(-1)
            .avec_malus_volontaire(4);
        let r = evaluer(&action, jet(345));
        assert_eq!(r.seuil, Seuil::Valeur(46));
        assert!(r.reussi);
        assert_eq!(r.unites, 9);
    }

    #[test]
    fn livre_albert_fusil_a_lunette() {
        // Perception 3, Arme d'épaule+1, +3 colonnes (lunette, visée) → 54.
        let base = Action::new(carac(3), niveau(1));
        assert_eq!(seuil_de(&base), Seuil::Valeur(43));
        let vise = base.avec_modificateur(3);
        assert_eq!(seuil_de(&vise), Seuil::Valeur(54));

        // Il s'accorde en plus 3 colonnes de malus → 43 ; jet 435 : réussi
        // de justesse, unités 5 + 3 = 8.
        let r = evaluer(&vise.avec_malus_volontaire(3), jet(435));
        assert_eq!(r.seuil, Seuil::Valeur(43));
        assert!(r.reussi);
        assert_eq!(r.unites, 8);
    }

    // ---- erratum : porte enfoncée par BOKA --------------------------------

    #[test]
    fn erratum_boka_force5_moyen_vaut_66_selon_la_table() {
        // Le texte du livre annonce 54 (jet 265, porte volant de 5 mètres), mais
        // la table donne 66 pour la colonne 5 à +1 ; 54 est la valeur de la
        // colonne 4. Le moteur suit la table, cohérente avec tous les autres
        // exemples. Voir maison/errata.md. Dans les deux cas, 265 réussit.
        let action = Action::new(carac(5), NiveauTalent::MOYEN);
        assert_eq!(seuil_de(&action), Seuil::Valeur(66));
        let r = evaluer(&action, jet(265));
        assert!(r.reussi);
        assert_eq!(r.unites, 5);
    }

    // ---- règles de bord ----------------------------------------------------

    #[test]
    fn sans_talent_utilise_la_ligne_0_avec_malus() {
        // Perception 3 → colonne 5 ; sans talent avec malus 2 → colonne 3, ligne 0.
        let action = Action::sans_talent(carac(3), 2);
        assert_eq!(action.niveau.valeur(), 0);
        assert_eq!(seuil_de(&action), Seuil::Valeur(22));
    }

    #[test]
    fn depasser_la_colonne_gauche_est_un_echec_automatique() {
        let action = Action::new(carac(1), niveau(3)).avec_modificateur(-2);
        assert_eq!(seuil_de(&action), Seuil::EchecAutomatique);
        let r = evaluer(&action, jet(111));
        assert!(!r.reussi, "même un 11 échoue");
    }

    #[test]
    fn la_colonne_gauche_elle_meme_reste_jouable() {
        let action = Action::new(carac(1), niveau(0)).avec_modificateur(-1);
        assert_eq!(action.colonne(), 0);
        assert_eq!(seuil_de(&action), Seuil::Valeur(11));
        assert!(evaluer(&action, jet(111)).reussi);
        assert!(!evaluer(&action, jet(121)).reussi);
    }

    #[test]
    fn depasser_la_colonne_droite_est_une_reussite_automatique() {
        let action = Action::new(carac(6), niveau(0)).avec_modificateur(1);
        assert_eq!(seuil_de(&action), Seuil::ReussiteAutomatique);
        assert!(evaluer(&action, jet(666)).reussi);
    }

    #[test]
    fn le_seuil_est_inclusif() {
        let action = Action::new(carac(3), niveau(1)); // 43
        assert!(evaluer(&action, jet(431)).reussi);
        assert!(!evaluer(&action, jet(441)).reussi);
    }

    #[test]
    fn resoudre_tire_les_des_dans_l_ordre_xyz() {
        use crate::des::SequenceDes;
        let action = Action::new(carac(2), niveau(2)); // 43
        let r = resoudre(&action, &mut SequenceDes::new([1, 2, 3]));
        assert_eq!(r.jet.valeur(), 123);
        assert!(r.reussi);
        assert_eq!(r.unites, 3);
    }

    #[test]
    fn echec_automatique_conserve_les_unites_pour_l_interpretation() {
        let action = Action::new(carac(1), niveau(0)).avec_modificateur(-5);
        let r = evaluer(&action, jet(114));
        assert!(!r.reussi);
        assert_eq!(r.unites, 4);
    }
}
