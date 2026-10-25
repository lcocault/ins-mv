# Décisions de règles du moteur

Interprétations retenues quand le livre est muet ou ambigu. Chacune est figée par un
test ; toute modification passe par une discussion avec le gestionnaire du système.
Les écarts constatés dans le livre lui-même sont dans `errata.md`.

## Création (jalon 1)

- **D-001 — Tirages de pouvoirs.** Un pouvoir remonté d'un niveau compte comme un tirage
  de plus : budget de pouvoirs = Σ (niveau + 1) des pouvoirs, et +1 PP par tirage.
  Les limitations comptent pour 1 PP chacune. (`personnage::tests::niveau_de_pouvoir_...`)
- **D-002 — Limitation de création.** Une seule limitation, réservée aux Anges et aux
  Démons, en échange d'un pouvoir de plus (+2 PP au total). Les autres espèces n'en ont pas.
- **D-003 — Exceptions de budget.** Un dépassement des budgets de création n'est accepté que
  par une exception motivée du MJ, tracée dans les notes de la fiche. Une fiche incohérente
  (pouvoir inconnu, talent en double...) n'est jamais acceptée.

## Blessures (jalon 2)

- **D-004 — Soins naturels.** « Il récupère ces points à raison d'un par semaine » est lu
  comme un point de **Force** par semaine, soit 1, 2 ou 3 points de dommage selon l'espèce.
  Les points de dommage restants qui ne forment pas un point de Force disparaissent la
  semaine où la Force est entièrement récupérée. (`blessures::tests::soins_naturels_...`)
- **D-005 — Seuils.** « Arrivé à -2 », c'est quand la Force courante est inférieure ou égale
  à -2. Ange de Force 4 : assommé à 18 dommages, mort à 21.
- **D-006 — Malus de blessure.** Appliqué (colonnes = dommages non guéris) aux humains,
  Serviteurs, Soldats et familiers ; ni aux Anges ni aux Démons (le livre n'en prévoit pas),
  ni aux morts-vivants (explicitement exclus). Un personnage inconscient, dans le coma ou mort
  ne peut plus agir ni se défendre.

## Combat (jalon 2)

- **D-007 — Dommages.** unités finales du conflit + puissance (arme + bonus) + bonus de Force
  au contact (FO - 3 si FO ≥ 4) - protection, minimum 0. Les unités finales sont l'écart des
  unités quand la défense réussit (règle des conflits), les unités de l'attaque sinon.
  Le livre écrit « unités d'attaque - unités de défense » : c'est la même chose.
- **D-008 — Protections.** La protection naturelle (pouvoir 211 Armure corporelle, 2 + niveau)
  et celle d'une armure portée **ne s'additionnent pas** : on retient la meilleure. Le Kevlar
  protège de 4 contre les armes à feu (Arme de poing, d'épaule, lourde) et de 2 sinon.
  Une protection supplémentaire (bouclier, objet) s'ajoute.
- **D-009 — Ajustement d'armure.** Les colonnes de malus d'une armure ne s'appliquent qu'aux
  jets dont la caractéristique est l'Agilité (attaque ou esquive).
- **D-010 — Parade.** Parer avec une arme applique sa précision au jet de défense et ajoute sa
  puissance aux unités de défense. L'esquive n'ajoute rien.
- **D-011 — Cible sans défense.** Surprise, attaquée de dos, inconsciente ou déclarée sans
  défense par le MJ : aucun jet de défense, les unités de l'attaque comptent en entier.
- **D-012 — Égalité des unités.** Le moteur ne tranche jamais : « arbitrage MJ », aucun dommage
  appliqué, et les dommages hypothétiques sont indiqués.
- **D-013 — Initiative.** Phase psychique par Volonté décroissante pour les personnages qui ont
  des pouvoirs (égalités départagées par un D66, règle du livre des Démons appliquée aux deux),
  puis phase physique par Agilité décroissante ; mêmes valeurs d'Agilité : simultané.

## Pouvoirs (jalon 2)

- **D-014 — Coût.** Un coût imprimé fixe (« 4 », « 1/attaque », « 1/défense ») est prélevé
  automatiquement ; un pouvoir sans coût imprimé est gratuit ; un coût variable
  (« 1, 2 ou 3/att. », « 1/minute », « La moitié (M. 4) ») exige que le MJ précise `cout_pp`.
  Le coût est prélevé même si le jet échoue.
- **D-015 — Caractéristique du jet.** Celle imprimée entre parenthèses dans le livre
  (ex. « Calme (Volonté) »), sinon la caractéristique d'attaque du tableau récapitulatif ;
  à défaut, le MJ la précise. Les augmentations de caractéristiques (24x) se jouent à la Volonté.
- **D-016 — Défense contre un pouvoir.** La cible se défend au niveau 0 avec la caractéristique
  de défense du tableau récapitulatif (Volonté pour un pouvoir psychique), comme le dit le
  livre pour le combat psychique.
