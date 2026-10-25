//! Les dés : D6, D66 et D666 (`livres/commun-02-des.md`).

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::ErreurRegle;

/// Source de dés à six faces. Injectée pour rendre les tirages testables.
pub trait SourceDes {
    /// Tire un dé à six faces (1 à 6).
    fn d6(&mut self) -> u8;
}

/// Dés réellement aléatoires.
pub struct DesAleatoires<R: Rng = rand::rngs::ThreadRng>(pub R);

impl Default for DesAleatoires {
    fn default() -> Self {
        Self(rand::thread_rng())
    }
}

impl<R: Rng> SourceDes for DesAleatoires<R> {
    fn d6(&mut self) -> u8 {
        self.0.gen_range(1..=6)
    }
}

/// Dés prédéterminés, pour les tests. Épuisés, ils provoquent une panique.
pub struct SequenceDes {
    faces: std::vec::IntoIter<u8>,
}

impl SequenceDes {
    pub fn new(faces: impl IntoIterator<Item = u8>) -> Self {
        Self { faces: faces.into_iter().collect::<Vec<_>>().into_iter() }
    }
}

impl SourceDes for SequenceDes {
    fn d6(&mut self) -> u8 {
        self.faces.next().expect("séquence de dés épuisée")
    }
}

/// Résultat d'un D666 : x (centaines), y (dizaines), z (unités).
///
/// Sur la table UM, `xy` se lit comme un D66 comparé au seuil de réussite et
/// `z` mesure la qualité de la réussite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct D666 {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl D666 {
    pub fn new(x: u8, y: u8, z: u8) -> Result<Self, ErreurRegle> {
        for face in [x, y, z] {
            if !(1..=6).contains(&face) {
                return Err(ErreurRegle::Face(face));
            }
        }
        Ok(Self { x, y, z })
    }

    /// Lit un résultat écrit sous la forme `346`.
    pub fn depuis_valeur(valeur: u16) -> Result<Self, ErreurRegle> {
        Self::new((valeur / 100 % 10) as u8, (valeur / 10 % 10) as u8, (valeur % 10) as u8)
    }

    /// Lance trois dés dans l'ordre x, y, z.
    pub fn tirer(source: &mut impl SourceDes) -> Self {
        let (x, y, z) = (source.d6(), source.d6(), source.d6());
        Self { x, y, z }
    }

    /// Le D66 formé par x et y (ex. 346 → 34).
    pub fn xy(&self) -> u8 {
        self.x * 10 + self.y
    }

    /// La valeur à trois chiffres (ex. 346).
    pub fn valeur(&self) -> u16 {
        u16::from(self.x) * 100 + u16::from(self.y) * 10 + u16::from(self.z)
    }
}

/// Lance un D66 (deux dés lus comme dizaines et unités).
pub fn d66(source: &mut impl SourceDes) -> u8 {
    let (dizaines, unites) = (source.d6(), source.d6());
    dizaines * 10 + unites
}

/// Lance `nombre` dés à six faces et ajoute `bonus` (notation `nD6+b`).
pub fn nd6(source: &mut impl SourceDes, nombre: u8, bonus: i32) -> (Vec<u8>, i32) {
    let faces: Vec<u8> = (0..nombre).map(|_| source.d6()).collect();
    let total = faces.iter().map(|&f| i32::from(f)).sum::<i32>() + bonus;
    (faces, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d666_lit_xyz_dans_l_ordre_du_tirage() {
        let jet = D666::tirer(&mut SequenceDes::new([4, 6, 2]));
        assert_eq!(jet, D666 { x: 4, y: 6, z: 2 });
        assert_eq!(jet.valeur(), 462);
        assert_eq!(jet.xy(), 46);
    }

    #[test]
    fn d666_exemple_du_livre_escalier() {
        // « Résultat 346 : 34 pour la chute, 6 pour le nombre de marches ratées. »
        let jet = D666::depuis_valeur(346).unwrap();
        assert_eq!((jet.xy(), jet.z), (34, 6));
    }

    #[test]
    fn d666_refuse_les_faces_invalides() {
        assert_eq!(D666::new(0, 3, 3), Err(ErreurRegle::Face(0)));
        assert_eq!(D666::new(1, 7, 3), Err(ErreurRegle::Face(7)));
        assert!(D666::depuis_valeur(160).is_err());
        assert!(D666::depuis_valeur(1000).is_err());
    }

    #[test]
    fn d66_exemple_du_livre() {
        // « Résultat des dés : 3 et 4. Le résultat est 34. »
        assert_eq!(d66(&mut SequenceDes::new([3, 4])), 34);
    }

    #[test]
    fn nd6_additionne_et_applique_le_bonus() {
        let (faces, total) = nd6(&mut SequenceDes::new([2, 5]), 2, 3);
        assert_eq!(faces, vec![2, 5]);
        assert_eq!(total, 10);
    }

    #[test]
    fn des_aleatoires_restent_dans_les_bornes() {
        let mut des = DesAleatoires::default();
        for _ in 0..2000 {
            let jet = D666::tirer(&mut des);
            assert!(D666::new(jet.x, jet.y, jet.z).is_ok());
        }
    }
}
