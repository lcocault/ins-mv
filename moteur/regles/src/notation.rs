//! Notation des jets (`1D6`, `2D6+3`, `D66`, `D666`) — `regles/commun-02-des.md`.

use serde::Serialize;

use crate::des::{d66, nd6, SourceDes, D666};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notation {
    /// `nD6+b`.
    NombreDe6 { nombre: u8, bonus: i32 },
    D66,
    D666,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("notation de dés invalide : « {0} » (attendu : D6, 2D6+3, D66 ou D666)")]
pub struct ErreurNotation(pub String);

/// Au-delà, on soupçonne une faute de frappe plutôt qu'un vrai jet.
const MAX_DES: u8 = 50;

impl Notation {
    pub fn analyser(texte: &str) -> Result<Self, ErreurNotation> {
        let erreur = || ErreurNotation(texte.to_string());
        let t: String = texte.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_uppercase();
        match t.as_str() {
            "D66" => return Ok(Self::D66),
            "D666" => return Ok(Self::D666),
            _ => {}
        }
        let (nombre, reste) = t.split_once('D').ok_or_else(erreur)?;
        let nombre: u8 = if nombre.is_empty() { 1 } else { nombre.parse().map_err(|_| erreur())? };
        let apres = reste.strip_prefix('6').ok_or_else(erreur)?;
        let bonus: i32 = if apres.is_empty() {
            0
        } else if apres.starts_with('+') || apres.starts_with('-') {
            apres.parse().map_err(|_| erreur())?
        } else {
            return Err(erreur());
        };
        if nombre == 0 || nombre > MAX_DES {
            return Err(erreur());
        }
        Ok(Self::NombreDe6 { nombre, bonus })
    }

    pub fn lancer(&self, source: &mut impl SourceDes) -> ResultatLancer {
        match *self {
            Self::NombreDe6 { nombre, bonus } => {
                let (faces, total) = nd6(source, nombre, bonus);
                ResultatLancer { faces, total }
            }
            Self::D66 => {
                let valeur = d66(source);
                ResultatLancer { faces: vec![valeur / 10, valeur % 10], total: i32::from(valeur) }
            }
            Self::D666 => {
                let jet = D666::tirer(source);
                ResultatLancer { faces: vec![jet.x, jet.y, jet.z], total: i32::from(jet.valeur()) }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResultatLancer {
    pub faces: Vec<u8>,
    pub total: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::des::SequenceDes;

    #[test]
    fn analyse_les_notations_du_livre() {
        assert_eq!(Notation::analyser("1D6"), Ok(Notation::NombreDe6 { nombre: 1, bonus: 0 }));
        assert_eq!(Notation::analyser("d6"), Ok(Notation::NombreDe6 { nombre: 1, bonus: 0 }));
        assert_eq!(Notation::analyser("2D6"), Ok(Notation::NombreDe6 { nombre: 2, bonus: 0 }));
        assert_eq!(Notation::analyser("1D6+3"), Ok(Notation::NombreDe6 { nombre: 1, bonus: 3 }));
        assert_eq!(Notation::analyser(" 2d6 - 1 "), Ok(Notation::NombreDe6 { nombre: 2, bonus: -1 }));
        assert_eq!(Notation::analyser("D66"), Ok(Notation::D66));
        assert_eq!(Notation::analyser("d666"), Ok(Notation::D666));
    }

    #[test]
    fn refuse_les_notations_invalides() {
        for t in ["", "6", "D", "D8", "0D6", "99D6", "2D6+", "2D6*3", "D6+x", "abc", "D666+1"] {
            assert!(Notation::analyser(t).is_err(), "{t:?} devrait être refusé");
        }
    }

    #[test]
    fn lancers() {
        let r = Notation::analyser("1D6+3").unwrap().lancer(&mut SequenceDes::new([4]));
        assert_eq!((r.faces, r.total), (vec![4], 7));
        let r = Notation::D66.lancer(&mut SequenceDes::new([3, 4]));
        assert_eq!((r.faces, r.total), (vec![3, 4], 34));
        let r = Notation::D666.lancer(&mut SequenceDes::new([4, 6, 2]));
        assert_eq!((r.faces, r.total), (vec![4, 6, 2], 462));
    }
}
