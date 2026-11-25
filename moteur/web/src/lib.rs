//! Interface Web pour discuter avec le maître de jeu et consulter la campagne.
//! Le serveur MCP écrit la base de campagne, que l'interface relit à l'actualisation.

pub mod rendu;
pub mod serveur;

pub use serveur::{routeur, Etat};
