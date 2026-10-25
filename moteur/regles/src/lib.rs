//! Moteur de règles d'In Nomine Satanis / Magna Veritas (1re édition).
//!
//! Ce crate est pur : aucune entrée/sortie, aucun état global. Le hasard est
//! injecté via [`des::SourceDes`], ce qui rend chaque comportement testable.
//! Les références renvoient à `regles/commun-05-table-um.md`.

pub mod blessures;
pub mod catalogue;
pub mod combat;
pub mod conflit;
pub mod des;
pub mod notation;
pub mod personnage;
pub mod table_um;

pub use conflit::{resoudre_conflit, Issue, Vainqueur};
pub use des::{DesAleatoires, SequenceDes, SourceDes, D666};
pub use notation::{ErreurNotation, Notation, ResultatLancer};
pub use table_um::{evaluer, resoudre, Action, Caracteristique, NiveauTalent, ResultatJet, Seuil};

/// Erreur de construction d'une valeur de règle hors bornes.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErreurRegle {
    #[error("caractéristique hors bornes (1 à 6) : {0}")]
    Caracteristique(u8),
    #[error("niveau de talent hors bornes (0 à 3) : {0}")]
    NiveauTalent(u8),
    #[error("face de dé hors bornes (1 à 6) : {0}")]
    Face(u8),
}
