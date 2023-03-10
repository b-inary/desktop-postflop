use crate::range::*;
use postflop_solver::*;
use rayon::ThreadPool;
use std::sync::Mutex;

#[tauri::command]
pub fn bunching_init(
    range_state: tauri::State<Mutex<RangeManager>>,
    bunching_state: tauri::State<Mutex<Option<BunchingData>>>,
    board: Vec<u8>,
) -> Option<String> {
    if board.len() < 3 {
        return Some("Board must have at least 3 cards".to_string());
    }

    let ranges = &range_state.lock().unwrap().0;
    let bunching_data = BunchingData::new(&ranges[2..], board[..3].try_into().unwrap());

    match bunching_data {
        Ok(bunching_data) => {
            *bunching_state.lock().unwrap() = Some(bunching_data);
            None
        }
        Err(e) => {
            *bunching_state.lock().unwrap() = None;
            Some(e)
        }
    }
}

#[tauri::command]
pub fn bunching_clear(bunching_state: tauri::State<Mutex<Option<BunchingData>>>) {
    *bunching_state.lock().unwrap() = None;
}

#[tauri::command(async)]
pub fn bunching_progress(
    bunching_state: tauri::State<Mutex<Option<BunchingData>>>,
    pool_state: tauri::State<Mutex<ThreadPool>>,
) -> [u8; 2] {
    let mut bunching_data = bunching_state.lock().unwrap();
    let bunching_data = bunching_data.as_mut().unwrap();
    let pool = pool_state.lock().unwrap();

    let phase = bunching_data.phase();
    let percent = bunching_data.progress_percent();
    if phase == 3 && percent == 100 {
        return [3, 100];
    }

    pool.install(|| {
        if phase == 0 {
            bunching_data.phase1_prepare();
        } else if phase == 1 {
            if percent < 100 {
                bunching_data.phase1_proceed_by_percent();
            } else {
                bunching_data.phase2_prepare();
            }
        } else if phase == 2 {
            if percent < 100 {
                bunching_data.phase2_proceed_by_percent();
            } else {
                bunching_data.phase3_prepare();
            }
        } else if phase == 3 {
            bunching_data.phase3_proceed_by_percent();
        }
    });

    if phase == 0 || percent == 100 {
        [phase + 1, 0]
    } else {
        [phase, percent + 1]
    }
}
