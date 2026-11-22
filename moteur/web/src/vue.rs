//! Ce que le joueur a le droit de voir : son équipe en clair, le reste seulement par ce qu'il a appris.

use insmv_campagne::Connaissance;
use insmv_regles::personnage::{Personnage, Role};

/// La campagne telle que le joueur la voit.
pub struct Vue {
    /// Le personnage du joueur et ses partenaires : fiches complètes.
    pub equipe: Vec<Personnage>,
    /// Tous les autres : jamais affichés tels quels.
    pub autres: Vec<Personnage>,
    pub connaissances: Vec<Connaissance>,
}

impl Vue {
    pub fn nouvelle(personnages: Vec<Personnage>, connaissances: Vec<Connaissance>) -> Self {
        let (equipe, autres) = personnages.into_iter().partition(|p| matches!(p.role, Role::Joueur | Role::Partenaire));
        Self { equipe, autres, connaissances }
    }

    pub fn est_equipe(&self, id: &str) -> bool {
        self.equipe.iter().any(|p| p.id == id)
    }

    /// Nom sous lequel le joueur connaît un personnage hors équipe : le dernier nom donné.
    pub fn nom_connu(&self, id: &str) -> Option<String> {
        self.connaissances
            .iter()
            .rev()
            .filter(|k| k.genre == "personnage" && k.sujet == id)
            .map(|k| k.nom_connu.clone())
            .find(|n| !n.is_empty())
    }

    /// Un personnage hors équipe n'existe pour le joueur qu'une fois révélé.
    pub fn est_rencontre(&self, id: &str) -> bool {
        self.connaissances.iter().any(|k| k.genre == "personnage" && k.sujet == id)
    }

    /// Nom à afficher pour n'importe quel personnage connu du joueur (équipe ou rencontré).
    pub fn nom_affiche(&self, id: &str) -> Option<String> {
        if let Some(p) = self.equipe.iter().find(|p| p.id == id) {
            return Some(p.nom.clone());
        }
        self.est_rencontre(id).then(|| self.nom_connu(id).unwrap_or_else(|| "Inconnu".into()))
    }

    pub fn masque(&self) -> Masque {
        let mut remplacements: Vec<(String, String)> = self
            .autres
            .iter()
            .map(|p| (p.nom.clone(), self.nom_connu(&p.id).unwrap_or_else(|| "un inconnu".into())))
            .collect();
        // Les noms longs d'abord : « Goule B » avant « Goule ».
        remplacements.sort_by_key(|(nom, _)| std::cmp::Reverse(nom.chars().count()));
        Masque { remplacements, equipe: self.equipe.iter().map(|p| p.nom.clone()).collect() }
    }
}

/// Remplace les vrais noms des adversaires par ceux que le joueur connaît.
#[derive(Debug, Default, Clone)]
pub struct Masque {
    remplacements: Vec<(String, String)>,
    equipe: Vec<String>,
}

impl Masque {
    pub fn texte(&self, texte: &str) -> String {
        self.remplacements.iter().fold(texte.to_string(), |t, (vrai, connu)| t.replace(vrai, connu))
    }

    pub fn est_equipe(&self, nom: &str) -> bool {
        self.equipe.iter().any(|n| n == nom)
    }

    /// « Jean : AG 5 + Arme de contact +2 » reste tel quel ; pour un adversaire, seul son nom
    /// connu est conservé : son niveau de talent ou de pouvoir n'a pas à être affiché.
    pub fn libelle(&self, libelle: &str) -> String {
        match libelle.split_once(" : ") {
            Some((acteur, _)) if !self.est_equipe(acteur) => self.texte(acteur),
            _ => self.texte(libelle),
        }
    }

    /// Vrai si le jet est celui de l'équipe du joueur (ou un jet sans acteur nommé).
    pub fn jet_de_l_equipe(&self, libelle: &str) -> bool {
        libelle.split_once(" : ").is_none_or(|(acteur, _)| self.est_equipe(acteur))
    }

    /// Compte rendu d'une action de combat. Les détails de dommages d'un adversaire (unités, Force
    /// restante, état) ne sont pas montrés : seul le résultat du jet l'est.
    pub fn compte_rendu(&self, resume: &str, cible: Option<&str>) -> String {
        let cible_equipe = cible.is_none_or(|c| self.est_equipe(c));
        let garde = |ligne: &&str| cible_equipe || !(ligne.starts_with("Dommages") || ligne.contains("dommage(s) au total") || ligne.starts_with("Si le MJ"));
        self.texte(&resume.lines().filter(garde).collect::<Vec<_>>().join("\n"))
    }

    pub fn etat_visible(&self, nom: &str) -> bool {
        self.est_equipe(nom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insmv_regles::personnage::{Caracteristiques, Espece, Personnage};

    fn perso(nom: &str, role: Role) -> Personnage {
        let mut p = Personnage {
            id: String::new(),
            nom: nom.into(),
            espece: Espece::Demon,
            role,
            caracteristiques: Caracteristiques { fo: 3, pr: 3, vo: 4, pe: 3, ag: 3, ap: 2 },
            talents: vec![],
            pouvoirs: vec![],
            patron: None,
            pp: None,
            pp_max_scenario: None,
            dommages: 0,
            equipement: vec![],
            notes: String::new(),
        };
        p.finaliser();
        p
    }

    fn connu(sujet: &str, nom: &str) -> Connaissance {
        Connaissance { id: 0, horodatage: 0, genre: "personnage".into(), sujet: sujet.into(), nom_connu: nom.into(), texte: "t".into() }
    }

    #[test]
    fn l_equipe_est_en_clair_les_autres_par_leur_nom_connu() {
        let vue = Vue::nouvelle(
            vec![perso("Jean", Role::Joueur), perso("Billy Diniez", Role::Pnj), perso("Goule B", Role::Pnj)],
            vec![connu("billy-diniez", "le batteur")],
        );
        assert!(vue.est_equipe("jean") && !vue.est_equipe("billy-diniez"));
        assert_eq!(vue.nom_affiche("jean").as_deref(), Some("Jean"));
        assert_eq!(vue.nom_affiche("billy-diniez").as_deref(), Some("le batteur"));
        assert_eq!(vue.nom_affiche("goule-b"), None, "jamais révélée : n'existe pas pour le joueur");
        let m = vue.masque();
        assert_eq!(m.texte("Billy Diniez tire sur Jean. Goule B tombe."), "le batteur tire sur Jean. un inconnu tombe.");
    }

    #[test]
    fn libelle_et_compte_rendu_cachent_les_details_des_adversaires() {
        let vue = Vue::nouvelle(vec![perso("Jean", Role::Joueur), perso("Zombi", Role::Pnj)], vec![]);
        let m = vue.masque();
        assert_eq!(m.libelle("Zombi : Agilité 3 (talent Esquive +1)"), "un inconnu");
        assert_eq!(m.libelle("Jean : Agilité 5 (talent Esquive +1)"), "Jean : Agilité 5 (talent Esquive +1)");
        let resume = "Jean attaque Zombi (contact) : 366 → touche\nDommages : 7 (3 unités)\nZombi : 7 dommage(s) au total, Force -5 (Mort).";
        assert_eq!(m.compte_rendu(resume, Some("Zombi")), "Jean attaque un inconnu (contact) : 366 → touche");
        assert!(m.compte_rendu(resume, Some("Jean")).contains("Dommages : 7"), "une cible de l'équipe garde ses détails");
    }
}
