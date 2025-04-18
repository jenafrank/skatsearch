use crate::types::{game::Game, player::Player, problem::Problem};
use super::{retargs::{AllGames, SolveRet, SolveWithSkatRet}, withskat::acceleration_mode::AccelerationMode, Solver};
use rayon::prelude::*; // Importiert die notwendigen Rayon-Traits

#[derive(Clone, Copy)]
pub enum ProblemTransformation {
    SpadesSwitch,
    HeartsSwitch,
    DiamondsSwitch
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] // Braucht diese Derives für den Map-Key
pub enum GameKey {
    Eichel,
    Gruen,
    Herz,
    Schell,
    Grand,
    Null,
}

// Definieren Sie einen einfachen Fehlertyp
#[derive(Debug)]
pub enum CalculationError {
    NoBestSkatFound(GameKey), // Kein bester Skat für diesen Spieltyp gefunden
    SolverError(String), // Falls der Solver andere Fehler zurückgeben könnte (optional)
    // Fügen Sie hier weitere Fehlertypen hinzu, falls nötig
}

// Implementieren Sie Display für den Fehlertyp, um ihn leicht auszugeben
impl std::fmt::Display for CalculationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculationError::NoBestSkatFound(key) => write!(f, "Kein bester Skat gefunden für Spieltyp: {:?}", key),
            CalculationError::SolverError(msg) => write!(f, "Solver Fehler: {}", msg),
        }
    }
}

impl Solver {

pub fn calc_all_games(
    left_cards: u32,
    right_cards: u32,
    declarer_cards: u32,
    start_player: Player,
) -> Result<AllGames, CalculationError> {
    let acc_mode = AccelerationMode::AlphaBetaAccelerating;

    // Erstelle das Basis-Problem einmal
    let base_problem_farbe_eichel =
        Problem::create(declarer_cards, left_cards, right_cards, Game::Farbe, start_player);

    // Erstelle die transformierten Probleme (sequenziell, da schnell)
    let problem_farbe_gruen =
        Problem::create_transformation(base_problem_farbe_eichel.clone(), ProblemTransformation::SpadesSwitch);
    let problem_farbe_herz =
        Problem::create_transformation(base_problem_farbe_eichel.clone(), ProblemTransformation::HeartsSwitch);
    let problem_farbe_schell =
        Problem::create_transformation(base_problem_farbe_eichel.clone(), ProblemTransformation::DiamondsSwitch);

    // Erstelle Probleme für Grand und Null
    let problem_grand =
        Problem::create(declarer_cards, left_cards, right_cards, Game::Grand, start_player);
    let problem_null =
        Problem::create(declarer_cards, left_cards, right_cards, Game::Null, start_player);


    // Definiere die Liste der Aufgaben. Jede Aufgabe ist ein Tupel aus
    // (Schlüssel zur Identifizierung, Problem-Instanz, Spieltyp, Beschleunigungsmodus)
    // Problem-Instanzen werden hier move'd, was für die parallele Verarbeitung ok ist,
    // da jede Instanz nur von einer Aufgabe benötigt wird.
    let tasks = vec![
        (GameKey::Eichel, base_problem_farbe_eichel, Game::Farbe, acc_mode),
        (GameKey::Gruen, problem_farbe_gruen, Game::Farbe, acc_mode),
        (GameKey::Herz, problem_farbe_herz, Game::Farbe, acc_mode),
        (GameKey::Schell, problem_farbe_schell, Game::Farbe, acc_mode),
        (GameKey::Grand, problem_grand, Game::Grand, acc_mode),
        (GameKey::Null, problem_null, Game::Null, acc_mode),
    ];

    // Helfer-Closure, die eine einzelne Aufgabe (ein Tupel) nimmt
    // und beide Lösungsarten (solve_with_skat und Hand) berechnet.
    // Gibt ein Result zurück, das den Schlüssel und die beiden Ergebnisse enthält.
    let calculate_single_pair =
        |(key, problem, game_type, acc_mode): (GameKey, Problem, Game, AccelerationMode)|
        -> Result<(GameKey, SolveWithSkatRet, SolveRet), CalculationError>
    {
            // Berechne solve_with_skat
            // Solver::solve_with_skat nimmt die Karten direkt, nicht das Problem.
            // Nutze die Karten des Problems, das bereits die richtige Transformation enthält.
            let skat_result = Solver::solve_with_skat(
                problem.left_cards(),
                problem.right_cards(),
                problem.declarer_cards(),
                game_type,
                problem.start_player(),
                acc_mode,
            );

            // Berechne die Hand-Variante
            // Annahme: solve_and_add_skat ist die korrekte Methode für alle Handspiele,
            // bei denen eine Solver-Instanz erstellt wird.
            let mut solver = Solver::new(problem, None); // problem wird hier move'd
            let hand_result = solver.solve_and_add_skat(); // Nutze die refaktorierte Funktion

            Ok((key, skat_result, hand_result))
        };

    // Führe die Aufgaben parallel aus und sammle die Ergebnisse
    let results: Vec<Result<(GameKey, SolveWithSkatRet, SolveRet), CalculationError>> = tasks
        .into_par_iter() // Rayon's paralleler Iterator
        .map(calculate_single_pair) // Wendet die Closure auf jedes Element parallel an
        .collect(); // Sammelt die Ergebnisse in einem Vektor von Results

    // Verarbeite die gesammelten Ergebnisse und fülle das finale AllGames Struct
    let mut final_games = AllGames::default(); // Oder eine andere geeignete Initialisierung

    for result in results {
        // Nutze den ?-Operator, um bei einem Fehler frühzeitig zurückzukehren
        let (key, skat_ret, hand_ret) = result?;

        // Extrahiere die Werte, behandle den Option-Fall für best_skat
        let skat_value = skat_ret
            .best_skat
            .ok_or(CalculationError::NoBestSkatFound(key))? // Fehler, wenn best_skat None ist
            .value; // Extrahiere den Wert, wenn Some

        let hand_value = hand_ret.best_value;

        // Weise die Werte dem entsprechenden Feld im AllGames Struct zu
        match key {
            GameKey::Eichel => {
                final_games.eichel_farbe = skat_value;
                final_games.eichel_hand = hand_value;
            }
            GameKey::Gruen => {
                final_games.gruen_farbe = skat_value;
                final_games.gruen_hand = hand_value;
            }
            GameKey::Herz => {
                final_games.herz_farbe = skat_value;
                final_games.herz_hand = hand_value;
            }
            GameKey::Schell => {
                final_games.schell_farbe = skat_value;
                final_games.schell_hand = hand_value;
            }
            GameKey::Grand => {
                final_games.grand = skat_value;
                final_games.grand_hand = hand_value;
            }
            GameKey::Null => {
                final_games.null = skat_value;
                final_games.null_hand = hand_value;
            }
        }
    }

    // Gib das finale AllGames Struct als Erfolg zurück
    Ok(final_games)
}

}