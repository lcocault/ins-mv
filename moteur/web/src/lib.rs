//! Interface Web en lecture seule. Elle lit la base de campagne que le serveur
//! MCP écrit : un PP dépensé ou une blessure y apparaît à l'actualisation.

pub mod rendu;
pub mod serveur;

pub use serveur::{routeur, Etat};
