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

## Séance d'essai « Hardcore-Holocauste »

Statut des entrées ci-dessous : toutes validées par le joueur et appliquées dans le moteur, chacune figée
par les tests nommés à la fin de l'entrée.

- **D-017 — Règle maison « neutraliser » (statut : appliquée).**
  Origine : Dédé (FO2) mort sous 10 dommages, un chauffeur sous 6, un tireur sous 4 ; les poings
  d'un Ange de Force 5 tuent les humains. Le livre n'a aucune règle de ce genre (les dommages
  humains vont jusqu'à la mort à Force -2, `demon-12-blessures.md`, `commun-10`).
  1. *Portée.* Quand un Ange ou un partenaire PNJ des Anges attaque une cible à table de blessures
     humaine (humain, Serviteur, Soldat) **sans déclarer vouloir la tuer**, l'attaque est « neutralisante » ;
     c'est le comportement par défaut.
  2. *Calcul.* Dommages calculés normalement (D-007). Dommages appliqués = min(dommages, points
     restants avant le seuil d'évanouissement). Pour un humain : la Force courante (arrivé à 0,
     évanoui D6 minutes, D-005). Le dépassement n'est pas appliqué : il se traduit par
     l'évanouissement ou, si le MJ préfère le raconter ainsi, par une maîtrise (cible ceinturée,
     désarmée, hors de combat) pour la même durée. Jamais de coma ni de mort.
  3. *Cible déjà évanouie* : plus de dommages (plafond nul).
  4. *Intention de tuer.* Si le joueur déclare vouloir tuer (avant le jet), dommages normaux, mort
     possible. Les conséquences pour un Ange (chute, hiérarchie) restent à l'arbitrage du MJ.
  5. *Surnaturels* (Anges, Archanges, Démons, Princes, familiers, morts-vivants) : dommages
     normaux, pas de plafond. Un PNJ ennemi (humain ou non) qui attaque un personnage ne neutralise
     pas : dommages normaux.
  6. *Exclusions.* Le MJ peut refuser la neutralisation pour une attaque de zone ou de rafale
     (grenade, lance-flammes, M-60), à décider au cas par cas.
  Application : l'attaquant est « de l'équipe » quand son rôle est joueur ou partenaire ; paramètre `tuer`
  de `attaquer` ; la marge avant l'évanouissement se relit sur la fiche au moment d'appliquer.
  Tests : `combat::tests::neutraliser_plafonne_a_levanouissement`, `neutraliser_ne_vise_que_les_humains_et_les_attaquants_de_l_equipe`,
  `jeu::tests::par_defaut_un_ange_neutralise_un_humain_sauf_intention_de_tuer`.
- **D-018 — Modificateurs improvisés par le MJ (statut : appliquée ; les trois valeurs proposées sont validées).**
  - *Cible invisible mais localisée.* **Livre** (pouvoir 261, `demon-06`/`ange-06`) : les attaquants
    subissent **1 + niveau** colonnes de malus (pas pour les conflits psychiques). Billy (Invisibilité+1)
    : -2 : le -2 utilisé en séance était exact. Décidée (livre). Invisibilité non surnaturelle
    (obscurité) : table des modificateurs, « Nuit » (-1 contact, -2 distance).
  - *Espace confiné* (arrière de camionnette). Le livre n'a rien (la table de `commun-09` n'a ni
    « confiné » ni « exigu »). Proposée : -1 colonne aux attaques de contact, pour tous les
    combattants ; armes de contact lourdes -2 ; aucun effet à distance. Le -1 utilisé en séance
    reste cohérent (même ordre que « attaquant mobile » et « nuit » au contact).
  - *Bonus pour bonne interprétation d'un jet social.* Le livre n'a pas de tel bonus : il donne au
    MJ le droit d'accorder des colonnes (lunette +1, visée +2 dans l'exemple d'Albert, `commun-05`),
    et réserve l'interprétation aux **unités** du jet réussi (ex. Piko : deux containers de plus).
    Proposée : +1 colonne au plus, accordé **avant** le jet, pas cumulable avec une récompense
    d'unités pour le même jet ; à noter dans le journal.
  - *Fouille d'une pièce.* Aucun talent Fouille dans le livre : action sans talent, caractéristique
    la plus proche (Perception), niveau de difficulté du barème de `commun-05` « Utilisation des
    caractéristiques » : 0 difficile, +1 **moyen**, +2 facile. « Moyen » pour une pièce ordinaire
    est donc conforme ; +2 si c'est un objet évident, 0 si c'est caché. Décidée (livre).
  Application : paramètre `invisibilite` (niveau du pouvoir) de `attaquer` ; circonstance `espace_confine`
  (-1 au contact, -2 pour une arme de contact lourde, rien à distance). Le bonus social reste un
  décalage de colonnes accordé par le MJ avant le jet, sans code.
  Tests : `combat::tests::invisibilite_malus_1_plus_niveau`, `espace_confine_malus_au_contact_et_plus_pour_les_armes_lourdes`.
