# Moteur de jeu In Nomine Satanis / Magna Veritas

Workspace Rust (toolchain épinglé dans `rust-toolchain.toml`).

| Crate | Rôle |
|---|---|
| `regles/` | Règles pures : dés, table UM, conflits, fiches et budgets de création, blessures par espèce, combat (attaque, dommages, initiative), catalogues de pouvoirs, talents, armes et protections. Aucune E/S. |
| `campagne/` | Persistance SQLite : personnages et journal de campagne. |
| `mcp/` | Serveur MCP (stdio) exposé au maître de jeu. La logique est dans `outils.rs` et `jeu.rs`, `serveur.rs` n'est qu'un adaptateur. |

## Commandes

```bash
cargo test                                     # toute la suite
python3 -I outils/generer_catalogues.py        # régénère regles/donnees/*.json (pouvoirs, talents, armes, protections) depuis livres/
```

## Données

- La campagne vit dans `campagne/jeu.sqlite` à la racine du dépôt (ignorée par git).
  `INSMV_CAMPAGNE=/chemin/jeu.sqlite` la déplace.
- `regles/donnees/{pouvoirs,talents,armes,protections}.json` sont générés depuis `livres/` ;
  ne pas les éditer à la main.
- Les écarts du livre sont dans `../maison/errata.md`, les interprétations du moteur dans
  `../maison/decisions.md` ; chacun est figé par un test.

## Interface Web (lecture seule)

```bash
scripts/web.sh start            # compile si besoin, démarre en arrière-plan : http://localhost:8787
scripts/web.sh stop | restart | status | logs
```

Depuis Windows, double-cliquer sur `scripts/web-start.cmd`, `web-stop.cmd` ou `web-status.cmd`
(ils appellent le script dans WSL). `INSMV_WEB_PORT` change le port ; le PID et le journal
sont dans `.run/` (ignoré par git).

Liste des personnages, fiche (PP, dommages, caractéristiques, talents et pouvoirs avec seuil
et chances de réussite sur la table UM) et journal filtrable. Elle lit la base en lecture seule :
les changements faits par le serveur MCP apparaissent à l'actualisation (case « Actualiser
toutes les 5 s »). Elle n'écoute que sur 127.0.0.1 : aucune authentification.

## Serveur MCP

`.mcp.json` lance `scripts/mcp.sh`, qui recompile seulement si les sources ont changé puis exécute le
binaire (démarrage rapide, les clients MCP abandonnent après ~30 s). Après une modification du code,
le premier lancement recompile : relancer la session si la connexion a expiré.
