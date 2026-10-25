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
