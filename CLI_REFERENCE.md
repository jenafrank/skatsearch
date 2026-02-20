# skat_aug23 – CLI Reference

Binary: `target/release/skat_aug23.exe`

```
skat_aug23 <COMMAND> [OPTIONS]
skat_aug23 --help
skat_aug23 <COMMAND> --help
```

---

## Inhaltsverzeichnis

| Gruppe | Kommandos |
|--------|-----------|
| **Analyse (Einzelposition)** | [value-calc](#value-calc), [analysis](#analysis) |
| **Spielplanung (Vorhand)** | [skat-calc](#skat-calc), [best-game](#best-game) |
| **PIMC-Analyse** | [pimc-calc](#pimc-calc), [pimc-best-game](#pimc-best-game) |
| **Playout / Simulation** | [standard-playout](#standard-playout), [analysis-playout](#analysis-playout), [playout](#playout), [points-playout](#points-playout) |
| **Massensimulation (Forschung)** | [analyze-grand](#analyze-grand), [analyze-suit](#analyze-suit), [analyze-null](#analyze-null), [analyze-general](#analyze-general), [analyze-general-hand](#analyze-general-hand) |
| **Hilfswerkzeuge** | [generate-json](#generate-json) |

---

## JSON-Kontext-Formate

Die meisten Kommandos lesen ihren Spielzustand aus einer JSON-Datei. Je nach Kommando wird ein anderes Format erwartet.

### `GameContextInput` – Vollständige Spielposition

```json
{
  "declarer_cards": "CJ SJ HJ DJ CA SA HA DA CT ST",
  "left_cards":     "C9 C8 C7 S9 S8 S7 H9 H8 H7 DK",
  "right_cards":    "CK CQ HQ HK HT DQ DT D9 D8 D7",
  "game_type":      "Grand",
  "start_player":   "Declarer",
  "mode":           "Value",
  "trick_cards":    null,
  "trick_suit":     null,
  "declarer_start_points": null,
  "samples":        null,
  "god_players":    null
}
```

| Feld | Typ | Beschreibung |
|------|-----|-------------|
| `declarer_cards` | String | Karten des Alleinspielers (Leerzeichen-getrennt) |
| `left_cards` | String | Karten von Links |
| `right_cards` | String | Karten von Rechts |
| `game_type` | `"Grand"` \| `"Null"` \| `"Suit"` | Spieltyp |
| `start_player` | `"Declarer"` \| `"Left"` \| `"Right"` | Anführender Spieler |
| `mode` | `"Value"` \| `"Win"` \| `null` | Suchmodus (optional) |
| `trick_cards` | String \| null | Bereits im laufenden Stich gespielte Karten |
| `trick_suit` | `"clubs"` \| `"spades"` \| `"hearts"` \| `"diamonds"` \| `"trump"` \| null | Farbe des laufenden Stichs |
| `declarer_start_points` | Zahl \| null | Bereits gesammelte Punkte (bei Mid-Game-Analyse) |

> **Hinweis zu 30-Karten-Kontexten:** Wenn `declarer_cards` + `left_cards` + `right_cards` = 30 Karten (inkl. Skat), werden die 2 Skatkarten automatisch erkannt und ihre Punkte dem Alleinspieler gutgeschrieben.

### `PimcContextInput` – PIMC-Analyse (unvollständige Information)

```json
{
  "game_type":      "Grand",
  "my_player":      "Declarer",
  "my_cards":       "CJ SJ HJ DJ CA SA HA DA CT ST",
  "remaining_cards": "C9 C8 C7 S9 S8 S7 H9 H8 H7 DK CK CQ HQ HK HT DQ DT D9 D8 D7",
  "trick_cards":    null,
  "trick_suit":     null,
  "previous_card":  null,
  "next_card":      null,
  "declarer_start_points": 0,
  "threshold":      61,
  "samples":        20,
  "facts": {
    "declarer": null,
    "left": { "no_trump": true, "no_clubs": false, "no_spades": false, "no_hearts": false, "no_diamonds": false },
    "right": null
  }
}
```

### `PimcBestGameInput` – Spielauswahl mit unvollständiger Information

```json
{
  "my_cards":    "CJ SJ HJ DJ CA SA HA DA CT ST",
  "start_player": "Declarer",
  "description": "Optional free-text info"
}
```

### Karten-Notation

Karten werden als 2-Zeichen-Codes angegeben: `{Suit}{Rank}`

| Suit-Kürzel | Bedeutung |
|-------------|-----------|
| `C` | Kreuz (Clubs) |
| `S` | Pik (Spades) |
| `H` | Herz (Hearts) |
| `D` | Karo (Diamonds) |

| Rank-Kürzel | Bedeutung |
|-------------|-----------|
| `7` `8` `9` | Sieben, Acht, Neun |
| `T` | Zehn |
| `J` | Bube |
| `Q` | Dame |
| `K` | König |
| `A` | Ass |

**Beispiele:** `CJ` = Kreuz-Bube, `HT` = Herz-Zehn, `DA` = Karo-Ass

---

## Analyse (Einzelposition)

### `value-calc`

Berechnet den **exakten Spielwert** einer Position mittels Alpha-Beta-Suche unter der Annahme, dass alle Karten bekannt sind (Perfect Information).

```
skat_aug23 value-calc --context <JSON> [--optimum-mode <MODE>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (GameContextInput) |
| `--optimum-mode <MODE>` | — | Wenn gesetzt: `best_value` oder `all_winning` |

**Ausgabe (stdout):**
- Ohne `--optimum-mode`: `Value: <N>, Best Card: <CARD>`
- Mit `--optimum-mode best_value`: beste Karte + Wert
- Mit `--optimum-mode all_winning`: schnellste Gewinnlinie (Win-Optimierung)

**Anwendungsfälle:**

```bash
# Exakten Punktwert einer 10-Karten-Stellung bestimmen
skat_aug23 value-calc --context game_state.json

# Beim Spielwert: Gewinnoptimierung (alle gewinnenden Karten)
skat_aug23 value-calc --context game_state.json --optimum-mode all_winning

# Mid-Game: Wert nach Stich 3, Alleinspieler hat 14 Punkte
# → trick_cards, trick_suit und declarer_start_points im JSON setzen
skat_aug23 value-calc --context mid_game.json
```

---

### `analysis`

**Einzelschritt-Analyse:** Berechnet den Wert jedes legalen Zugs des aktiven Spielers.

```
skat_aug23 analysis --context <JSON>
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (GameContextInput) |

**Ausgabe (stdout):** Liste aller legalen Züge mit ihrem jeweiligen Wert, sortiert absteigend.

**Anwendungsfall:**
```bash
# Welche Karte sollte ich an Position X spielen?
skat_aug23 analysis --context decision_point.json
```

---

## Spielplanung (Vorhand)

### `skat-calc`

Ermittelt die **optimale Skataussage** (2-Karten-Abwurf) für eine 12-Karten-Hand.

```
skat_aug23 skat-calc --context <JSON> [--mode <MODE>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei mit 12 Deklarant-Karten |
| `-m, --mode <MODE>` | `best` | `best` \| `all` \| `win` |

**Modi:**
- `best` – Schnellster Modus. Liefert nur das beste Abwurfpaar.
- `all` – Listet alle möglichen Abwürfe nach Wert sortiert.
- `win` – Zeigt nur Abwürfe, die zu ≥ 61 Punkten führen.

**Ausgabe (stdout):** Optimale Karten + erreichter Spielwert.

**Anwendungsfälle:**
```bash
# Beste Skataussage finden (schnell)
skat_aug23 skat-calc --context hand_12cards.json

# Alle Optionen vergleichen
skat_aug23 skat-calc --context hand_12cards.json --mode all

# Welche Skataussagen gewinnen überhaupt?
skat_aug23 skat-calc --context hand_12cards.json --mode win
```

---

### `best-game`

Ermittelt den **optimalen Spieltyp** (Grand, Farbe, Null) für eine 12-Karten-Hand.

```
skat_aug23 best-game --context <JSON> [--mode <MODE>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei mit 12 Deklarant-Karten |
| `-m, --mode <MODE>` | `best` | `best` \| `win` |

**Modi:**
- `best` – Spieltyp mit dem höchsten erreichbaren Punktwert.
- `win` – Spieltyp, der am sichersten gewinnt (≥ 61 Punkte).

**Ausgabe (stdout):** Bester Spieltyp + Wert/Wahrscheinlichkeit.

```bash
# Was ist der beste Spieltyp für diese Hand?
skat_aug23 best-game --context hand_12cards.json

# Welcher Spieltyp gewinnt am sichersten?
skat_aug23 best-game --context hand_12cards.json --mode win
```

---

## PIMC-Analyse

### `pimc-calc`

Berechnet den Spielwert oder Zugwert unter **unvollständiger Information** via PIMC-Sampling.

```
skat_aug23 pimc-calc --context <JSON> [--mode <MODE>] [--log-file <PATH>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (PimcContextInput) |
| `-m, --mode <MODE>` | `win` | `win` \| `best` |
| `--log-file <PATH>` | — | Optional: Schreibt Sample-Details in diese Datei |

**Modi:**
- `win` – Schätzt die Gewinnwahrscheinlichkeit für den aktuellen Zustand.
- `best` – Schätzt die Gewinnwahrscheinlichkeit jedes legalen Zugs.

**Ausgabe:**
- stdout: Wahrscheinlichkeit(en)
- `--log-file`: Detaillierte Sample-Protokolle (JSON-Format, ein Sample pro Zeile)

```bash
# Wie wahrscheinlich gewinne ich aus dieser Position?
skat_aug23 pimc-calc --context pimc_state.json

# Welche Karte hat die beste Gewinnchance?
skat_aug23 pimc-calc --context pimc_state.json --mode best

# Mit Detail-Log für spätere Analyse
skat_aug23 pimc-calc --context pimc_state.json --mode best --log-file pimc_detail.log
```

---

### `pimc-best-game`

Ermittelt den besten Spieltyp **mit unvollständiger Information** über die Gegenspielerkarten.

```
skat_aug23 pimc-best-game --context <JSON> [--samples <N>] [--log-file <PATH>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (PimcBestGameInput) |
| `-s, --samples <N>` | `20` | PIMC-Samples pro Spieltyp-Variante |
| `--log-file <PATH>` | — | Optional: Schreibt Sample-Details in diese Datei |

**Ausgabe (stdout):** Bester Spieltyp + geschätzte Gewinnwahrscheinlichkeit.

```bash
# Beste Spielart bei unbekannten Gegenspielerkarten (20 Samples)
skat_aug23 pimc-best-game --context my_hand.json

# Genauere Schätzung mit mehr Samples
skat_aug23 pimc-best-game --context my_hand.json --samples 100

# Mit Protokoll
skat_aug23 pimc-best-game --context my_hand.json --samples 50 --log-file best_game.log
```

---

## Playout / Simulation

### `standard-playout`

Spielt die Partie unter **Perfect Information** (alle Karten bekannt) komplett aus. Zeigt den optimalen Spielverlauf.

```
skat_aug23 standard-playout --context <JSON>
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (GameContextInput) |

**Ausgabe (stdout):** Stich-für-Stich-Protokoll + Endergebnis.

```bash
skat_aug23 standard-playout --context game.json
```

---

### `analysis-playout`

Wie `standard-playout`, aber bei **jedem Zug** werden alle legalen Züge mit ihrem Wert ausgegeben.

```
skat_aug23 analysis-playout --context <JSON>
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --context <FILE>` | — | JSON-Datei (GameContextInput) |

**Ausgabe (stdout):** Stich-für-Stich-Protokoll mit vollständiger Zugliste pro Entscheidungspunkt.

```bash
# Warum wurde Karte X gespielt? (didaktische Analyse)
skat_aug23 analysis-playout --context game.json
```

---

### `playout`

PIMC-Playout: Simuliert eine Partie unter **unvollständiger Information** (Win-Wahrscheinlichkeits-Optimierung). Vergleicht PIMC-Züge mit der Perfect-Play-Referenz.

```
skat_aug23 playout [--game-type <TYPE>] [--start-player <PLAYER>]
                   [--context <JSON>] [--samples <N>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `--game-type <TYPE>` | `grand` | `grand` \| `null` \| `clubs` (und weitere Farben) |
| `--start-player <PLAYER>` | `declarer` | `declarer` \| `left` \| `right` |
| `-c, --context <FILE>` | — | Optional: JSON-Kontext; ohne = zufälliger Deal |
| `-s, --samples <N>` | `20` | PIMC-Samples pro Zug |

**Ausgabe (stdout):**
- Perfect Play Sektion: Stich-für-Stich Referenzlinie
- PIMC Sektion: Stich-für-Stich PIMC-Entscheidungen mit Abweichungsmarkierungen

```bash
# Zufälliger Grand mit Standard-Samples
skat_aug23 playout --game-type grand --start-player declarer

# Konkreter Kontext, mehr Samples für bessere Qualität
skat_aug23 playout --context game.json --samples 50

# Null-Spiel simulieren
skat_aug23 playout --game-type null --start-player declarer --samples 30
```

---

### `points-playout`

PIMC-Playout mit **Punkte-Optimierung** (statt Win/Loss). Nützlicher als `playout` für realistischere Zugentscheidungen und als Trainingsdaten-Generator. Vergleicht jeden Zug mit der Perfect-Play-Referenz und berechnet den Punktverlust.

```
skat_aug23 points-playout [--game-type <TYPE>] [--start-player <PLAYER>]
                           [--context <JSON>] [--samples <N>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `--game-type <TYPE>` | `grand` | `grand` \| `null` \| `suit` |
| `--start-player <PLAYER>` | `declarer` | `declarer` \| `left` \| `right` |
| `-c, --context <FILE>` | — | Optional: JSON-Kontext; ohne = zufälliger Deal |
| `-s, --samples <N>` | `20` | PIMC-Samples pro Zug |

**Ausgabe (stdout):** Detailliertes Stich-für-Stich-Protokoll im Format:

```
=== Perfect Play Simulation ===
  Trick  1  D:SA  L:S7  R:SK
  ...
  Perfect Play Finished. Result: 72 pts

=== PIMC Points Simulation (20 samples/move) ===
-- Trick  1 (D leads) ----------------------------------------------------------------
  D    PIMC:    SA   perf:SA opt= 72  loss=0(D)  avgs:[SA=68  HA=61  CT=55  ...]
  L  * PIMC:    H7   perf:HQ opt= 61  loss=0(O)  avgs:[H7=91  HQ=78  ...]
  R    PIMC:    SK   perf:SK opt= 58  loss=0(O)  avgs:[SK=83  ...]
  +- score after trick  1: 11 pts
-- Trick  2 (R leads) ----------------------------------------------------------------
  ...
Game Finished. Total Point Loss: 7 (D:7 O:0) | Final score: 65 pts
```

**Spalten-Erklärung:**

| Marker | Bedeutung |
|--------|-----------|
| `D` / `L` / `R` | Spieler (Declarer / Left / Right) |
| `*` | PIMC hat eine andere Karte als Perfect Play gewählt |
| `(!)` | Abweichung hat einen Punktverlust > 0 verursacht |
| `perf:XX` | Perfect-Play-Referenzkarte |
| `opt=NN` | Optimaler Punktwert bei Perfect Play |
| `loss=N(D)` | Punktverlust des Zugs; `D` = Alleinspieler, `O` = Gegner |
| `avgs:[...]` | Durchschnittliche Punktwerte aller legalen Karten laut PIMC |

```bash
# Suit-Spiel mit 20 Samples (Standard)
skat_aug23 points-playout --game-type suit --start-player declarer

# Mit konkretem Kontext und mehr Samples
skat_aug23 points-playout --context game.json --samples 50

# Null-Spiel analysieren
skat_aug23 points-playout --game-type null --samples 30

# Als Loop über 100 Spiele (Python-Wrapper)
python run_points_playout_loop.py
# → Ausgabe: points_playout_log.txt (im Arbeitsverzeichnis)
```

---

## Massensimulation (Forschung)

### `analyze-grand`

Generiert zufällige Grand-Hände und analysiert ihre Gewinnwahrscheinlichkeit (PIMC). Ausgabe als CSV.

```
skat_aug23 analyze-grand [--count <N>] [--samples <N>] [--output <PATH>]
                          [--hand] [--post-discard]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --count <N>` | `100` | Anzahl zu analysierender Hände |
| `-s, --samples <N>` | `100` | PIMC-Samples pro Hand |
| `-o, --output <PATH>` | stdout | Ausgabe-CSV-Pfad |
| `--hand` | `false` | Hand-Spiel (kein Skat-Aufnehmen) |
| `--post-discard` | `false` | Analyse nach optimalem Abwurf |

**CSV-Ausgabe:** Enthält Hand-Signatur (Buben, Asse, Zehner, Farblängen etc.) und Gewinnwahrscheinlichkeit.

```bash
# 1000 Hände nach stdout
skat_aug23 analyze-grand --count 1000 --samples 50

# CSV-Datei für spätere Auswertung
skat_aug23 analyze-grand --count 5000 --samples 100 --output research/data/grand_sim.csv

# Hand-Spiel (kein Skataufnehmen)
skat_aug23 analyze-grand --count 1000 --samples 100 --output grand_hand.csv --hand

# Analyse nach Skataufnahme und optimalem Abwurf
skat_aug23 analyze-grand --count 1000 --samples 100 --output grand_post.csv --post-discard
```

---

### `analyze-suit`

Wie `analyze-grand`, aber für **Farb-Spiele** (aktuell: Kreuz).

```
skat_aug23 analyze-suit [--count <N>] [--samples <N>] [--output <PATH>]
                         [--hand] [--post-discard]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --count <N>` | `100` | Anzahl zu analysierender Hände |
| `-s, --samples <N>` | `100` | PIMC-Samples pro Hand |
| `-o, --output <PATH>` | stdout | Ausgabe-CSV-Pfad |
| `--hand` | `false` | Hand-Spiel |
| `--post-discard` | `false` | Analyse nach optimalem Abwurf |

```bash
skat_aug23 analyze-suit --count 5000 --samples 100 --output research/data/suit_sim.csv
```

---

### `analyze-null`

Detaillierte **Null-Spiel-Simulation** mit PIMC-Trajektorie. Erzeugt pro Hand: Kartenverteilung, Gewinn/Verlust, Folgekarten mit Win-Wahrscheinlichkeiten.

```
skat_aug23 analyze-null [--count <N>] [--samples <N>] [--output <PATH>] [--hand]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --count <N>` | `5000` | Anzahl Hände |
| `-s, --samples <N>` | `20` | Samples pro Zug |
| `-o, --output <PATH>` | `research/data/null_sim_detailed.csv` | Ausgabe-CSV |
| `--hand` | `false` | Null-Hand (kein Skataufnehmen) |

**CSV-Spalten:** `Hand, Skat, Won, Points, Moves, DurationMs, StartPlayer`

```bash
# Standard: 5000 Null-Hände ins Default-CSV
skat_aug23 analyze-null

# Kleinere Stichprobe schnell testen
skat_aug23 analyze-null --count 500 --samples 10 --output test_null.csv

# Null-Hand-Spiel (kein Skat)
skat_aug23 analyze-null --count 2000 --hand --output null_hand_sim.csv

# 48h-Run auf Server (Bash-Beispiel)
for i in $(seq 1 16); do
  skat_aug23 analyze-null --count 50000 --samples 20 \
    --output "null_sim_part_$i.csv" &
done
```

---

### `analyze-general`

Analysiert **allgemeine Vorhand-Stärke**: Für jede generierte Hand wird bestimmt, welchen Spieltyp man am besten spielen kann und wie hoch die jeweilige Gewinnwahrscheinlichkeit ist.

```
skat_aug23 analyze-general [--count <N>] [--samples <N>] [--output <PATH>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `--count <N>` | `100` | Anzahl Hände |
| `--samples <N>` | `20` | PIMC-Samples pro Spieltyp |
| `--output <PATH>` | `research/data/general_pre_stats.csv` | Ausgabe-CSV |

**CSV-Spalten:** InitHand, InitSkat, FinalHand, SkatCards, JacksMask, Aces, Tens, ... WinProb, ProbGrand, ProbClubs, ProbSpades, ProbHearts, ProbDiamonds, ProbNull, WonMask, BestGame, DurationMs

```bash
skat_aug23 analyze-general --count 10000 --samples 20 \
  --output research/data/general_pre_stats.csv
```

---

### `analyze-general-hand`

Wie `analyze-general`, aber für **Hand-Spiele** (ohne Skataufnehmen).

```
skat_aug23 analyze-general-hand [--count <N>] [--samples <N>] [--output <PATH>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `--count <N>` | `100` | Anzahl Hände |
| `--samples <N>` | `100` | PIMC-Samples pro Spieltyp |
| `--output <PATH>` | `research/data/hand_best_game.csv` | Ausgabe-CSV |

```bash
skat_aug23 analyze-general-hand --count 5000 --samples 100 \
  --output research/data/hand_best_game.csv
```

---

## Hilfswerkzeuge

### `generate-json`

Sucht zufällig nach Grand-Händen mit einer **Mindesstgewinnwahrscheinlichkeit** (1 Bube + 2 Asse als Filterkriterium) und speichert sie als spielfertige JSON-Dateien.

```
skat_aug23 generate-json [--count <N>] [--min-win <F>] [--output-dir <DIR>]
```

| Option | Standard | Beschreibung |
|--------|----------|-------------|
| `-c, --count <N>` | `10` | Anzahl gesuchter Szenarien |
| `--min-win <F>` | `0.8` | Mindest-Gewinnwahrscheinlichkeit (0.0–1.0) |
| `--output-dir <DIR>` | `generated_scenarios` | Ausgabeverzeichnis |

**Ausgabe:** Pro Szenario zwei Dateien:
- `<output-dir>/scenario_N_win_NN.json` – Spielkontext (GameContextInput)
- `<output-dir>/scenario_N_info.txt` – Skatkarten + Gewinnwahrscheinlichkeit

```bash
# 5 Szenarien mit ≥ 80% Gewinnchance (Standard)
skat_aug23 generate-json --count 5

# Schwieriger: ≥ 90%, eigenes Verzeichnis
skat_aug23 generate-json --count 20 --min-win 0.9 --output-dir test_scenarios/hard
```

---

## Python-Wrapper und Hilfsskripte

| Skript | Beschreibung |
|--------|-------------|
| `run_points_playout_loop.py` | Führt 100 `points-playout`-Spiele aus und schreibt das Protokoll nach `points_playout_log.txt` |
| `analyze_loss_games.py` | Liest `points_playout_log.txt` und analysiert Verlustmuster |
| `run_suit_sim.ps1` | PowerShell-Script für Langzeit-Suit-Simulationen (Parallelisierung) |

### `run_points_playout_loop.py`

```bash
python run_points_playout_loop.py
```

Konfiguration am Anfang der Datei:

| Variable | Standard | Beschreibung |
|----------|----------|-------------|
| `N_GAMES` | `100` | Anzahl Spiele |
| `SAMPLES` | `20` | PIMC-Samples pro Zug |
| `LOG_FILE` | `points_playout_log.txt` | Ausgabedatei (überschreibt) |

**Ausgabepfad:** `points_playout_log.txt` im Arbeitsverzeichnis (`skatsearch/`)

---

## Standardwerte auf einen Blick

| Kommando | Samples | Output |
|----------|---------|--------|
| `pimc-calc` | — | stdout |
| `pimc-best-game` | 20 | stdout |
| `playout` | 20 | stdout |
| `points-playout` | 20 | stdout |
| `analyze-grand` | 100 | stdout |
| `analyze-suit` | 100 | stdout |
| `analyze-null` | 20 | `research/data/null_sim_detailed.csv` |
| `analyze-general` | 20 | `research/data/general_pre_stats.csv` |
| `analyze-general-hand` | 100 | `research/data/hand_best_game.csv` |
| `generate-json` | 20/100 | `generated_scenarios/` |
