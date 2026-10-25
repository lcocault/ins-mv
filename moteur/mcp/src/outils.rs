//! Logique des outils, indépendante du protocole MCP.

use insmv_regles::{
    evaluer, Action, Caracteristique, ErreurRegle, Issue, Notation, NiveauTalent, ResultatJet,
    ResultatLancer, SourceDes, Vainqueur, D666,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Description d'une action à résoudre sur la table UM.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ActionParams {
    /// Valeur de la caractéristique utilisée, de 1 à 6.
    pub caracteristique: u8,
    /// Niveau du talent ou du pouvoir (0 à 3), ou difficulté d'une simple
    /// caractéristique (0 difficile, 1 moyen, 2 facile). Défaut : 0.
    #[serde(default)]
    pub niveau: u8,
    /// Décalage de colonnes accordé par le MJ : positif = bonus, négatif =
    /// malus. Défaut : 0.
    #[serde(default)]
    pub modificateur: i32,
    /// Malus volontaire du joueur, en colonnes (≥ 0). Il s'ajoute aux
    /// unités du jet. Défaut : 0.
    #[serde(default)]
    pub malus_volontaire: u8,
    /// Si le personnage ne possède pas le talent : le malus en colonnes
    /// indiqué entre parenthèses après le nom du talent. Impose la ligne 0 ;
    /// `niveau` est alors ignoré.
    #[serde(default)]
    pub malus_sans_talent: Option<u8>,
}

impl ActionParams {
    pub fn vers_action(&self) -> Result<Action, ErreurRegle> {
        let carac = Caracteristique::new(self.caracteristique)?;
        let base = match self.malus_sans_talent {
            Some(malus) => Action::sans_talent(carac, malus),
            None => Action::new(carac, NiveauTalent::new(self.niveau)?),
        };
        Ok(base
            .avec_modificateur(self.modificateur)
            .avec_malus_volontaire(self.malus_volontaire))
    }
}

#[derive(Debug, Serialize)]
pub struct SortieLancer {
    pub notation: String,
    #[serde(flatten)]
    pub resultat: ResultatLancer,
}

pub fn lancer_des(notation: &str, source: &mut impl SourceDes) -> Result<SortieLancer, String> {
    let analysee = Notation::analyser(notation).map_err(|e| e.to_string())?;
    Ok(SortieLancer { notation: notation.trim().to_string(), resultat: analysee.lancer(source) })
}

#[derive(Debug, Serialize)]
pub struct SortieAction {
    #[serde(flatten)]
    pub jet: ResultatJet,
    pub resume: String,
}

pub fn resoudre_action(
    params: &ActionParams,
    source: &mut impl SourceDes,
) -> Result<SortieAction, String> {
    let action = params.vers_action().map_err(|e| e.to_string())?;
    Ok(decrire_action(evaluer(&action, D666::tirer(source))))
}

pub fn decrire_action(jet: ResultatJet) -> SortieAction {
    let verdict = if jet.reussi { "RÉUSSI" } else { "RATÉ" };
    let resume = format!(
        "D666 = {} (xy = {}, z = {}) contre {} : {verdict}, unités = {}",
        jet.jet.valeur(),
        jet.jet.xy(),
        jet.jet.z,
        decrire_seuil(&jet),
        jet.unites,
    );
    SortieAction { jet, resume }
}

fn decrire_seuil(jet: &ResultatJet) -> String {
    use insmv_regles::Seuil::*;
    match jet.seuil {
        Valeur(v) => format!("{v} (colonne {})", jet.colonne),
        EchecAutomatique => format!("échec automatique (colonne {} hors table)", jet.colonne),
        ReussiteAutomatique => format!("réussite automatique (colonne {} hors table)", jet.colonne),
    }
}

#[derive(Debug, Serialize)]
pub struct SortieConflit {
    pub attaquant: SortieAction,
    pub defenseur: SortieAction,
    #[serde(flatten)]
    pub issue: Issue,
    pub resume: String,
}

pub fn resoudre_conflit(
    attaquant: &ActionParams,
    defenseur: &ActionParams,
    source: &mut impl SourceDes,
) -> Result<SortieConflit, String> {
    // L'attaquant lance avant le défenseur : l'ordre est stable et documenté.
    let a = resoudre_action(attaquant, source)?;
    let d = resoudre_action(defenseur, source)?;
    let issue = insmv_regles::resoudre_conflit(&a.jet, &d.jet);
    let resume = decrire_issue(&issue);
    Ok(SortieConflit { attaquant: a, defenseur: d, issue, resume })
}