- **D-019 — Talent à +3 par les pouvoirs 341-346 (statut : appliquée).** Le livre
  donne explicitement le niveau +3 au talent choisi (« il choisit un talent humain qu'il acquiert au
  niveau +3 », 346 : Combat). Le plafond de création +2 concerne les points de talents (1/2/4 points
  pour 0/+1/+2). Règle : un talent à +3 est accepté **hors budget de points** s'il est rattaché à un
  pouvoir 34x de la fiche, un par pouvoir, et choisi dans la famille du tirage (346 = talents de
  combat : Arme de poing, Arme d'épaule, Arme lourde, Lancer, Arme de contact, Arme de contact lourde,
  Corps à corps). Le pouvoir 346 compte un tirage (niveau 0). Un talent possédé en plus à un niveau
  inférieur (Billy : Arme d'épaule+2) est remplacé par le +3, il ne se cumule pas : Billy n'a que
  « Arme d'épaule+3 ». La bidouille « +2 plus 1 colonne de bonus » de la fiche actuelle disparaît.
  Message d'erreur : « niveau 3 hors pouvoir 34x (maximum +2 à la création) ». Test :
  `personnage::tests::talent_3_via_pouvoir_346`.
- **D-020 — Armes et talents créés par le MJ (statut : appliquée, valeurs validées).** Le livre
  autorise le MJ à créer armes et talents. Plutôt que des avertissements « absent du catalogue »,
  une section « maison » du catalogue (marquée comme telle) :
  - Armes (valeurs proposées, entre le .22 et le .44) : Revolver cal.38 : +1, précision -, portée 12,
    6 coups, Arme de poing. Pistolet automatique : +1, précision -, portée 15, 15 coups, Arme de
    poing. Griffes de goule/squelette : arme de contact, +1 (goules, squelettes, profils du livre),
    précision -, talent Corps à corps ; la note a (Corps à corps ≥ +2 : +1 dommage) ne s'y ajoute pas
    (la puissance imprimée du profil la remplace).
  - Talents (malus hors talent proposés) : Déguisement (-2, Ruse, Précision), Intimidation (-2,
    Communication, Volonté ou Force), Fouille (-1, Ruse, Perception), Observation (-1, Ruse,
    Perception). Comédie : déjà au catalogue, -3 MV / -4 INS (le catalogue ne garde qu'une valeur :
    à vérifier selon le livre de la partie).
  Application : `moteur/regles/donnees/maison.json` (armes, talents et le pouvoir spécial Suicide de Furfur),
  chargé avec le catalogue et marqué `maison`. Test : `catalogue::tests::section_maison_marquee`.
- **D-021 — Actions simultanées (statut : appliquée).** Livre (`commun-09`,
  « Le combat ») : points 2 et 5 ; ceux de même Agilité agissent simultanément, et les effets des
  dommages prennent effet immédiatement. Lecture retenue : entre deux rangs d'Agilité, effets
  immédiats (un mort ou un inconscient n'agit plus au rang suivant, D-006) ; à l'intérieur d'un rang,
  tous les jets sont faits avec l'état du début du rang, puis les dommages sont appliqués ensemble.
  Conséquences : deux adversaires peuvent se toucher mutuellement ; un personnage tué ou assommé à ce
  rang a quand même frappé ; un bonus de blessure ne rétroagit pas dans le rang. Contournement actuel
  (`appliquer: false` puis application à la main) reste possible. Application : outil `attaques_simultanees`.
  Test : `jeu::tests::rang_simultane_applique_apres_tous_les_jets`.
- **D-022 — Goules et serviteurs 51x (statut : appliquée).** Le profil imprimé du
  pouvoir fait foi (livre : 515 = FO2 AG3 AP2 VO1 PE2 PR2, griffes +1, Corps à corps+2, 3 talents
  à 0 ; 516 = FO1 AG2 AP2 VO1 PE2 PR1, Corps à corps+1). Le budget de création des morts-vivants
  (12 / 4 / 2 pouvoirs) ne s'applique pas à ces créatures : pas de dépassement signalé tant que le
  profil est celui du livre. Un profil de scénario qui s'en écarte (Déguisement+1 des goules du
  scénario) passe en D-003 (exception motivée) et non en budget. Peur+1 : capacité innée (D-006 :
  pas de malus de blessure), sans PP ; PP0. Test : `personnage::tests::goule_515_conforme`.
- **D-023 — Récupération des PP (statut : appliquée).** 1 PP par 24 h est le plancher
  du repos complet, pas le barème d'un Ange ou d'un Démon. Serviteur 6 h, Soldat 3 h, mort-vivant
  12 h, familier 6 h sont des barèmes propres. Pour l'Ange et le Démon : barème du patron
  (Archange / Prince, `ange-16`, `demon-16`) ; à défaut, 24 h. Le repos nocturne ne s'ajoute pas au
  barème : le patron l'indique (ex. Furfur : +1 par 4 h de repos). Pas de cumul plancher + barème.
  Application : `Espece::bareme_du_patron` et `REPOS_COMPLET_HEURES`. Test : `personnage::tests::heures_par_pp_plancher_24_h`.
- **D-024 — PP de PNJ de scénario (statut : appliquée, décidée avec E-004).** Le PP maximum imprimé dans
  le scénario peut surcharger le calcul de D-001 pour un PNJ, avec mention dans les notes. Application :
  champ `pp_max_scenario` et paramètre `pp_max` de `creer_personnage` (exige une `exception`). Test :
  `personnage::tests::pp_max_surcharge_pnj`.
