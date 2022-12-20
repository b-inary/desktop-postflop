use postflop_solver::*;
use std::cmp::Ordering;
use std::sync::Mutex;

#[derive(Default)]
pub struct RangeManager(pub [Range; 2]);

#[tauri::command]
pub fn range_clear(range_state: tauri::State<Mutex<RangeManager>>, player: usize) {
    let range = &mut (range_state.lock().unwrap().0)[player];
    range.clear();
}

#[tauri::command]
pub fn range_update(
    range_state: tauri::State<Mutex<RangeManager>>,
    player: usize,
    row: u8,
    col: u8,
    weight: f32,
) {
    let range = &mut (range_state.lock().unwrap().0)[player];
    let rank1 = 13 - row;
    let rank2 = 13 - col;
    match row.cmp(&col) {
        Ordering::Equal => range.set_weight_pair(rank1, weight),
        Ordering::Less => range.set_weight_suited(rank1, rank2, weight),
        Ordering::Greater => range.set_weight_offsuit(rank1, rank2, weight),
    }
}

#[tauri::command]
pub fn range_from_string(
    range_state: tauri::State<Mutex<RangeManager>>,
    player: usize,
    str: String,
) -> Option<String> {
    let range = &mut (range_state.lock().unwrap().0)[player];
    let result = Range::from_sanitized_str(str.as_str());
    if let Ok(unwrap) = result {
        *range = unwrap;
        None
    } else {
        result.err()
    }
}

#[tauri::command]
pub fn range_to_string(range_state: tauri::State<Mutex<RangeManager>>, player: usize) -> String {
    let range = &(range_state.lock().unwrap().0)[player];
    range.to_string()
}

#[tauri::command]
pub fn range_get_weights(
    range_state: tauri::State<Mutex<RangeManager>>,
    player: usize,
) -> Vec<f32> {
    let range = &(range_state.lock().unwrap().0)[player];
    let mut weights = vec![0.0; 13 * 13];

    for row in 0..13 {
        for col in 0..13 {
            let rank1 = 12 - row as u8;
            let rank2 = 12 - col as u8;
            weights[row * 13 + col] = match row.cmp(&col) {
                Ordering::Equal => range.get_weight_pair(rank1),
                Ordering::Less => range.get_weight_suited(rank1, rank2),
                Ordering::Greater => range.get_weight_offsuit(rank1, rank2),
            };
        }
    }

    weights
}

#[tauri::command]
pub fn range_raw_data(range_state: tauri::State<Mutex<RangeManager>>, player: usize) -> Vec<f32> {
    let range = &(range_state.lock().unwrap().0)[player];
    range.raw_data().to_vec()
}
