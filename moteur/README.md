# Moteur de jeu In Nomine Satanis / Magna Veritas

Workspace Rust (toolchain épinglé dans `rust-toolchain.toml`).

| Crate | Rôle |
|---|---|
| `regles/` | Règles pures : dés, table UM, conflits, fiches et budgets de création, catalogues de pouvoirs et de talents. Aucune E/S. |
| `campagne/` | Persistance SQLite : personnages et journal de campagne. |
| `mcp/` | Serveur MCP (stdio) exposé au maître de jeu. La logique est dans `outils.rs` et `jeu.rs`, `serveur.rs` n'est qu'un adaptateur. |

## Commandes

```bash
cargo test                                     # toute la suite
python3 -I outils/generer_catalogues.py        # régénère regles/donnees/*.json depuis livres/
```

## Données

- La campagne vit dans `campagne/jeu.sqlite` à la racine du dépôt (ignorée par git).
  `INSMV_CAMPAGNE=/chemin/jeu.sqlite` la déplace.
- `regles/donnees/pouvoirs.json` et `talents.json` sont générés depuis `livres/` ;
  ne pas les éditer à la main.
- Les écarts du livre et les décisions de règles sont dans `../maison/errata.md`,
  chacun figé par un test.
