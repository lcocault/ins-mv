# Errata et décisions de règles

Registre des écarts constatés dans les règles de base (`livres/`, immuables) et
des décisions prises pour le moteur. Chaque entrée indique la règle appliquée
par le moteur et le test qui la fige.

## E-001 — Porte enfoncée par BOKA (Force 5, difficulté moyenne)

- **Source :** `livres/commun-05-table-um.md`, exemple [MV] « Autre exemple »
  de l'utilisation des caractéristiques (MV p.9).
- **Constat :** le texte annonce « Force de 5, difficulté moyenne (+1) → jet
  inférieur ou égal à 54 ». La table UM donne **66** pour la colonne Force 5 à
  la ligne +1 ; 54 est la valeur de la colonne **4**.
- **Analyse :** les six autres exemples chiffrés du livre sont conformes à la
  table, qui est croissante et cohérente. L'erreur est donc dans l'exemple.
  Le jet 265 de l'exemple réussit dans les deux lectures et les unités (5)
  restent inchangées : la narration n'est pas affectée.
- **Décision :** le moteur applique la **table** (66).
- **Test :** `table_um::tests::erratum_boka_force5_moyen_vaut_66_selon_la_table`.
- **Statut :** `livres/` n'est pas modifié, par respect de son statut de
  référence immuable ; cet erratum fait foi pour le moteur.

## E-002 — Humains d'exemple qui dépassent le budget de création

- **Source :** `livres/commun-06-creation-personnages-humains.md`, tableau
  « Quelques êtres humains ».
- **Règle :** un humain adulte a 10 points de caractéristiques (maximum 3) et
  6 points de talents (niveau 0 = 1 point, +1 = 2 points, +2 = 4 points).
- **Constat :** quatre des huit profils d'exemple dépassent ce budget.

  | Profil | Caractéristiques | Talents |
  |---|---|---|
  | Scientifique | 10 | **7** (Scientifique+2, Scientifique, Discussion+1) |
  | Militaire | **11** | **7** (Arme+2, Arme, Tactique+1) |
  | Policier | **11** | 6 |
  | Truand | **11** | 6 |

  Les quatre autres (Normal, Skinhead, Chrétien, Loubard) respectent le budget.
- **Analyse :** soit ces profils sont des exceptions assumées (« il existe bien
  sûr des exceptions aux valeurs indiquées »), soit des erreurs d'impression.
  Dans les deux cas, leur sommet est d'un point au-dessus du budget.
- **Décision :** en attente d'arbitrage du joueur. En attendant, le moteur
  applique le budget : ces profils sont signalés comme dépassements et ne
  passent qu'avec une exception motivée à la création.
- **Test :** `personnage::tests::humains_du_livre_et_leurs_depassements_de_budget`.
- **Statut :** ouvert.

## E-003 — Pouvoir 245 du livre des Anges : « Agilité » ou « Volonté » ?

- **Source :** `livres/commun-07-pouvoirs-communs.md` (liste des pouvoirs 24x) et
  `livres/ange-17-liste-des-pouvoirs.md`.
- **Constat :** dans la section « Augmentation temporaire des caractéristiques »,
  le livre des Anges liste « 243- Agilité » **et** « 245- Agilité ». Le livre des
  Démons liste « 245- Volonté ». Le tableau récapitulatif du livre des Anges
  indique « 245 Volonté ».
- **Analyse :** coquille du texte de la section MV (la caractéristique manquante
  est la Volonté, comme dans INS et dans le récapitulatif).
- **Décision :** le catalogue du moteur suit le tableau récapitulatif : 245 =
  Volonté dans les deux livres.
- **Statut :** appliqué (aucun test dédié : c'est la donnée générée depuis le récapitulatif).
