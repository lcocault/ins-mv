//! Rendu HTML des pages. Fonctions pures : données en entrée, texte en sortie.
//! Tout texte provenant des fiches ou du journal passe par [`esc`].

use insmv_campagne::EntreeJournal;
use insmv_regles::blessures::EtatSante;
use insmv_regles::catalogue::{self, Livre};
use insmv_regles::personnage::{Carac, Espece, Personnage, Role};
use insmv_regles::{Action, Caracteristique, NiveauTalent, Seuil};

/// Échappe le texte pour l'insérer dans du HTML (contenu et attributs).
pub fn esc(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '&' => sortie.push_str("&amp;"),
            '<' => sortie.push_str("&lt;"),
            '>' => sortie.push_str("&gt;"),
            '"' => sortie.push_str("&quot;"),
            '\'' => sortie.push_str("&#39;"),
            c => sortie.push(c),
        }
    }
    sortie
}

pub fn nom_espece(e: Espece) -> &'static str {
    match e {
        Espece::Humain => "Humain",
        Espece::Serviteur => "Serviteur de Dieu",
        Espece::Soldat => "Soldat de Dieu",
        Espece::Ange => "Ange",
        Espece::MortVivant => "Mort-vivant",
        Espece::Familier => "Familier",
        Espece::Demon => "Démon",
    }
}

pub fn nom_role(r: Role) -> &'static str {
    match r {
        Role::Joueur => "Joueur",
        Role::Partenaire => "Partenaire",
        Role::Pnj => "PNJ",
    }
}

/// Date et heure UTC « 2026-10-25 14:03 » depuis des secondes Unix.
pub fn date_utc(secondes: i64) -> String {
    let jours = secondes.div_euclid(86_400);
    let reste = secondes.rem_euclid(86_400);
    // Conversion jours → date civile (algorithme de H. Hinnant).
    let z = jours + 719_468;
    let ere = z.div_euclid(146_097);
    let jour_ere = z.rem_euclid(146_097);
    let annee_ere = (jour_ere - jour_ere / 1_460 + jour_ere / 36_524 - jour_ere / 146_096) / 365;
    let jour_annee = jour_ere - (365 * annee_ere + annee_ere / 4 - annee_ere / 100);
    let mois_p = (5 * jour_annee + 2) / 153;
    let jour = jour_annee - (153 * mois_p + 2) / 5 + 1;
    let mois = if mois_p < 10 { mois_p + 3 } else { mois_p - 9 };
    let annee = annee_ere + ere * 400 + i64::from(mois <= 2);
    format!("{annee:04}-{mois:02}-{jour:02} {:02}:{:02}", reste / 3600, reste % 3600 / 60)
}

