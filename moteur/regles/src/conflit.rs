//! Résolution des conflits (`regles/commun-05-table-um.md`).
//!
//! Lorsqu'un défenseur s'oppose à l'action d'un attaquant, les deux jets sont
//! confrontés. S'il y a conflit réel, le plus grand nombre d'unités l'emporte.

use serde::{Deserialize, Serialize};

use crate::table_um::ResultatJet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vainqueur {
    Attaquant,
    Defenseur,
    /// Unités identiques : le maître de jeu tranche, le moteur ne décide pas.
    ArbitrageMj,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "issue", rename_all = "snake_case")]
pub enum Issue {
    /// L'attaquant a raté : rien ne se passe, quel que soit le jet adverse.
    RienNeSePasse,
    /// L'attaquant réussit, le défenseur rate : la défense est sans effet.
    DefenseSansEffet { unites: u8 },
    /// Les deux réussissent. `unites_finales` est l'écart des unités ; il
    /// qualifie la réussite du vainqueur (dommages, mètres gagnés, etc.).
    ConflitReel { vainqueur: Vainqueur, unites_finales: u8 },
}

impl Issue {
    /// L'action de l'attaquant aboutit-elle ? `None` si le MJ doit trancher.
    pub fn action_reussie(&self) -> Option<bool> {
        match self {
            Issue::RienNeSePasse => Some(false),
            Issue::DefenseSansEffet { .. } => Some(true),
            Issue::ConflitReel { vainqueur: Vainqueur::Attaquant, .. } => Some(true),
            Issue::ConflitReel { vainqueur: Vainqueur::Defenseur, .. } => Some(false),
            Issue::ConflitReel { vainqueur: Vainqueur::ArbitrageMj, .. } => None,
        }
    }
}

pub fn resoudre_conflit(attaquant: &ResultatJet, defenseur: &ResultatJet) -> Issue {
    match (attaquant.reussi, defenseur.reussi) {
        (false, _) => Issue::RienNeSePasse,
        (true, false) => Issue::DefenseSansEffet { unites: attaquant.unites },
        (true, true) => {
            let (a, d) = (attaquant.unites, defenseur.unites);
            let vainqueur = match a.cmp(&d) {
                std::cmp::Ordering::Greater => Vainqueur::Attaquant,
                std::cmp::Ordering::Less => Vainqueur::Defenseur,
                std::cmp::Ordering::Equal => Vainqueur::ArbitrageMj,
            };
            Issue::ConflitReel { vainqueur, unites_finales: a.abs_diff(d) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::des::D666;
    use crate::table_um::{evaluer, Action, Caracteristique, NiveauTalent};

    fn jet_reussi(unites: u8) -> ResultatJet {
        let action = Action::new(Caracteristique::new(6).unwrap(), NiveauTalent::FACILE);
        let mut r = evaluer(&action, D666::new(1, 1, 1).unwrap());
        r.unites = unites;
        r
    }
    fn jet_rate(unites: u8) -> ResultatJet {
        ResultatJet { reussi: false, ..jet_reussi(unites) }
    }

    #[test]
    fn les_quatre_cas_de_la_table_des_conflits() {
        assert_eq!(resoudre_conflit(&jet_rate(5), &jet_rate(5)), Issue::RienNeSePasse);
        assert_eq!(resoudre_conflit(&jet_rate(5), &jet_reussi(1)), Issue::RienNeSePasse);
        assert_eq!(
            resoudre_conflit(&jet_reussi(4), &jet_rate(6)),
            Issue::DefenseSansEffet { unites: 4 }
        );
        assert!(matches!(
            resoudre_conflit(&jet_reussi(4), &jet_reussi(2)),
            Issue::ConflitReel { .. }
        ));
    }

    #[test]
    fn livre_gustave_contre_le_medecin() {
        // Gustave touche (163), le médecin esquive (216) : 3 contre 6, le
        // défenseur gagne ; l'écart des unités est 3.
        let issue = resoudre_conflit(&jet_reussi(3), &jet_reussi(6));
        assert_eq!(issue, Issue::ConflitReel { vainqueur: Vainqueur::Defenseur, unites_finales: 3 });
        assert_eq!(issue.action_reussie(), Some(false));
    }

    #[test]
    fn livre_paulo_contre_le_familier() {
        // 125 contre 343 : 5 contre 3, l'attaquant gagne avec un écart de 2.
        let issue = resoudre_conflit(&jet_reussi(5), &jet_reussi(3));
        assert_eq!(issue, Issue::ConflitReel { vainqueur: Vainqueur::Attaquant, unites_finales: 2 });
        assert_eq!(issue.action_reussie(), Some(true));
    }

    #[test]
    fn egalite_renvoyee_au_maitre_de_jeu() {
        let issue = resoudre_conflit(&jet_reussi(4), &jet_reussi(4));
        assert_eq!(issue, Issue::ConflitReel { vainqueur: Vainqueur::ArbitrageMj, unites_finales: 0 });
        assert_eq!(issue.action_reussie(), None);
    }

    #[test]
    fn le_malus_volontaire_compte_dans_la_comparaison() {
        // Les unités comparées sont les unités finales (z + malus volontaire).
        let c = Caracteristique::new(6).unwrap();
        let action = Action::new(c, NiveauTalent::FACILE).avec_malus_volontaire(3);
        let attaque = evaluer(&action, D666::new(1, 1, 2).unwrap()); // unités 5
        let defense = jet_reussi(4);
        assert_eq!(
            resoudre_conflit(&attaque, &defense),
            Issue::ConflitReel { vainqueur: Vainqueur::Attaquant, unites_finales: 1 }
        );
    }
}
