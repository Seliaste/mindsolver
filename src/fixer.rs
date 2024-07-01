use crate::classification::ColorPoint;
use itertools::Itertools;
use kewb::CubieCube;
use kewb::FaceCube;

fn scoring(rgb_values: &Vec<ColorPoint>, notation: &str) -> f64 {
    let char_vec = notation.chars().collect_vec();
    let centre_position: Vec<usize> = vec![4, 22, 31, 49, 13, 40];
    let mut score = 0.0;
    for centre in centre_position {
        let letter = char_vec[centre];
        let facelet = rgb_values.get(centre).unwrap();
        for (i, char) in char_vec.iter().enumerate() {
            if i == centre {
                continue;
            }
            if char.clone() == letter {
                let facelet2 = rgb_values.get(i).unwrap();
                score += facelet.distance_to(facelet2);
            }
        }
    }
    return score;
}

pub fn fixer(rgb_values: &Vec<ColorPoint>, nota: String) -> (f64, String) {
    const BANNED: [usize; 6] = [4, 22, 31, 49, 13, 40];
    let chars = nota.chars().collect_vec();
    let swap_options = (0..54)
        .filter(|x| !BANNED.contains(x))
        .combinations(2)
        .filter(|x| &chars[x[0]] != &chars[x[1]])
        .collect_vec();
    let mut min = (f64::INFINITY, nota);
    for k in 0..4 {
        println!("Exploring depth {k}");
        let to_be_tried = swap_options.iter().combinations(k);
        for option in to_be_tried {
            let mut try_nota = chars.clone();
            for permutation in option {
                (try_nota[permutation[0]], try_nota[permutation[1]]) =
                    (try_nota[permutation[1]], try_nota[permutation[0]])
            }
            let string = try_nota.iter().collect::<String>();
            let facecube_option = FaceCube::try_from(string.as_str());
            if !facecube_option.is_err() {
                let x = CubieCube::try_from(&facecube_option.unwrap());
                if !x.is_err() && x.unwrap().is_solvable() {
                    let score = scoring(rgb_values, string.as_str());
                    if score < min.0 {
                        min = (score, string);
                    }
                }
            }
        }
        if min.0 < f64::INFINITY {
            break;
        }
    }
    min
}