const STYLE: &str = r#"
:root{--fond:#f6f4ef;--carte:#fff;--texte:#1d1b18;--discret:#6b665e;--bord:#ddd7cb;--accent:#8a5a12;--accent-fond:#f3e6cc;--ok:#2f6b3a;--alerte:#a02a2a;--barre:#e6dfd0}
@media (prefers-color-scheme:dark){:root:not([data-theme="light"]){--fond:#17150f;--carte:#201d16;--texte:#ece6d8;--discret:#a39c8c;--bord:#3a3528;--accent:#e0b25c;--accent-fond:#3a2f17;--ok:#7fc28b;--alerte:#e68585;--barre:#33301f}}
:root[data-theme="dark"]{--fond:#17150f;--carte:#201d16;--texte:#ece6d8;--discret:#a39c8c;--bord:#3a3528;--accent:#e0b25c;--accent-fond:#3a2f17;--ok:#7fc28b;--alerte:#e68585;--barre:#33301f}
*{box-sizing:border-box}
body{margin:0;background:var(--fond);color:var(--texte);font:16px/1.5 system-ui,-apple-system,"Segoe UI",sans-serif}
a{color:var(--accent)}
header.site{display:flex;flex-wrap:wrap;gap:12px 24px;align-items:center;justify-content:space-between;padding:12px 16px;border-bottom:1px solid var(--bord);background:var(--carte)}
header.site nav a{margin-right:16px;text-decoration:none;font-weight:600}
header.site label{color:var(--discret);font-size:14px}
main{max-width:980px;margin:0 auto;padding:16px}
h1{margin:8px 0 2px;font-size:28px}
h2{margin:28px 0 8px;font-size:18px;border-bottom:1px solid var(--bord);padding-bottom:4px}
.sous-titre{color:var(--discret);margin:0 0 12px}
.carte{background:var(--carte);border:1px solid var(--bord);border-radius:8px;padding:12px 16px}
.grille{display:grid;gap:12px;grid-template-columns:repeat(auto-fit,minmax(280px,1fr))}
.stats{display:grid;gap:8px;grid-template-columns:repeat(auto-fit,minmax(86px,1fr))}
.stat{border:1px solid var(--bord);border-radius:8px;padding:8px;text-align:center;background:var(--carte)}
.stat b{display:block;font-size:26px;line-height:1.1}
.stat span{color:var(--discret);font-size:13px}
.pips{display:flex;gap:3px;justify-content:center;margin-top:4px}
.pips i{width:8px;height:8px;border-radius:50%;background:var(--barre)}
.pips i.plein{background:var(--accent)}
.barre{height:14px;background:var(--barre);border-radius:7px;overflow:hidden;margin:6px 0}
.barre div{height:100%;background:var(--accent)}
table{width:100%;border-collapse:collapse;background:var(--carte);border:1px solid var(--bord);border-radius:8px;overflow:hidden}
th,td{text-align:left;padding:6px 10px;border-bottom:1px solid var(--bord);vertical-align:top}
th{font-size:13px;color:var(--discret);font-weight:600}
tr:last-child td{border-bottom:0}
.table-defilante{overflow-x:auto}
.pastille{display:inline-block;padding:1px 8px;border-radius:10px;font-size:12px;background:var(--accent-fond);color:var(--accent);font-weight:600}
.pastille.alerte{background:transparent;border:1px solid var(--alerte);color:var(--alerte)}
.pastille.limitation{background:transparent;border:1px solid var(--alerte);color:var(--alerte)}
.discret{color:var(--discret)}
.pct{font-variant-numeric:tabular-nums}
.notes{white-space:pre-line}
.liste-persos{list-style:none;padding:0;margin:0;display:grid;gap:12px;grid-template-columns:repeat(auto-fill,minmax(260px,1fr))}
.liste-persos a.carte{display:block;text-decoration:none;color:inherit}
.liste-persos a.carte:hover{border-color:var(--accent)}
form.filtres{display:flex;flex-wrap:wrap;gap:8px;margin:8px 0 16px}
form.filtres input,form.filtres select,form.filtres button{font:inherit;padding:6px 8px;border:1px solid var(--bord);border-radius:6px;background:var(--carte);color:var(--texte)}
.entree{margin:0 0 10px}
.entree .meta{color:var(--discret);font-size:13px}
@media (max-width:520px){main{padding:12px}h1{font-size:24px}}
"#;

const SCRIPT: &str = r#"
(function(){
  var case_ = document.getElementById('auto'); if(!case_) return;
  var cle = 'insmv-auto', actif = false;
  try { actif = localStorage.getItem(cle) === '1'; } catch (e) {}
  case_.checked = actif;
  var minuteur = null;
  function appliquer(){
    if (minuteur) { clearInterval(minuteur); minuteur = null; }
    if (case_.checked) minuteur = setInterval(function(){ location.reload(); }, 5000);
  }
  case_.addEventListener('change', function(){
    try { localStorage.setItem(cle, case_.checked ? '1' : '0'); } catch (e) {}
    appliquer();
  });
  appliquer();
})();
"#;

pub fn page(titre: &str, contenu: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{titre} — In Nomine</title>
<style>{STYLE}</style>
</head>
<body>
<header class="site">
<nav><a href="/">Personnages</a><a href="/journal">Journal</a></nav>
<label><input type="checkbox" id="auto"> Actualiser toutes les 5 s</label>
</header>
<main>
{contenu}
</main>
<script>{SCRIPT}</script>
</body>
</html>
"#,
        titre = esc(titre)
    )
}

pub fn page_erreur(statut: &str, message: &str) -> String {
    page(statut, &format!("<h1>{}</h1><p>{}</p><p><a href=\"/\">Retour aux personnages</a></p>", esc(statut), esc(message)))
}

// ---- Liste ---------------------------------------------------------------------

pub fn liste(personnages: &[Personnage]) -> String {
    if personnages.is_empty() {
        return page(
            "Personnages",
            "<h1>Personnages</h1><p class=\"discret\">Aucun personnage dans la campagne pour l'instant.</p>",
        );
    }
    let mut html = String::from("<h1>Personnages</h1><ul class=\"liste-persos\">");
    for p in personnages {
        html.push_str(&format!(
            "<li><a class=\"carte\" href=\"/personnage/{id}\"><strong>{nom}</strong>\
             <div class=\"discret\">{espece} · {role}</div>\
             <div>PP {pp}/{pp_max} · dommages {dom}</div></a></li>",
            id = esc(&p.id),
            nom = esc(&p.nom),
            espece = nom_espece(p.espece),
            role = nom_role(p.role),
            pp = p.pp_courants(),
            pp_max = p.pp_max(),
            dom = p.dommages,
        ));
    }
    html.push_str("</ul>");
    page("Personnages", &html)
}

// ---- Fiche ---------------------------------------------------------------------

fn niveau_texte(n: u8) -> String {
    if n == 0 { "+0".to_string() } else { format!("+{n}") }
}

/// « AG 5 → ≤ 66 (100 %) » pour une caractéristique, un niveau et la fiche donnés.
fn chance(p: &Personnage, carac: Carac, niveau: u8) -> Option<String> {
    let action = Action::new(
        Caracteristique::new(p.caracteristiques.get(carac)).ok()?,
        NiveauTalent::new(niveau).ok()?,
    );
    Some(match action.seuil() {
        Seuil::Valeur(v) => format!(
            "{} {} → ≤ {v} <span class=\"pct discret\">({} %)</span>",
            abreviation(carac),
            p.caracteristiques.get(carac),
            action.seuil().pourcentage()
        ),
        autre => format!("{} → {autre:?}", abreviation(carac)),
    })
}

fn abreviation(c: Carac) -> &'static str {
    match c {
        Carac::FO => "FO",
        Carac::PR => "PR",
        Carac::VO => "VO",
        Carac::PE => "PE",
        Carac::AG => "AG",
        Carac::AP => "AP",
    }
}

pub fn fiche(p: &Personnage) -> String {
    let budget = p.espece.budget();
    let (pp, pp_max) = (p.pp_courants(), p.pp_max());
    let part = if pp_max == 0 { 0 } else { pp * 100 / pp_max };
    let recup = match budget.heures_par_pp {
        Some(_) if matches!(p.espece, Espece::Ange | Espece::Demon) => {
            "récupération : selon le barème du patron, et 1 PP par 24 h de repos complet".to_string()
        }
        Some(h) => format!("récupération : 1 PP toutes les {h} h (et 1 PP par 24 h de repos complet)"),
        None => "aucun point de pouvoir".to_string(),
    };

    let mut html = format!("<h1>{}</h1>", esc(&p.nom));
    html.push_str(&format!(
        "<p class=\"sous-titre\">{} · {}{}</p>",
        nom_espece(p.espece),
        nom_role(p.role),
        p.patron.as_deref().map(|x| format!(" · au service de {}", esc(x))).unwrap_or_default()
    ));

    // État.
    html.push_str("<div class=\"grille\"><section class=\"carte\"><strong>Points de pouvoir</strong>");
    html.push_str(&format!(
        "<div class=\"barre\" role=\"progressbar\" aria-valuemin=\"0\" aria-valuemax=\"{pp_max}\" aria-valuenow=\"{pp}\" \
         aria-label=\"Points de pouvoir\"><div style=\"width:{part}%\"></div></div>\
         <div><b>{pp}</b> / {pp_max}</div><div class=\"discret\">{recup}</div></section>"
    ));
    let sante = p.sante();
    let (etat, classe) = match sante.etat {
        EtatSante::Valide => ("Valide", ""),
        EtatSante::Inconscient => ("Inconscient", " alerte"),
        EtatSante::Coma => ("Dans le coma", " alerte"),
        EtatSante::Mort => ("Mort", " alerte"),
    };
    let malus = if sante.malus_colonnes > 0 {
        format!("malus de blessure : -{} colonne(s) sur toutes les actions", sante.malus_colonnes)
    } else {
        "aucun malus de blessure".to_string()
    };
    html.push_str(&format!(
        "<section class=\"carte\"><strong>Dommages subis</strong><div style=\"font-size:26px\"><b>{dom}</b></div>\
         <div class=\"discret\">points non encore guéris</div>\
         <div>Force {courante} (d'origine {origine}) · <span class=\"pastille{classe}\">{etat}</span></div>\
         <div class=\"discret\">{malus}</div></section></div>",
        dom = p.dommages,
        courante = sante.force_courante,
        origine = p.caracteristiques.fo,
    ));

    // Caractéristiques.
    html.push_str("<h2>Caractéristiques</h2><div class=\"stats\">");
    for c in Carac::TOUTES {
        let v = p.caracteristiques.get(c);
        let pips: String = (1..=6).map(|i| format!("<i class=\"{}\"></i>", if i <= v { "plein" } else { "" })).collect();
        html.push_str(&format!(
            "<div class=\"stat\"><span>{}</span><b>{v}</b><span>{}</span><div class=\"pips\" aria-hidden=\"true\">{pips}</div></div>",
            c.nom(),
            abreviation(c)
        ));
    }
    html.push_str("</div>");

    // Talents.
    html.push_str("<h2>Talents</h2>");
    if p.talents.is_empty() {
        html.push_str("<p class=\"discret\">Aucun talent.</p>");
    } else {
        html.push_str("<div class=\"table-defilante\"><table><thead><tr><th>Talent</th><th>Niveau</th><th>Jet de table UM</th></tr></thead><tbody>");
        for t in &p.talents {
            let jets: Vec<String> = catalogue::talent_par_nom(&t.nom)
                .map(|c| c.caracteristiques.iter().filter_map(|a| Carac::analyser(a)).collect::<Vec<_>>())
                .unwrap_or_default()
                .into_iter()
                .filter_map(|c| chance(p, c, t.niveau))
                .collect();
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                esc(&t.nom),
                niveau_texte(t.niveau),
                if jets.is_empty() { "<span class=\"discret\">—</span>".to_string() } else { jets.join("<br>") }
            ));
        }
        html.push_str("</tbody></table></div>");
    }

    // Pouvoirs.
    html.push_str("<h2>Pouvoirs</h2>");
    if p.pouvoirs.is_empty() {
        html.push_str("<p class=\"discret\">Aucun pouvoir.</p>");
    } else {
        let livre: Option<Livre> = p.espece.livre_pouvoirs();
        html.push_str("<div class=\"table-defilante\"><table><thead><tr><th>N°</th><th>Pouvoir</th><th>Niveau</th><th>Famille</th><th>Jet</th><th>Coût (PP)</th></tr></thead><tbody>");
        for pv in &p.pouvoirs {
            let cat = livre.zip(pv.code).and_then(|(l, c)| catalogue::pouvoir_par_code(l, c));
            let jet = cat
                .and_then(|c| c.attaque.as_deref())
                .and_then(Carac::analyser)
                .and_then(|c| chance(p, c, pv.niveau))
                .unwrap_or_else(|| "<span class=\"discret\">—</span>".to_string());
            let niveau = if pv.limitation {
                "<span class=\"pastille limitation\">limitation</span>".to_string()
            } else {
                niveau_texte(pv.niveau)
            };
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{niveau}</td><td class=\"discret\">{}</td><td>{jet}</td><td>{}</td></tr>",
                pv.code.map(|c| c.to_string()).unwrap_or_default(),
                esc(&pv.nom),
                esc(cat.map(|c| c.categorie.as_str()).unwrap_or("")),
                esc(cat.and_then(|c| c.cout.as_deref()).unwrap_or("—")),
            ));
        }
        html.push_str("</tbody></table></div>");
    }

    // Équipement et notes.
    html.push_str("<h2>Équipement</h2>");
    if p.equipement.is_empty() {
        html.push_str("<p class=\"discret\">Rien.</p>");
    } else {
        html.push_str("<ul>");
        for e in &p.equipement {
            html.push_str(&format!("<li>{}</li>", esc(e)));
        }
        html.push_str("</ul>");
    }
    if !p.notes.trim().is_empty() {
        html.push_str(&format!("<h2>Notes</h2><p class=\"notes\">{}</p>", esc(&p.notes)));
    }
    page(&p.nom, &html)
}

