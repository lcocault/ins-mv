# Moteur de jeu In Nomine Satanis / Magna Veritas

Workspace Rust (toolchain épinglé dans `rust-toolchain.toml`).

| Crate | Rôle |
|---|---|
| `regles/` | Règles pures : dés, table UM, conflits, fiches et budgets de création, blessures par espèce, combat (attaque, dommages, initiative), catalogues de pouvoirs, talents, armes et protections. Aucune E/S. |
| `campagne/` | Persistance SQLite : personnages, journal (avec visibilité pour le joueur), connaissances du joueur, événements de partie. |
| `mcp/` | Serveur MCP (stdio) exposé au maître de jeu. La logique est dans `outils.rs` et `jeu.rs`, `serveur.rs` n'est qu'un adaptateur. |

## Commandes

```bash
cargo test                                     # toute la suite
python3 -I outils/generer_catalogues.py        # régénère regles/donnees/*.json (pouvoirs, talents, armes, protections) depuis regles/
```

## Données

- La campagne vit dans `campagne/jeu.sqlite` à la racine du dépôt (ignorée par git).
  `INSMV_CAMPAGNE=/chemin/jeu.sqlite` la déplace.
- `regles/donnees/{pouvoirs,talents,armes,protections}.json` sont générés depuis `regles/` ;
  ne pas les éditer à la main. `regles/donnees/maison.json` est écrit à la main : armes, talents et pouvoirs
  créés pour le jeu (marqués `maison`), chargés avec les catalogues du livre.
- Les écarts du livre sont dans `../maison/errata.md`, les interprétations du moteur dans
  `../maison/decisions.md` ; chacun est figé par un test.

## Interface Web du joueur

```bash
scripts/web.sh start            # compile si besoin, démarre en arrière-plan : http://localhost:8787
scripts/web.sh stop | restart | status | logs
```

Depuis Windows, double-cliquer sur `scripts/web-start.cmd`, `web-stop.cmd` ou `web-status.cmd`
(ils appellent le script dans WSL). `INSMV_WEB_PORT` change le port ; le PID et le journal
sont dans `.run/` (ignoré par git).

La page lit la base que le serveur MCP écrit (case « Actualiser toutes les 5 s »). Elle n'écoute
que sur 127.0.0.1 : aucune authentification. Elle ne montre au joueur que ce qu'il a le droit de savoir :

- **Personnages** (`/`) : l'équipe (rôles joueur et partenaire) en fiche complète ; les autres personnages
  n'apparaissent qu'une fois révélés par l'outil MCP `reveler`, sous le nom que le joueur leur connaît et avec
  les seules informations apprises. Jamais d'espèce, de PP, de caractéristiques ni de vrai nom.
- **Lieux** (`/lieux`) : ce que le joueur sait de chaque lieu (`reveler` avec le genre `lieu`).
- **Journal** (`/journal`) : seulement les entrées notées avec `visible: true`.
- **Partie** (`/partie`) : table UM avec la ligne et la colonne de l'action. Quand le MJ propose un jet
  (`annoncer_action`), la page affiche quatre boutons « Lancer » (malus volontaire de 0 à 3 colonnes). Le clic lance
  le D666 (`POST /lancer`, seule écriture de la page) ; le MJ résout ensuite l'action avec `de_joueur: true`
  (`tester_personnage`, `conflit_personnages`, `attaquer`, `utiliser_pouvoir`). Les jets récents et le combat en cours
  (ordre d'action, état, fil des actions) masquent les noms et les détails des adversaires non révélés.

## Serveur MCP

`.mcp.json` lance `scripts/mcp.sh`, qui recompile seulement si les sources ont changé puis exécute le
binaire (démarrage rapide, les clients MCP abandonnent après ~30 s). Après une modification du code,
le premier lancement recompile : relancer la session si la connexion a expiré.
