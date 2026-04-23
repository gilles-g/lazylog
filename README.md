# lazylog

TUI pour explorer les fichiers de logs (Symfony / Monolog, nginx access & error,
Apache access & error, PHP errors, texte générique), inspiré de lazygit.

- Ouverture en mémoire mappée (pas de recopie) — tient sur des fichiers de plusieurs Go.
- Parsing en tâche de fond, UI jamais bloquée.
- Facettes cliquables (niveau, canal, méthode, status, IP, pays…), recherche plein texte,
  filtre par plage de dates, histogramme temporel.
- Navigation clavier type vim : `j/k`, `g/G`, `PgUp/PgDn`.
- Les événements les plus récents s'affichent **en bas** de la liste, comme `tail -f`.

## Installation

```bash
# Depuis les sources (nécessite cargo)
cargo install --path .

# Ou binaire pré-compilé
tar -xzf lazylog-<version>-<os>-<arch>.tar.gz
install -m 0755 lazylog-*/lazylog "$HOME/.local/bin/lazylog"
```

## Utilisation

```bash
# Ouvre un fichier directement
lazylog /var/log/nginx/access.log

# Pas de chemin → picker interactif qui scanne var/log, logs/ et /var/log
lazylog

# Forcer un format si l'auto-détection se trompe
lazylog --format nginx-access access.log

# Restreindre la période chargée (plus rapide sur gros fichiers)
lazylog --from '2026-04-22' --to '2026-04-22 18:00:00' access.log
lazylog --all huge.log   # désactive le prompt date-range sur fichiers > 100 Mo
```

Formats reconnus pour `--format` :
`symfony`, `php`, `nginx-access`, `nginx-error`, `apache-access`, `apache-error`, `generic`.

### Raccourcis clavier

| Touche           | Action                                  |
|------------------|-----------------------------------------|
| `q` / `Ctrl-C`   | quitter                                 |
| `?`              | afficher / masquer l'aide               |
| `1` / `2`        | onglet Events / Histogram               |
| `j` / `↓`        | descendre (vers plus récent)            |
| `k` / `↑`        | monter (vers plus ancien)               |
| `g`              | haut de la liste (plus ancien)          |
| `G`              | bas de la liste (plus récent, tail)     |
| `PgUp` / `PgDn`  | saut de 10 lignes                       |
| `f` / `e`        | focus panneau Facets / Events           |
| `Space`          | activer / désactiver une valeur de facette |
| `/`              | recherche plein texte                   |
| `d`              | modale de plage de dates                |
| `r`              | reset de tous les filtres               |
| `x`              | menu d'export (facette focus / log filtré → `.txt`) |
| `Enter`          | ouvrir le détail (à venir)              |
| `Esc`            | fermer popup / effacer la recherche     |

## Facette « Country » via GeoIP

`lazylog` peut afficher une facette **Country** sur les logs access (et error)
nginx/apache, en résolvant chaque IP cliente vers son pays à l'aide d'une base
GeoIP2 au format `.mmdb`. Sans base, la facette n'apparaît simplement pas — le
reste du TUI fonctionne normalement.

### 1. Télécharger une base `.mmdb`

Deux options gratuites, interchangeables (même format MaxMind DB) :

**DB-IP Lite (recommandé — pas de compte)**
Licence [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/), mise à jour mensuelle.

```bash
# Remplace YYYY-MM par le mois courant (ex: 2026-04)
curl -fL -o /tmp/dbip.mmdb.gz \
  "https://download.db-ip.com/free/dbip-country-lite-YYYY-MM.mmdb.gz"
gunzip /tmp/dbip.mmdb.gz
```

**MaxMind GeoLite2 Country** (gratuit mais nécessite un compte + licence key)
<https://www.maxmind.com/en/geolite2/signup> puis télécharger `GeoLite2-Country.mmdb`.

### 2. Placer la base à un endroit auto-détecté

`lazylog` cherche, dans l'ordre :

1. Le fichier passé à `--geoip /chemin/vers/geoip.mmdb`
2. `$LAZYLOG_GEOIP` (variable d'environnement)
3. `$XDG_DATA_HOME/lazylog/geoip.mmdb`
4. `~/.local/share/lazylog/geoip.mmdb`
5. `~/.lazylog/geoip.mmdb`

Install type (pas besoin de flag ensuite) :

```bash
mkdir -p ~/.local/share/lazylog
mv /tmp/dbip.mmdb ~/.local/share/lazylog/geoip.mmdb
```

Ou usage ponctuel :

```bash
lazylog --geoip ~/Downloads/dbip.mmdb access.log
# ou
LAZYLOG_GEOIP=~/Downloads/dbip.mmdb lazylog access.log
```

### 3. Vérifier

Au démarrage, un log est écrit dans `$XDG_CACHE_HOME/lazylog/lazylog.log`
(ou `~/.cache/lazylog/lazylog.log`) :

```
[INFO  lazylog] geoip database loaded: /home/you/.local/share/lazylog/geoip.mmdb
```

Dans le TUI, sur un log nginx/apache access, une rubrique **Country** apparaît
dans le panneau Facets (top 15 pays par volume). `Space` pour filtrer.

### Notes

- La résolution est faite au chargement, en tâche de fond, avec un cache
  mémoire par IP (les IPs qui reviennent souvent ne paient le coût qu'une
  seule fois).
- Aucune requête réseau n'est faite au runtime : la base `.mmdb` est entièrement
  locale.
- Les IPs privées (10.0.0.0/8, 192.168.0.0/16, etc.) ne sont pas géolocalisées
  et n'apparaissent pas dans la facette.
- Respecter la licence de la base choisie si tu redistribues les résultats.

## Journal applicatif

En cas de problème de parsing ou de chargement, le journal est ici :

```
$XDG_CACHE_HOME/lazylog/lazylog.log
# ou, à défaut :
~/.cache/lazylog/lazylog.log
```

Niveau ajustable via `RUST_LOG=debug lazylog …`.

## Licence

MIT.
# lazylog
