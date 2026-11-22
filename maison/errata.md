# Errata et décisions de règles

Registre des écarts constatés dans les règles de base (`regles/`, immuables) et
des décisions prises pour le moteur. Chaque entrée indique la règle appliquée
par le moteur et le test qui la fige.

## E-001 — Porte enfoncée par BOKA (Force 5, difficulté moyenne)

- **Source :** `regles/commun-05-table-um.md`, exemple [MV] « Autre exemple »
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
- **Statut :** `regles/` n'est pas modifié, par respect de son statut de
  référence immuable ; cet erratum fait foi pour le moteur.

## E-002 — Humains d'exemple qui dépassent le budget de création

- **Source :** `regles/commun-06-creation-personnages-humains.md`, tableau
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

- **Source :** `regles/commun-07-pouvoirs-communs.md` (liste des pouvoirs 24x) et
  `regles/ange-17-liste-des-pouvoirs.md`.
- **Constat :** dans la section « Augmentation temporaire des caractéristiques »,
  le livre des Anges liste « 243- Agilité » **et** « 245- Agilité ». Le livre des
  Démons liste « 245- Volonté ». Le tableau récapitulatif du livre des Anges
  indique « 245 Volonté ».
- **Analyse :** coquille du texte de la section MV (la caractéristique manquante
  est la Volonté, comme dans INS et dans le récapitulatif).
- **Décision :** le catalogue du moteur suit le tableau récapitulatif : 245 =
  Volonté dans les deux livres.
- **Statut :** appliqué (aucun test dédié : c'est la donnée générée depuis le récapitulatif).

## E-004 — PP de Billy : 12 au scénario, 11 au moteur

- **Source :** `scenarios/MV-hardcore-holocauste/scenario.md` (Billy : VO5, « PP12 » ; pouvoirs
  Invisibilité+1, Bond+0, Combat (346), Arme à distance maudite, Goules (515), Suicide+1 spécial) ;
  `regles/demon-02-regles-specifiques.md` (« PP maximum égal à la volonté, plus un point par
  tirage sur la table des pouvoirs »).
- **Constat :** fiche de campagne : 5 pouvoirs encodés (Suicide non encodé), 6 tirages
  (Invisibilité+1 = 2) + VO5 = 11. Avec Suicide+1 encodé, D-001 donnerait 5 + 8 = 13. Aucune lecture
  de D-001 ne donne 12 ; on l'obtient seulement par VO5 + 6 pouvoirs listés + 1 niveau.
- **Analyse :** le livre dit « un point par tirage » sans dire si un pouvoir remonté d'un niveau
  compte pour un tirage de plus (D-001 est une interprétation). Billy a 6 pouvoirs pour 4 autorisés
  à un Démon débutant : le profil du scénario ne suit pas les règles de création, et son total de
  PP est une valeur de scénario, pas un calcul.
- **Décision :** le PP maximum imprimé d'un PNJ de scénario fait foi (surcharge explicite et motivée
  dans la fiche) ; D-001 reste la règle pour les personnages créés (voir D-024).
- **Test :** `personnage::tests::pp_max_surcharge_pnj`.
- **Statut :** appliqué (Billy : 12 PP).

## E-005 — Goules : le budget « mort-vivant » ne s'applique pas au pouvoir 515

- **Source :** `regles/demon-09-pouvoirs-serviteurs.md` (515 Goules D6 ; 516 Goules 2D6),
  `regles/demon-03-creation-personnages.md` (mort-vivant de base : 12 points de caractéristiques,
  4 points de talents, 2 pouvoirs).
- **Constat :** le livre donne le profil complet des goules : FO2 AG3 AP2 VO1 PE2 PR2 (12 points,
  coquille « F2 » pour FO2), griffes (Puissance +1), Corps à corps+2, plus 3 talents au niveau 0.
  Ce profil coûte 4 + 3 = 7 points de talents, au-dessus du budget de 4 d'un mort-vivant créé
  à la main. Le pouvoir Peur+1 est une capacité innée (aucun PP, aucun pouvoir compté). Les goules
  du scénario (Corps à corps+2, Déguisement+1, Acrobatie+0) coûtent 7 points : écart au livre
  (1 talent à +1 au lieu de 3 talents à 0, Déguisement absent de la liste), assumé comme « évoluées ».
