use crate::classification::ColorPoint;
use crate::constants::CENTRE_INDICES;
use itertools::Itertools;
use kewb::CubieCube;
use kewb::FaceCube;
use paris::log;

fn calculate_score(rgb_values: &Vec<ColorPoint>, notation: &str) -> f64 {
    let char_vec = notation.chars().collect_vec();
    return CENTRE_INDICES.iter().fold(0.0, |mut acc, &centre| {
        let letter = char_vec[centre];
        let facelet = rgb_values.get(centre).unwrap();
        for (i, _) in char_vec
            .iter()
            .enumerate()
            .filter(|(i, &char)| *i != centre && char.clone() == letter)
        {
            let facelet2 = rgb_values.get(i).unwrap();
            acc += facelet.distance_to(facelet2);
        }
        acc
    });
}

fn generate_swap_options(chars: &Vec<char>) -> Vec<Vec<usize>> {
    return (0..54)
        .filter(|x| !CENTRE_INDICES.contains(x))
        .combinations(2)
        .filter(|x| &chars[x[0]] != &chars[x[1]])
        .collect_vec();
}

fn apply_swaps(chars: &Vec<char>, swaps: &Vec<&Vec<usize>>) -> String {
    let mut chars = chars.clone();
    for swap in swaps {
        let (i, j) = (swap[0], swap[1]);
        chars.swap(i, j);
    }
    chars.iter().collect()
}

/// Finds the optimal valid notation for the given (possibly invalid) notation
///
/// # Arguments
/// * `rgb_values` - The previously scanned facelet RGB tuples.
/// * `nota` - The initial notation string to be fixed.
///
/// # Returns
/// A tuple containing the best score (f64) and its corresponding notation (String).
pub fn find_optimal_fix(rgb_values: &Vec<ColorPoint>, nota: String) -> (f64, String) {
    let chars = nota.chars().collect_vec();
    let swap_options = generate_swap_options(&chars);
    let mut best_score: (f64, String) = (f64::INFINITY, nota);
    for k in 0..4 {
        log!("Exploring permutations at depth {k}");
        let to_be_tried = swap_options.iter().combinations(k);
        for option in to_be_tried {
            let permutted_string = apply_swaps(&chars, &option);
            if let Ok(facecube) = FaceCube::try_from(permutted_string.as_str()) {
                if CubieCube::try_from(&facecube).is_ok() {
                    let score = calculate_score(rgb_values, &permutted_string);
                    if score < best_score.0 {
                        best_score = (score, permutted_string);
                    }
                }
            }
        }
        if best_score.0 < f64::INFINITY {
            break;
        }
    }
    best_score
}
