#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use postflop_solver::*;
use std::sync::Mutex;
use sysinfo::{System, SystemExt};

#[derive(Default)]
struct RangeManager([Range; 2]);

struct PoolManager {
    pool: rayon::ThreadPool,
}

impl Default for PoolManager {
    fn default() -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(RangeManager::default()))
        .manage(Mutex::new(PostFlopGame::default()))
        .manage(Mutex::new(PoolManager::default()))
        .invoke_handler(tauri::generate_handler![
            available_memory,
            range_clear,
            range_update,
            range_from_string,
            range_to_string,
            range_get_weights,
            range_raw_data,
            game_init,
            game_private_hand_cards,
            game_memory_usage,
            game_allocate_memory,
            game_solve_step,
            game_exploitability,
            game_finalize,
            game_apply_history,
            game_available_actions,
            game_is_terminal_action,
            game_possible_cards,
            game_current_player,
            game_results
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn available_memory() -> u64 {
    let system = System::new_all();
    1024 * system.available_memory()
}

#[tauri::command]
fn range_clear(player: usize, range_state: tauri::State<Mutex<RangeManager>>) {
    let range = &mut (range_state.lock().unwrap().0)[player];
    range.clear();
}

#[derive(serde::Deserialize)]
struct RangeUpdatePayload {
    player: usize,
    row: u8,
    col: u8,
    weight: f32,
}

#[tauri::command]
fn range_update(payload: RangeUpdatePayload, range_state: tauri::State<Mutex<RangeManager>>) {
    let range = &mut (range_state.lock().unwrap().0)[payload.player];
    let rank1 = 13 - payload.row;
    let rank2 = 13 - payload.col;
    if payload.row == payload.col {
        range.set_weight_pair(rank1, payload.weight)
    } else if payload.row < payload.col {
        range.set_weight_suited(rank1, rank2, payload.weight)
    } else {
        range.set_weight_offsuit(rank1, rank2, payload.weight)
    }
    .unwrap();
}

#[derive(serde::Deserialize)]
struct RangeFromStringPayload {
    player: usize,
    string: String,
}

#[tauri::command]
fn range_from_string(
    payload: RangeFromStringPayload,
    range_state: tauri::State<Mutex<RangeManager>>,
) -> Option<String> {
    let range = &mut (range_state.lock().unwrap().0)[payload.player];
    let result = Range::from_sanitized_ranges(payload.string.as_str());
    if result.is_ok() {
        *range = result.unwrap();
        None
    } else {
        result.err()
    }
}

#[tauri::command]
fn range_to_string(player: usize, range_state: tauri::State<Mutex<RangeManager>>) -> String {
    let range = &(range_state.lock().unwrap().0)[player];
    range.to_string()
}

#[tauri::command]
fn range_get_weights(player: usize, range_state: tauri::State<Mutex<RangeManager>>) -> Vec<f32> {
    let range = &(range_state.lock().unwrap().0)[player];
    let mut weights = vec![0.0; 13 * 13];

    for row in 0..13 {
        for col in 0..13 {
            let rank1 = 12 - row as u8;
            let rank2 = 12 - col as u8;
            if row == col {
                weights[row * 13 + col] = range.get_weight_pair(rank1);
            } else if row < col {
                weights[row * 13 + col] = range.get_weight_suited(rank1, rank2);
            } else {
                weights[row * 13 + col] = range.get_weight_offsuit(rank1, rank2);
            }
        }
    }

    weights
}

#[tauri::command]
fn range_raw_data(player: usize, range_state: tauri::State<Mutex<RangeManager>>) -> Vec<f32> {
    let range = &(range_state.lock().unwrap().0)[player];
    range.raw_data().to_vec()
}

#[derive(serde::Deserialize)]
struct GameInitPayload {
    num_threads: usize,
    board: Vec<u8>,
    starting_pot: i32,
    effective_stack: i32,
    oop_flop_bet: Vec<f32>,
    oop_flop_raise: Vec<f32>,
    oop_turn_bet: Vec<f32>,
    oop_turn_raise: Vec<f32>,
    oop_river_bet: Vec<f32>,
    oop_river_raise: Vec<f32>,
    ip_flop_bet: Vec<f32>,
    ip_flop_raise: Vec<f32>,
    ip_turn_bet: Vec<f32>,
    ip_turn_raise: Vec<f32>,
    ip_river_bet: Vec<f32>,
    ip_river_raise: Vec<f32>,
    add_all_in_threshold: f32,
    force_all_in_threshold: f32,
    adjust_last_two_bet_sizes: bool,
}

#[tauri::command]
fn game_init(
    payload: GameInitPayload,
    range_state: tauri::State<Mutex<RangeManager>>,
    game_state: tauri::State<Mutex<PostFlopGame>>,
    pool_state: tauri::State<Mutex<PoolManager>>,
) -> Option<String> {
    let ranges = &range_state.lock().unwrap().0;
    let mut game = game_state.lock().unwrap();
    let mut pool_manager = pool_state.lock().unwrap();

    pool_manager.pool = rayon::ThreadPoolBuilder::new()
        .num_threads(payload.num_threads)
        .build()
        .unwrap();

    let turn = if payload.board.len() >= 4 {
        payload.board[3]
    } else {
        NOT_DEALT
    };

    let river = if payload.board.len() == 5 {
        payload.board[4]
    } else {
        NOT_DEALT
    };

    let convert_bet_sizes = |sizes: &[f32]| {
        sizes
            .iter()
            .map(|&x| {
                if x >= 0.0 {
                    BetSize::PotRelative(x)
                } else {
                    BetSize::LastBetRelative(-x)
                }
            })
            .collect()
    };

    let bet_sizes = |bet: &[f32], raise: &[f32]| BetSizeCandidates {
        bet: convert_bet_sizes(bet),
        raise: convert_bet_sizes(raise),
    };

    let config = GameConfig {
        flop: payload.board[..3].try_into().unwrap(),
        turn,
        river,
        starting_pot: payload.starting_pot,
        effective_stack: payload.effective_stack,
        range: *ranges,
        flop_bet_sizes: [
            bet_sizes(&payload.oop_flop_bet, &payload.oop_flop_raise),
            bet_sizes(&payload.ip_flop_bet, &payload.ip_flop_raise),
        ],
        turn_bet_sizes: [
            bet_sizes(&payload.oop_turn_bet, &payload.oop_turn_raise),
            bet_sizes(&payload.ip_turn_bet, &payload.ip_turn_raise),
        ],
        river_bet_sizes: [
            bet_sizes(&payload.oop_river_bet, &payload.oop_river_raise),
            bet_sizes(&payload.ip_river_bet, &payload.ip_river_raise),
        ],
        add_all_in_threshold: payload.add_all_in_threshold,
        force_all_in_threshold: payload.force_all_in_threshold,
        adjust_last_two_bet_sizes: payload.adjust_last_two_bet_sizes,
    };

    game.update_config(&config).err()
}

#[tauri::command]
fn game_private_hand_cards(
    player: usize,
    game_state: tauri::State<Mutex<PostFlopGame>>,
) -> Vec<(u8, u8)> {
    let game = game_state.lock().unwrap();
    game.private_hand_cards(player).to_vec()
}

#[tauri::command]
fn game_memory_usage(game_state: tauri::State<Mutex<PostFlopGame>>) -> (u64, u64) {
    let game = game_state.lock().unwrap();
    game.memory_usage()
}

#[tauri::command]
fn game_allocate_memory(enable_compression: bool, game_state: tauri::State<Mutex<PostFlopGame>>) {
    let mut game = game_state.lock().unwrap();
    game.allocate_memory(enable_compression);
}

#[tauri::command(async)]
fn game_solve_step(
    current_iteration: u32,
    game_state: tauri::State<Mutex<PostFlopGame>>,
    pool_state: tauri::State<Mutex<PoolManager>>,
) {
    let game = game_state.lock().unwrap();
    let pool_manager = pool_state.lock().unwrap();
    pool_manager
        .pool
        .install(|| solve_step(&*game, current_iteration));
}

#[tauri::command(async)]
fn game_exploitability(
    game_state: tauri::State<Mutex<PostFlopGame>>,
    pool_state: tauri::State<Mutex<PoolManager>>,
) -> f32 {
    let game = game_state.lock().unwrap();
    let pool_manager = pool_state.lock().unwrap();
    pool_manager.pool.install(|| compute_exploitability(&*game))
}

#[tauri::command(async)]
fn game_finalize(
    game_state: tauri::State<Mutex<PostFlopGame>>,
    pool_state: tauri::State<Mutex<PoolManager>>,
) {
    let pool_manager = pool_state.lock().unwrap();
    pool_manager
        .pool
        .install(|| finalize(&mut *game_state.lock().unwrap()));
    game_state.lock().unwrap().cache_normalized_weights();
}

#[tauri::command]
fn game_apply_history(history: Vec<u32>, game_state: tauri::State<Mutex<PostFlopGame>>) {
    let mut game = game_state.lock().unwrap();
    game.back_to_root();
    for &action in &history {
        game.play(action as usize);
    }
    game.cache_normalized_weights();
}

#[tauri::command]
fn game_available_actions(game_state: tauri::State<Mutex<PostFlopGame>>) -> Vec<String> {
    let game = game_state.lock().unwrap();
    if game.is_chance_node() {
        vec!["Chance".to_string()]
    } else {
        game.available_actions()
            .iter()
            .map(|&x| match x {
                Action::Fold => "Fold".to_string(),
                Action::Check => "Check".to_string(),
                Action::Call => "Call".to_string(),
                Action::Bet(size) => format!("Bet {}", size),
                Action::Raise(size) => format!("Raise {}", size),
                Action::AllIn(size) => format!("All-in {}", size),
                _ => unreachable!(),
            })
            .collect()
    }
}

#[tauri::command]
fn game_is_terminal_action(game_state: tauri::State<Mutex<PostFlopGame>>) -> u32 {
    let game = game_state.lock().unwrap();
    game.is_terminal_action()
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &x)| acc | (x as u32) << i)
}

#[tauri::command]
fn game_possible_cards(game_state: tauri::State<Mutex<PostFlopGame>>) -> u64 {
    let game = game_state.lock().unwrap();
    game.possible_cards()
}

#[tauri::command]
fn game_current_player(game_state: tauri::State<Mutex<PostFlopGame>>) -> usize {
    let game = game_state.lock().unwrap();
    game.current_player()
}

#[derive(serde::Serialize)]
struct GameResultsResponse {
    weights: Vec<f32>,
    weights_normalized: Vec<f64>,
    expected_values: Vec<f32>,
    equity: Vec<f32>,
    strategy: Vec<f32>,
}

#[tauri::command]
fn game_results(game_state: tauri::State<Mutex<PostFlopGame>>) -> GameResultsResponse {
    let game = game_state.lock().unwrap();
    let player = game.current_player();

    GameResultsResponse {
        weights: game.weights(player).to_vec(),
        weights_normalized: game.normalized_weights(player).to_vec(),
        expected_values: game.expected_values(),
        equity: game.equity(),
        strategy: game.strategy(),
    }
}