// ---- Journal -------------------------------------------------------------------

/// Valeurs courantes du formulaire de filtre.
#[derive(Debug, Default, Clone)]
pub struct FiltresAffiches {
    pub seance: String,
    pub categorie: String,
    pub contient: String,
    pub personnage: String,
}

pub fn journal(
    entrees: &[EntreeJournal],
    noms_par_id: &dyn Fn(&str) -> String,
    personnages: &[Personnage],
    filtres: &FiltresAffiches,
) -> String {
    let mut html = String::from("<h1>Journal de campagne</h1><form class=\"filtres\" method=\"get\" action=\"/journal\">");
    html.push_str(&format!(
        "<input name=\"seance\" placeholder=\"Séance\" value=\"{}\" aria-label=\"Séance\">\
         <input name=\"categorie\" placeholder=\"Catégorie\" value=\"{}\" aria-label=\"Catégorie\">\
         <input name=\"contient\" placeholder=\"Texte contenu\" value=\"{}\" aria-label=\"Texte contenu\">\
         <select name=\"personnage\" aria-label=\"Personnage\"><option value=\"\">Tous les personnages</option>",
        esc(&filtres.seance),
        esc(&filtres.categorie),
        esc(&filtres.contient)
    ));
    for p in personnages {
        let choisi = if p.id == filtres.personnage { " selected" } else { "" };
        html.push_str(&format!("<option value=\"{}\"{choisi}>{}</option>", esc(&p.id), esc(&p.nom)));
    }
    html.push_str("</select><button type=\"submit\">Filtrer</button></form>");

    if entrees.is_empty() {
        html.push_str("<p class=\"discret\">Aucune entrée.</p>");
    }
    // Plus récentes d'abord.
    for e in entrees.iter().rev() {
        let qui: Vec<String> = e.personnages.iter().map(|id| format!("<a href=\"/personnage/{}\">{}</a>", esc(id), esc(&noms_par_id(id)))).collect();
        let mut meta = vec![date_utc(e.horodatage)];
        if !e.seance.is_empty() {
            meta.push(esc(&e.seance));
        }
        if !e.categorie.is_empty() {
            meta.push(format!("<span class=\"pastille\">{}</span>", esc(&e.categorie)));
        }
        html.push_str(&format!(
            "<article class=\"carte entree\"><div class=\"meta\">{}{}</div><div class=\"notes\">{}</div></article>",
            meta.join(" · "),
            if qui.is_empty() { String::new() } else { format!(" · {}", qui.join(", ")) },
            esc(&e.texte)
        ));
    }
    page("Journal", &html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use insmv_regles::personnage::{Caracteristiques, PouvoirPossede, TalentPossede};

    fn jean() -> Personnage {
        let mut p = Personnage {
            id: String::new(),
            nom: "Jean De Lavenue".into(),
            espece: Espece::Ange,
            role: Role::Joueur,
            caracteristiques: Caracteristiques { fo: 4, pr: 2, vo: 4, pe: 3, ag: 5, ap: 2 },
            talents: vec![
                TalentPossede { nom: "Arme de contact".into(), niveau: 2 },
                TalentPossede { nom: "Esquive".into(), niveau: 1 },
            ],
            pouvoirs: [413u16, 166, 622]
                .map(|c| PouvoirPossede { code: Some(c), nom: String::new(), niveau: 0, limitation: false })
                .into(),
            patron: Some("Laurent".into()),
            pp: None,
            dommages: 2,
            equipement: vec!["Erendil".into()],
            notes: "ligne 1\nligne 2".into(),
        };
        p.finaliser();
        p
    }

    #[test]
    fn echappement_html() {
        assert_eq!(esc("<b>\"a\" & 'b'</b>"), "&lt;b&gt;&quot;a&quot; &amp; &#39;b&#39;&lt;/b&gt;");
    }

    #[test]
    fn dates_utc() {
        assert_eq!(date_utc(0), "1970-01-01 00:00");
        assert_eq!(date_utc(951_782_400), "2000-02-29 00:00");
        assert_eq!(date_utc(1_792_939_573), "2026-10-25 14:46");
        assert_eq!(date_utc(1_709_251_199), "2024-02-29 23:59");
    }

    #[test]
    fn la_fiche_affiche_etat_caracteristiques_et_jets() {
        let html = fiche(&jean());
        assert!(html.contains("Jean De Lavenue"));
        assert!(html.contains("Ange · Joueur · au service de Laurent"));
        assert!(html.contains("<b>7</b> / 7"), "PP : VO 4 + 3 tirages (413, 166, limitation)");
        assert!(html.contains("<b>2</b></div>"), "dommages");
        // Arme de contact +2 avec AG 5 : seuil 66, 100 %.
        assert!(html.contains("AG 5 → ≤ 66 <span class=\"pct discret\">(100 %)</span>"));
        // Calme : Volonté 4, niveau 0 → 32, 39 %.
        assert!(html.contains("VO 4 → ≤ 32 <span class=\"pct discret\">(39 %)</span>"));
        assert!(html.contains("limitation</span>"));
        assert!(html.contains("Erendil"));
        assert!(html.contains("ligne 1\nligne 2"));
    }

    #[test]
    fn recuperation_des_pp_depend_du_patron_pour_anges_et_demons() {
        let html = fiche(&jean());
        assert!(html.contains("selon le barème du patron"));
        assert!(!html.contains("toutes les 24 h"));

        let mut serviteur = jean();
        serviteur.espece = Espece::Serviteur;
        serviteur.pouvoirs.truncate(1);
        serviteur.finaliser();
        assert!(fiche(&serviteur).contains("1 PP toutes les 6 h"));
    }

    #[test]
    fn la_fiche_affiche_la_sante() {
        let html = fiche(&jean()); // Ange Force 4, 2 dommages
        assert!(html.contains("Force 4 (d'origine 4) · <span class=\"pastille\">Valide</span>"));
        assert!(html.contains("aucun malus de blessure"));

        let mut blesse = jean();
        blesse.dommages = 9; // 3 points de Force perdus
        assert!(fiche(&blesse).contains("Force 1 (d'origine 4)"));

        blesse.dommages = 18; // Force -2 : assommé
        assert!(fiche(&blesse).contains("<span class=\"pastille alerte\">Inconscient</span>"));

        let mut humain = jean();
        humain.espece = Espece::Soldat;
        humain.pouvoirs.truncate(1);
        humain.finaliser();
        humain.dommages = 3;
        assert!(fiche(&humain).contains("malus de blessure : -3 colonne(s)"));
    }

    #[test]
    fn contenu_utilisateur_est_echappe() {
        let mut p = jean();
        p.nom = "<script>alert(1)</script>".into();
        p.notes = "<img src=x onerror=alert(1)>".into();
        p.equipement = vec!["\"><b>".into()];
        let html = fiche(&p);
        assert!(!html.contains("<script>alert"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    }

    #[test]
    fn liste_vide_et_remplie() {
        assert!(liste(&[]).contains("Aucun personnage"));
        let html = liste(&[jean()]);
        assert!(html.contains("href=\"/personnage/jean-de-lavenue\""));
        assert!(html.contains("PP 7/7 · dommages 2"));
    }

    #[test]
    fn journal_recent_d_abord_et_liens_vers_les_fiches() {
        let entree = |id, texte: &str| EntreeJournal {
            id,
            horodatage: 1_792_939_573,
            seance: "Séance 1".into(),
            categorie: "scene".into(),
            texte: texte.into(),
            personnages: vec!["jean-de-lavenue".into()],
        };
        let entrees = vec![entree(1, "Premier"), entree(2, "Second <b>")];
        let html = journal(&entrees, &|_| "Jean De Lavenue".into(), &[jean()], &FiltresAffiches::default());
        assert!(html.find("Second").unwrap() < html.find("Premier").unwrap());
        assert!(html.contains("Second &lt;b&gt;"));
        assert!(html.contains("<a href=\"/personnage/jean-de-lavenue\">Jean De Lavenue</a>"));
        assert!(html.contains("2026-10-25"));
    }

    #[test]
    fn journal_conserve_les_filtres_dans_le_formulaire() {
        let f = FiltresAffiches { seance: "S\"1".into(), personnage: "jean-de-lavenue".into(), ..Default::default() };
        let html = journal(&[], &|id| id.into(), &[jean()], &f);
        assert!(html.contains("value=\"S&quot;1\""));
        assert!(html.contains("value=\"jean-de-lavenue\" selected"));
        assert!(html.contains("Aucune entrée"));
    }
}
