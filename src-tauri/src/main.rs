#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod bunching;
mod range;
mod solver;
mod tree;
use crate::bunching::*;
use crate::range::*;
use crate::solver::*;
use crate::tree::*;

use postflop_solver::*;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::sync::Mutex;
use sysinfo::{System, SystemExt};

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(RangeManager::default()))
        .manage(Mutex::new(default_action_tree()))
        .manage(Mutex::new(None as Option<BunchingData>))
        .manage(Mutex::new(PostFlopGame::default()))
        .manage(Mutex::new(ThreadPoolBuilder::new().build().unwrap()))
        .invoke_handler(tauri::generate_handler![
            os_name,
            memory,
            set_num_threads,
            range_num_combos,
            range_clear,
            range_invert,
            range_update,
            range_from_string,
            range_to_string,
            range_get_weights,
            range_raw_data,
            tree_new,
            tree_added_lines,
            tree_removed_lines,
            tree_invalid_terminals,
            tree_actions,
            tree_is_terminal_node,
            tree_is_chance_node,
            tree_back_to_root,
            tree_apply_history,
            tree_play,
            tree_total_bet_amount,
            tree_add_bet_action,
            tree_remove_current_node,
            tree_delete_added_line,
            tree_delete_removed_line,
            bunching_init,
            bunching_clear,
            bunching_progress,
            game_init,
            game_private_cards,
            game_memory_usage,
            game_memory_usage_bunching,
            game_allocate_memory,
            game_set_bunching,
            game_solve_step,
            game_exploitability,
            game_finalize,
            game_apply_history,
            game_total_bet_amount,
            game_actions_after,
            game_possible_cards,
            game_get_results,
            game_get_chance_reports
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn os_name() -> String {
    "windows".to_string()
}

#[cfg(target_os = "macos")]
#[tauri::command]
fn os_name() -> String {
    "macos".to_string()
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn os_name() -> String {
    "linux".to_string()
}

#[tauri::command]
fn memory() -> (u64, u64) {
    let mut system = System::new_all();
    system.refresh_memory();
    (system.available_memory(), system.total_memory())
}

#[tauri::command]
fn set_num_threads(pool_state: tauri::State<Mutex<ThreadPool>>, num_threads: usize) {
    *pool_state.lock().unwrap() = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();
}
