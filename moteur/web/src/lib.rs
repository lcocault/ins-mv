//! Interface Web du joueur. Elle lit la base de campagne que le serveur MCP écrit (un PP dépensé
//! ou une blessure y apparaît à l'actualisation) et n'y écrit qu'un dé lancé par le joueur.

pub mod rendu;
pub mod serveur;
pub mod vue;

pub use serveur::{routeur, Etat};