pub fn decrire_issue(issue: &Issue) -> String {
    match *issue {
        Issue::RienNeSePasse => "L'attaquant rate : rien ne se passe.".to_string(),
        Issue::DefenseSansEffet { unites } => {
            format!("L'attaquant réussit, la défense échoue : action réussie, unités = {unites}.")
        }
        Issue::ConflitReel { vainqueur: Vainqueur::Attaquant, unites_finales } => {
            format!("Conflit réel : l'attaquant l'emporte, unités finales = {unites_finales}.")
        }
        Issue::ConflitReel { vainqueur: Vainqueur::Defenseur, unites_finales } => {
            format!("Conflit réel : le défenseur l'emporte (action ratée), écart = {unites_finales}.")
        }
        Issue::ConflitReel { vainqueur: Vainqueur::ArbitrageMj, .. } => {
            "Conflit réel à unités égales : le maître de jeu doit trancher.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insmv_regles::SequenceDes;

    fn params(carac: u8, niveau: u8) -> ActionParams {
        ActionParams {
            caracteristique: carac,
            niveau,
            modificateur: 0,
            malus_volontaire: 0,
            malus_sans_talent: None,
        }
    }

    #[test]
    fn lancer_des_decrit_le_tirage() {
        let s = lancer_des(" 2D6+3 ", &mut SequenceDes::new([2, 5])).unwrap();
        assert_eq!(s.notation, "2D6+3");
        assert_eq!(s.resultat.total, 10);
        assert!(lancer_des("D8", &mut SequenceDes::new([])).is_err());
    }

    #[test]
    fn resoudre_action_exemple_du_livre() {
        // Perception 2, Arme d'épaule+2 : seuil 43 ; jet 545 : raté de 5.
        let s = resoudre_action(&params(2, 2), &mut SequenceDes::new([5, 4, 5])).unwrap();
        assert!(!s.jet.reussi);
        assert_eq!(s.jet.unites, 5);
        assert_eq!(
            s.resume,
            "D666 = 545 (xy = 54, z = 5) contre 43 (colonne 2) : RATÉ, unités = 5"
        );
    }

    #[test]
    fn sans_talent_ignore_le_niveau() {
        let mut p = params(3, 3);
        p.malus_sans_talent = Some(2);
        let action = p.vers_action().unwrap();
        assert_eq!(action.niveau.valeur(), 0);
        assert_eq!(action.modificateur, -2);
    }

    #[test]
    fn parametres_hors_bornes_sont_refuses_avec_un_message() {
        let err = resoudre_action(&params(7, 0), &mut SequenceDes::new([])).unwrap_err();
        assert!(err.contains("caractéristique"), "{err}");
        let err = resoudre_action(&params(3, 4), &mut SequenceDes::new([])).unwrap_err();
        assert!(err.contains("niveau"), "{err}");
    }

    #[test]
    fn conflit_attaquant_tire_en_premier() {
        // Gustave (AG 1 +2 → 24) : 163 → 3 ; médecin (AG 1 +2 → 24) : 216 → 6.
        let s = resoudre_conflit(
            &params(1, 2),
            &params(1, 2),
            &mut SequenceDes::new([1, 6, 3, 2, 1, 6]),
        )
        .unwrap();
        assert_eq!(s.attaquant.jet.jet.valeur(), 163);
        assert_eq!(s.defenseur.jet.jet.valeur(), 216);
        assert_eq!(
            s.issue,
            Issue::ConflitReel { vainqueur: Vainqueur::Defenseur, unites_finales: 3 }
        );
    }

    #[test]
    fn conflit_attaquant_rate() {
        let s = resoudre_conflit(
            &params(1, 0),
            &params(6, 3),
            &mut SequenceDes::new([6, 6, 6, 1, 1, 1]),
        )
        .unwrap();
        assert_eq!(s.issue, Issue::RienNeSePasse);
    }

    #[test]
    fn sortie_serialisable_en_json() {
        let s = resoudre_action(&params(3, 1), &mut SequenceDes::new([4, 3, 5])).unwrap();
        let json = serde_json::to_value(&s).unwrap();
        assert_eq!(json["reussi"], true);
        assert_eq!(json["unites"], 5);
        assert_eq!(json["seuil"]["type"], "valeur");
    }
}
