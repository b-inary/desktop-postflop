#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use postflop_solver::*;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Mutex;
use sysinfo::{System, SystemExt};

#[derive(Default)]
struct RangeManager([Range; 2]);

struct GameManager {
    pool: rayon::ThreadPool,
    game: PostFlopGame,
    node: AtomicPtr<PostFlopNode>,
    board: Vec<u8>,
    turn_swapped_suit: Option<(u8, u8)>,
    turn_swap: [Vec<(usize, usize)>; 2],
    river_swap: [Vec<(usize, usize)>; 2],
    weights: [Vec<f32>; 2],
    weights_normalized: [Vec<f32>; 2],
}

impl Default for GameManager {
    fn default() -> Self {
        Self {
            pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
            game: PostFlopGame::new(),
            node: Default::default(),
            board: Vec::new(),
            turn_swapped_suit: None,
            turn_swap: Default::default(),
            river_swap: Default::default(),
            weights: Default::default(),
            weights_normalized: Default::default(),
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(RangeManager::default()))
        .manage(Mutex::new(GameManager::default()))
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
            game_actions,
            game_is_terminal_action,
            game_is_possible_chance,
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
    game_state: tauri::State<Mutex<GameManager>>,
) -> Option<String> {
    let ranges = &range_state.lock().unwrap().0;
    let mut game_manager = game_state.lock().unwrap();

    game_manager.pool = rayon::ThreadPoolBuilder::new()
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

    game_manager.game.update_config(&config).err()
}

#[tauri::command]
fn game_private_hand_cards(
    player: usize,
    game_state: tauri::State<Mutex<GameManager>>,
) -> Vec<(u8, u8)> {
    let game_manager = game_state.lock().unwrap();
    game_manager.game.private_hand_cards(player).to_vec()
}

#[tauri::command]
fn game_memory_usage(game_state: tauri::State<Mutex<GameManager>>) -> (u64, u64) {
    let game_manager = game_state.lock().unwrap();
    game_manager.game.memory_usage()
}

#[tauri::command]
fn game_allocate_memory(enable_compression: bool, game_state: tauri::State<Mutex<GameManager>>) {
    let mut game_manager = game_state.lock().unwrap();
    game_manager.game.allocate_memory(enable_compression);
}

#[tauri::command]
async fn game_solve_step(
    current_iteration: u32,
    game_state: tauri::State<'_, Mutex<GameManager>>,
) -> Result<(), String> {
    let game_manager = game_state.lock().unwrap();
    game_manager
        .pool
        .install(|| solve_step(&game_manager.game, current_iteration));
    Ok(())
}

#[tauri::command]
async fn game_exploitability(
    game_state: tauri::State<'_, Mutex<GameManager>>,
) -> Result<f32, String> {
    let game_manager = game_state.lock().unwrap();
    Ok(game_manager
        .pool
        .install(|| compute_exploitability(&game_manager.game, false)))
}

#[tauri::command]
async fn game_finalize(game_state: tauri::State<'_, Mutex<GameManager>>) -> Result<f64, String> {
    let mut game_manager = game_state.lock().unwrap();

    game_manager.pool.install(|| {
        normalize_strategy(&game_manager.game);
        compute_ev_and_equity(&game_manager.game);
    });

    game_manager.node = AtomicPtr::new(&mut *game_manager.game.root() as *mut PostFlopNode);
    game_manager.board = game_manager.game.config().flop.to_vec();
    let turn = game_manager.game.config().turn;
    let river = game_manager.game.config().river;
    if turn != NOT_DEALT {
        game_manager.board.push(turn);
    }
    if river != NOT_DEALT {
        game_manager.board.push(river);
    }
    game_manager.turn_swapped_suit = None;
    game_manager.turn_swap = Default::default();
    game_manager.river_swap = Default::default();
    game_manager.weights = [
        game_manager.game.initial_weight(0).to_vec(),
        game_manager.game.initial_weight(1).to_vec(),
    ];
    game_manager.compute_normalized_weight();
    Ok(game_manager.weights_normalized[0]
        .iter()
        .fold(0.0, |acc, &x| acc + x as f64))
}

#[tauri::command]
fn game_apply_history(history: Vec<u32>, game_state: tauri::State<Mutex<GameManager>>) {
    let mut game_manager = game_state.lock().unwrap();
    game_manager.node = AtomicPtr::new(&mut *game_manager.game.root() as *mut PostFlopNode);
    game_manager.board = game_manager.game.config().flop.to_vec();
    let turn = game_manager.game.config().turn;
    let river = game_manager.game.config().river;
    if turn != NOT_DEALT {
        game_manager.board.push(turn);
    }
    if river != NOT_DEALT {
        game_manager.board.push(river);
    }
    game_manager.turn_swapped_suit = None;
    game_manager.turn_swap = Default::default();
    game_manager.river_swap = Default::default();
    game_manager.weights = [
        vec![1.0; game_manager.game.num_private_hands(0)],
        vec![1.0; game_manager.game.num_private_hands(1)],
    ];
    game_manager.apply_history_recursive(&history, true);
    game_manager.compute_normalized_weight();
}

#[tauri::command]
fn game_actions(game_state: tauri::State<Mutex<GameManager>>) -> Vec<String> {
    let game_manager = game_state.lock().unwrap();
    game_manager
        .node()
        .get_actions()
        .iter()
        .map(|&x| match x {
            Action::None => unreachable!(),
            Action::Fold => "Fold".to_string(),
            Action::Check => "Check".to_string(),
            Action::Call => "Call".to_string(),
            Action::Bet(size) => format!("Bet {}", size),
            Action::Raise(size) => format!("Raise {}", size),
            Action::AllIn(size) => format!("All-in {}", size),
            Action::Chance(_) => format!("Chance"),
        })
        .collect::<Vec<_>>()
}

#[tauri::command]
fn game_is_terminal_action(game_state: tauri::State<Mutex<GameManager>>) -> Vec<bool> {
    let game_manager = game_state.lock().unwrap();
    let node = game_manager.node();
    node.actions()
        .map(|action| {
            let child = node.play(action);
            child.is_terminal() || child.amount() == game_manager.game.config().effective_stack
        })
        .collect()
}

#[tauri::command]
fn game_is_possible_chance(game_state: tauri::State<Mutex<GameManager>>) -> Vec<bool> {
    let game_manager = game_state.lock().unwrap();

    let mut mask: u64 = (1 << 52) - 1;
    let board_mask: u64 = game_manager
        .board
        .iter()
        .fold(0, |acc, &card| acc | 1 << card);

    let private_hand_cards = [
        game_manager.game.private_hand_cards(0),
        game_manager.game.private_hand_cards(1),
    ];

    for (i, &(c1, c2)) in private_hand_cards[0].iter().enumerate() {
        let oop_mask: u64 = (1 << c1) | (1 << c2);
        let oop_weight = game_manager.weights[0][i];
        if board_mask & oop_mask == 0 && oop_weight >= 0.05 / 100.0 {
            for (j, &(c3, c4)) in private_hand_cards[1].iter().enumerate() {
                let ip_mask: u64 = (1 << c3) | (1 << c4);
                let ip_weight = game_manager.weights[1][j];
                if (board_mask | oop_mask) & ip_mask == 0 && ip_weight >= 0.05 / 100.0 {
                    mask &= board_mask | oop_mask | ip_mask;
                }
            }
            if mask == board_mask {
                break;
            }
        }
    }

    (0..52).map(|card| (mask & (1 << card)) == 0).collect()
}

#[tauri::command]
fn game_current_player(game_state: tauri::State<Mutex<GameManager>>) -> usize {
    let game_manager = game_state.lock().unwrap();
    game_manager.node().player()
}

#[tauri::command]
fn game_results(game_state: tauri::State<Mutex<GameManager>>) -> Vec<f32> {
    let game_manager = game_state.lock().unwrap();
    let player = game_manager.node().player();
    game_manager.weights[player]
        .iter()
        .chain(game_manager.weights_normalized[player].iter())
        .cloned()
        .chain(game_manager.get_expected_values().into_iter())
        .chain(game_manager.get_equity().into_iter())
        .chain(game_manager.get_strategy().into_iter())
        .collect()
}

impl GameManager {
    fn node(&self) -> &PostFlopNode {
        unsafe { &*self.node.load(Ordering::Relaxed) }
    }

