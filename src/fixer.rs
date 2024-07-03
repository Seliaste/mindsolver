use std::collections::HashMap;

use crate::classification::ColorPoint;
use crate::constants::CENTRE_INDICES;
use itertools::Itertools;
use kewb::CubieCube;
use kewb::FaceCube;
use paris::log;

/// Calculates the score for a given notation based on the closeness to the mean RGB values of facelets.
///
/// # Arguments
/// * `rgb_values` - The previously scanned facelet RGB tuples.
/// * `notation` - The notation string to calculate the score for.
///
/// # Returns
/// The score (f64) for the given notation.
fn calculate_score(rgb_values: &Vec<ColorPoint>, notation: &str) -> f64 {
    // get groups of rgb values
    let chars = notation.chars().collect_vec();
    let mut groups = HashMap::new();
    for (i, color) in rgb_values.iter().enumerate() {
        let facelet = chars[i];
        groups.entry(facelet).or_insert(vec![]).push(color);
    }
    // calculate the sum of mahlanobis distances
    let mut score = 0.;
    for group in groups.values() {
        let mean = group
            .iter()
            .fold([0., 0., 0.], |acc, x| {
                [acc[0] + x.r, acc[1] + x.g, acc[2] + x.b]
            })
            .map(|x| x / group.len() as f64);
        for color in group {
            let distance = (0..3)
                .map(|i| (color.to_array()[i] - mean[i]).powi(2))
                .sum::<f64>()
                .sqrt();
            score += distance;
        }
    }
    score
}

/// Generates all possible swap options for the given characters,
/// excluding the centre facelets and useless swaps of same characters.
///
/// # Arguments
/// * `chars` - The characters to generate swap options for.
///
/// # Returns
/// A vector of vectors, where each inner vector represents a swap option.
fn generate_swap_options(chars: &Vec<char>) -> Vec<Vec<usize>> {
    return (0..54)
        .filter(|x| !CENTRE_INDICES.contains(x))
        .combinations(2)
        .filter(|x| &chars[x[0]] != &chars[x[1]])
        .collect_vec();
}

fn generate_swap_weights(rgb_values: &Vec<ColorPoint>, notation: &str ,swaps: &Vec<Vec<usize>>) -> Vec<i32> {
    let mut weights = vec![];
    for swap in swaps{
        let mut chars = notation.chars().collect_vec();
        let (i, j) = (swap[0], swap[1]);
        chars.swap(i, j);
        weights.push((calculate_score(rgb_values, chars.iter().collect::<String>().as_str())*100.) as i32);
    }
    weights
}

/// Applies the given swaps to the characters and returns the resulting string.
///
/// # Arguments
/// * `chars` - The characters to apply the swaps to.
/// * `swaps` - The swaps to apply.
///
/// # Returns
/// The resulting string after applying the swaps.
fn apply_swaps(chars: &Vec<char>, swaps: &Vec<&Vec<usize>>) -> String {
    let mut chars = chars.clone();
    for swap in swaps {
        let (i, j) = (swap[0], swap[1]);
        chars.swap(i, j);
    }
    chars.iter().collect()
}

/// Finds the optimal valid notation for the given (possibly invalid) notation.
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
    let weigts = generate_swap_weights(rgb_values, &nota, &swap_options);
    let quartile_weight = weigts.iter().sorted().nth(weigts.len()/16).unwrap();
    let swap_elagated = swap_options.iter().enumerate().filter(|x| {weigts[x.0] < *quartile_weight}).map(|x| x.1.clone()).collect_vec();
    log!("Optimized swaps #: {}, before: {}", swap_elagated.len(), swap_options.len());
    let mut best_score: (f64, String) = (f64::INFINITY, nota);
    for k in 0..5 {
        log!("Exploring permutations at depth {k}");
        let to_be_tried = swap_elagated.iter().combinations(k);
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
    }
    best_score
}

#[cfg(test)]
mod tests {
    use crate::cube::Cube;
    use crate::fixer;
    use std::fs::read_dir;

    #[test]
    fn official_solved_cube_test() {
        let mut tested = 0;
        let mut correct = 0;
        'outer: for entry in read_dir("scan_test_files/official_cube_solved").unwrap() {
            if entry.is_err() {
                continue;
            }
            let file = entry.unwrap();
            let mut cube = Cube::init();
            cube.import(file.path().to_str().unwrap().to_string())
                .expect("Could not load scan file");
            tested += 1;
            correct += 1;
            let cube_notation = cube.to_notation();
            let fixed = fixer::find_optimal_fix(&cube.facelet_rgb_values, cube_notation.clone());
            println!("Cube notation is: {}", fixed.1);
            let mut consecutive = 0;
            let mut current = 'U';
            for char in fixed.1.chars() {
                if char == current {
                    consecutive += 1;
                } else if consecutive == 9 {
                    consecutive = 1;
                    current = char;
                } else {
                    correct -= 1;
                    continue 'outer;
                }
            }
        }
        println!("correct : {correct} / {tested}");
        assert_eq!(correct, tested);
    }
}