- **Analyse :** les créatures des pouvoirs 51x ne sont pas des personnages créés : leur profil est
  imprimé. Le budget de création n'a pas à les juger.
- **Décision :** voir D-022.
- **Statut :** appliqué.

## E-006 — Talent au niveau +3 : plafond de création et pouvoirs 341-346

- **Source :** `regles/commun-04-talents.md` (échelle jusqu'à +3 « niveau impossible pour un être
  humain, rare même pour un Démon » ; « jusqu'au niveau 3 (le maximum) ») ;
  `regles/demon-07-pouvoirs-utilitaires.md` et `ange-07` (34x : talent « humain » au niveau +3) ;
  `regles/demon-03-creation-personnages.md` (« Le niveau maximum d'un talent est +2 »).
- **Constat :** le maximum de création (+2) et le +3 des pouvoirs 34x sont compatibles : le +3 ne
  s'achète pas avec les points de talents, il est donné par le pouvoir. Le moteur le refuse parce que
  `cout_talent` ne connaît pas le niveau 3, et son message (« niveau 3 invalide (0 à 3) ») est
  contradictoire avec `Erreur::NiveauTalent` (« 0 à 3 »).
- **Décision :** voir D-019.
- **Statut :** appliqué.

## E-007 — Catalogues : ce que le livre ne prévoit pas

- **Source :** `regles/commun-09-confrontation-armes-combat.md`, `regles/commun-04-talents.md`.
- **Constat :** le livre ne contient ni « Pistolet automatique », ni « Revolver cal.38 » (seulement
  cal.22 et cal.44 ; le Colt .45 n'apparaît que dans un exemple), ni arme « Griffes » dans la table
  (les griffes viennent du pouvoir 113 : +2 et 1 de précision, ou des profils de goules/squelettes :
  +1 ou +2). Talents absents : Déguisement, Intimidation, Fouille, Observation. Comédie existe
  seulement comme exemple de talent artistique (-3 MV, -4 INS). Le livre précise : la liste « n'est
  pas du tout limitative », le MJ crée talents et armes ; un talent non possédé se joue au niveau 0
  avec le malus imprimé (règle de `commun-04-talents.md`), et une action sans talent se joue sur la
  caractéristique la plus proche avec difficulté 0 / +1 / +2 (`commun-05-table-um.md`).
- **Décision :** voir D-020.
- **Statut :** appliqué.

## E-008 — Récupération des PP des Anges et Démons : 24 h plancher, barème du patron

- **Source :** `regles/ange-02-regles-specifiques.md` et `demon-02-regles-specifiques.md`
  (« Quel que soit ce barème, … au repos complet récupère 1PP par 24 heures ») ;
  `ange-03-creation-personnages.md` (Ange : « barème spécial qui dépend de leur patron (et un tous
  les 24 heures de toute façon) ») ; `ange-16-archanges.md`, `demon-16-princes-demons.md`
  (rubriques « Récupération » par patron) ; Serviteur : 6 h ; Soldat : 3 h ; mort-vivant : 12 h ;
  familier : 6 h.
- **Constat :** le moteur fixe `heures_par_pp = 24` pour l'Ange et le Démon. Or 24 h est le plancher
  commun du repos complet ; le rythme réel d'un Ange ou d'un Démon est celui de son patron (ex.
  Furfur : +1 par quatre heures de repos en écoutant du hard-core, +3 par humain tué en concert).
  Le livre ne dit pas si le plancher s'ajoute au barème ; « de toute façon » et « quel que soit ce
  barème » se lisent comme un minimum garanti, pas un cumul.
- **Décision :** voir D-023.
- **Statut :** appliqué.