    fn compute_normalized_weight(&mut self) {
        self.weights_normalized = [
            vec![0.0; self.game.num_private_hands(0)],
            vec![0.0; self.game.num_private_hands(1)],
        ];

        let private_hand_cards = [
            self.game.private_hand_cards(0),
            self.game.private_hand_cards(1),
        ];

        let board_mask: u64 = self.board.iter().fold(0, |acc, &card| acc | 1 << card);

        for (i, &(c1, c2)) in private_hand_cards[0].iter().enumerate() {
            let oop_mask: u64 = (1 << c1) | (1 << c2);
            let oop_weight = self.weights[0][i];
            if board_mask & oop_mask == 0 && oop_weight > 0.0 {
                for (j, &(c3, c4)) in private_hand_cards[1].iter().enumerate() {
                    let ip_mask: u64 = (1 << c3) | (1 << c4);
                    let ip_weight = self.weights[1][j];
                    if (board_mask | oop_mask) & ip_mask == 0 && ip_weight > 0.0 {
                        let weight = oop_weight * ip_weight;
                        self.weights_normalized[0][i] += weight;
                        self.weights_normalized[1][j] += weight;
                    }
                }
            }
        }
    }

    fn apply_history_recursive(&mut self, history: &[u32], is_root: bool) {
        if history.is_empty() {
            if is_root {
                mul_slice(&mut self.weights[0], &self.game.initial_weight(0));
                mul_slice(&mut self.weights[1], &self.game.initial_weight(1));
            }
            return;
        }

        let node = unsafe { &*self.node.load(Ordering::Relaxed) };
        let action = history[0] as usize;

        if node.is_chance() {
            let is_turn = self.board.len() == 3;

            let card = if let Some((suit1, suit2)) = self.turn_swapped_suit {
                if action as u8 & 3 == suit1 {
                    action as u8 - suit1 + suit2
                } else if action as u8 & 3 == suit2 {
                    action as u8 + suit1 - suit2
                } else {
                    action as u8
                }
            } else {
                action as u8
            };

            let mut index = usize::MAX;
            let mut isomorphic_index = usize::MAX;

            let actions = node.get_actions();
            for i in 0..actions.len() {
                if actions[i] == Action::Chance(card) {
                    index = i;
                    break;
                }
            }

            if index == usize::MAX {
                let isomorphism = self.game.isomorphic_chances(node);
                let isomorphic_card = self.game.isomorphic_card(node);
                for i in 0..isomorphism.len() {
                    if isomorphic_card[i] == card {
                        index = isomorphism[i];
                        isomorphic_index = i;
                        if self.board.len() == 3 {
                            if let Action::Chance(card2) = actions[index] {
                                self.turn_swapped_suit = Some((card & 3, card2 & 3));
                            }
                        }
                        break;
                    }
                }
            }

            self.node = AtomicPtr::new(&mut *node.play(index));
            self.board.push(action as u8);
            self.apply_history_recursive(&history[1..], false);

            if isomorphic_index != usize::MAX {
                let swap_list = self.game.isomorphic_swap(node, isomorphic_index);
                for player in 0..2 {
                    for &(i, j) in &swap_list[player] {
                        self.weights[player].swap(i, j);
                    }
                }
                if is_turn {
                    self.turn_swap = swap_list.clone();
                } else {
                    self.river_swap = swap_list.clone();
                }
            }
        } else {
            self.node = AtomicPtr::new(&mut *node.play(action));
            self.apply_history_recursive(&history[1..], false);

            if node.num_actions() > 1 {
                let player = node.player();
                if !self.game.is_compression_enabled() {
                    mul_slice(
                        &mut self.weights[player],
                        row(
                            node.strategy(),
                            action as usize,
                            self.game.num_private_hands(player),
                        ),
                    );
                } else {
                    let decoder = node.strategy_scale() / u16::MAX as f32;
                    self.weights[player]
                        .iter_mut()
                        .zip(row(
                            node.strategy_compressed(),
                            action as usize,
                            self.game.num_private_hands(player),
                        ))
                        .for_each(|(w, &s)| *w *= s as f32 * decoder);
                }
            }
        }

        if is_root {
            mul_slice(&mut self.weights[0], &self.game.initial_weight(0));
            mul_slice(&mut self.weights[1], &self.game.initial_weight(1));
        }
    }

