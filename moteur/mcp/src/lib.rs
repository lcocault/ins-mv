//! Serveur MCP du maître de jeu. La logique des outils vit dans [`outils`]
//! (pure, testée avec des dés prédéterminés) ; [`serveur`] n'est qu'un
//! adaptateur vers le protocole MCP.

pub mod jeu;
pub mod outils;
pub mod serveur;