    fn get_expected_values(&self) -> Vec<f32> {
        let node = self.node();
        let player = node.player();
        let num_private_hands = self.game.num_private_hands(player);
        let mut ret = if !self.game.is_compression_enabled() {
            node.cum_regret()
                .iter()
                .take(num_private_hands)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            let decoder = node.cum_regret_scale() / i16::MAX as f32;
            node.cum_regret_compressed()
                .iter()
                .take(num_private_hands)
                .map(|&x| x as f32 * decoder)
                .collect::<Vec<_>>()
        };
        for swap in [&self.river_swap, &self.turn_swap] {
            for &(i, j) in &swap[player] {
                ret.swap(i, j);
            }
        }
        ret
    }

    fn get_equity(&self) -> Vec<f32> {
        let node = self.node();
        let player = node.player();
        let num_actions = node.num_actions();
        let num_private_hands = self.game.num_private_hands(player);
        let mut ret = if !self.game.is_compression_enabled() {
            if num_actions == 1 {
                node.strategy().to_vec()
            } else {
                row(node.cum_regret(), 1, num_private_hands).to_vec()
            }
        } else {
            let decoder = node.equity_scale() / i16::MAX as f32;
            if num_actions == 1 {
                node.strategy_compressed()
                    .iter()
                    .map(|&x| x as i16 as f32 * decoder)
                    .collect()
            } else {
                row(node.cum_regret_compressed(), 1, num_private_hands)
                    .iter()
                    .map(|&x| x as f32 * decoder)
                    .collect()
            }
        };
        for swap in [&self.river_swap, &self.turn_swap] {
            for &(i, j) in &swap[player] {
                ret.swap(i, j);
            }
        }
        ret
    }

    fn get_strategy(&self) -> Vec<f32> {
        let node = self.node();
        let player = node.player();
        let num_actions = node.num_actions();
        let num_private_hands = self.game.num_private_hands(player);
        let mut ret = if num_actions == 1 {
            vec![1.0; num_private_hands]
        } else if !self.game.is_compression_enabled() {
            node.strategy().to_vec()
        } else {
            let decoder = node.strategy_scale() / u16::MAX as f32;
            node.strategy_compressed()
                .iter()
                .map(|&x| x as f32 * decoder)
                .collect::<Vec<_>>()
        };
        for swap in [&self.river_swap, &self.turn_swap] {
            for &(i, j) in &swap[player] {
                for action in 0..num_actions {
                    ret.swap(
                        action * num_private_hands + i,
                        action * num_private_hands + j,
                    );
                }
            }
        }
        ret
    }
}
