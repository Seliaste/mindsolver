use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use std::{fs, iter};

use itertools::Itertools;
use kewb::fs::read_table;
use kewb::{CubieCube, FaceCube, Solution, Solver};
use paris::{error, info};

use crate::classification::{Classification, ColorPoint};
use crate::constants::{get_corner_colors, CORNER_FACELET, SIDE_INDEXES};

/// Represents the cube faces and state
pub struct Cube {
    // Current facelet number
    pub curr_idx: usize,
    /// Stores RGB values in the order of the standard notation
    pub facelet_rgb_values: Vec<ColorPoint>,
    /// Faces that can be accessed by simply flipping. First one is the one currently down
    pub next_faces: [char; 4],
    /// right from the sensor POV
    pub right_face: char,
    /// left from the sensor POV
    pub left_face: char,
}

impl Cube {
    pub fn init() -> Self {
        Self {
            curr_idx: 0,
            facelet_rgb_values: iter::repeat(ColorPoint {
                r: 0.,
                g: 0.,
                b: 0.,
                index: 0,
            })
            .take(54)
            .collect(),
            next_faces: ['R', 'D', 'L', 'U'],
            right_face: 'B',
            left_face: 'F',
        }
    }

    /// Converts the cube into the standard notation. A solved cube would be UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
    pub fn to_notation(&self) -> String {
        // we clone so that a function named to_something doesn't have side effects
        let facelets = self.facelet_rgb_values.clone();
        let mut centres = vec![]; // centroids (red points)
        let mut sides = vec![]; // points to classify (black points)
        let mut corners = vec![];
        let centre_to_face: HashMap<usize, char> = HashMap::from([
            (4, 'U'),
            (22, 'F'),
            (31, 'D'),
            (49, 'B'),
            (13, 'R'),
            (40, 'L'),
        ]);
        let centre_index = centre_to_face.keys();
        for centre in centre_index.clone() {
            let face = facelets.get(*centre).unwrap();
            centres.push(face.clone());
        }
        for side in 0..54 {
            if !centre_index.clone().any(|x| x == &side) {
                let face = facelets.get(side).unwrap();
                if SIDE_INDEXES.contains(&face.index) {
                    sides.push(face.clone());
                } else {
                    corners.push(face.clone());
                }
            }
        }
        let mut classification_side = Classification::init(centres.clone(), sides);
        let res_sides = classification_side.classify();
        let mut classification_corners = Classification::init(centres, corners);
        let res_corners = classification_corners.classify();
        let mut string: Vec<char> = iter::repeat(' ').take(54).collect();
        for res in [res_sides, res_corners] {
            for class in res {
                let face_char = centre_to_face.get(&class.0.index).unwrap().clone();
                string[class.0.index] = face_char;
                for point in class.1 {
                    string[point.1.index] = face_char;
                }
            }
        }
        string.iter().collect()
    }

    /// takes a notation and returns a solution
    pub fn solve(notation: String) -> Solution {
        let table = read_table("./cache_file").unwrap();
        let mut solver = Solver::new(&table, 30, None);
        let face_cube =
            FaceCube::try_from(notation.as_str()).expect("Could not convert string to faces");
        let state = CubieCube::try_from(&face_cube).expect("Invalid cube");
        solver.solve(state).expect("Could not solve cube")
    }

    /// Saves the scan to file.
    pub fn export(&self) {
        fs::create_dir_all("scans").ok();
        let mut file = File::create(format!(
            "scans/{}",
            chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S")
        ))
        .unwrap();
        let mut string = String::new();
        for point in self.facelet_rgb_values.iter().map(ColorPoint::to_array) {
            string.push_str(format!("{}, {}, {}\n", point[0], point[1], point[2]).as_str())
        }
        file.write_all(&format!("{}", string).into_bytes()).unwrap();
        info!("Saved scan to file");
    }

    /// Imports a scan from file. Used for debugging
    pub fn import(&mut self, file_path: String) -> std::io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut output = String::new();
        file.read_to_string(&mut output)?;
        for (pos, line) in output.split('\n').enumerate() {
            if line.trim() == "" {
                continue;
            }
            let rgb: Vec<f64> = line
                .split(", ")
                .map(str::parse::<f64>)
                .map(Result::unwrap)
                .collect();
            self.facelet_rgb_values[pos] = ColorPoint {
                r: rgb[0],
                g: rgb[1],
                b: rgb[2],
                index: pos,
            };
        }
        info!("Loaded scan from file");
        Ok(())
    }

    /// Tries fixing and invalid solution. Won't do anything if solution is correct
    pub fn fixer(nota: String, log: bool) -> String {
        let notation: Vec<char> = nota.chars().collect();
        let mut possibilities: Vec<String> = Vec::new();
        let mut corners = Vec::new();
        let mut corners_idx = Vec::new();
        let mut invalid_corners = Vec::new();
        let corner_colors = get_corner_colors();
        for corner in CORNER_FACELET {
            let hashset = HashSet::from([
                notation[corner[0]],
                notation[corner[1]],
                notation[corner[2]],
            ]);
            if !corner_colors.contains(&hashset) {
                invalid_corners.push((
                    [
                        notation[corner[0]],
                        notation[corner[1]],
                        notation[corner[2]],
                    ],
                    [corner[0], corner[1], corner[2]],
                ))
            } else if corners.contains(&hashset) {
                invalid_corners.push((
                    [
                        notation[corner[0]],
                        notation[corner[1]],
                        notation[corner[2]],
                    ],
                    [corner[0], corner[1], corner[2]],
                ))
            } else {
                corners.push(hashset);
                corners_idx.push([corner[0], corner[1], corner[2]])
            }
        }
        let missing_corners: Vec<&HashSet<char>> = corner_colors
            .iter()
            .filter(|x| !corners.contains(*x))
            .collect();
        if log && missing_corners.len() > 0 {
            error!("Missing corners are: {:?}", missing_corners);
        }
        if log && invalid_corners.len() > 0 {
            error!("Invalid corners are: {:?}", invalid_corners);
        }
        // fixing duplicate corners
        for permut in missing_corners
            .to_owned()
            .iter()
            .permutations(missing_corners.len())
        {
            let mut notatmp = notation.clone();
            for (i, corner) in invalid_corners.iter().enumerate() {
                let new = permut[i];
                for character in new.iter() {
                    if !corner.0.contains(character) {
                        for i in 0..3 {
                            if !new.contains(&corner.0[i]) {
                                notatmp[corner.1[i]] = character.clone();
                                break;
                            }
                        }
                    }
                }
            }
            possibilities.push(notatmp.iter().collect());
        }
        for possibility in possibilities {
            let face_cube = FaceCube::try_from(possibility.as_str());
            if face_cube.is_err() {
                continue;
            }
            let state = CubieCube::try_from(&face_cube.unwrap());
            if state.is_err() {
                continue;
            }
            if state.unwrap().is_solvable() {
                return possibility;
            }
        }
        notation.iter().collect()
    }

    pub fn bruteforce_fixer(nota: String) -> String {
        const BANNED: [usize; 6] = [4, 22, 31, 49, 13, 40];
        let chars = nota.chars().collect_vec();
        let swap_options = (0..54).permutations(2);
        for k in 0..5 {
            let to_be_tried = swap_options.clone().permutations(k);
            for option in to_be_tried {
                let mut try_nota = chars.clone();
                for permutation in option {
                    if BANNED.contains(&permutation[0]) && BANNED.contains(&permutation[1]) {
                        continue;
                    }
                    (try_nota[permutation[0]], try_nota[permutation[1]]) =
                        (try_nota[permutation[1]], try_nota[permutation[0]])
                }
                let facecube_option =
                    FaceCube::try_from(try_nota.iter().collect::<String>().as_str());
                if !facecube_option.is_err() {
                    let x = CubieCube::try_from(&facecube_option.unwrap());
                    if !x.is_err() && x.unwrap().is_solvable() {
                        return try_nota.iter().collect::<String>();
                    }
                }
            }
        }
        nota
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use kewb::generators::generate_random_state;
    use kewb::FaceCube;
    use rand::Rng;

    use crate::cube::Cube;

    const BANNED: [usize; 6] = [4, 22, 31, 49, 13, 40];
    #[test]
    fn test_fixer() {
        let mut rng = rand::thread_rng();
        let mut success_counter = 0;
        let mut false_positives = 0;
        let mut no_result = 0;
        for _ in 0..100 {
            let original = FaceCube::try_from(&generate_random_state())
                .unwrap()
                .to_string();
            let mut jammed = original.chars().collect_vec();
            for _ in 0..rng.gen_range(0..5) {
                let i1 = rng.gen_range(0..54);
                let i2 = rng.gen_range(0..54);
                if BANNED.contains(&i1) || BANNED.contains(&i2) {
                    continue;
                }
                (jammed[i1], jammed[i2]) = (jammed[i2], jammed[i1]);
            }
            let jammed_string: String = jammed.into_iter().collect();
            let fixed = Cube::fixer(jammed_string.clone(), false);
            if fixed == original {
                success_counter += 1;
            } else if fixed == jammed_string {
                no_result += 1;
            } else {
                false_positives += 1;
            }
        }
        println!("Fixer managed to fix {success_counter} out of 100 jammed configs ({false_positives} false positives, {no_result} without result)");
    }

    #[test]
    fn test_bruteforce_fixer() {
        let mut rng = rand::thread_rng();
        let mut success_counter = 0;
        let mut false_positives = 0;
        let mut no_result = 0;
        for _ in 0..100 {
            let original = FaceCube::try_from(&generate_random_state())
                .unwrap()
                .to_string();
            let mut jammed = original.chars().collect_vec();
            for _ in 0..rng.gen_range(0..3) {
                let i1 = rng.gen_range(0..54);
                let i2 = rng.gen_range(0..54);
                if BANNED.contains(&i1) || BANNED.contains(&i2) {
                    continue;
                }
                (jammed[i1], jammed[i2]) = (jammed[i2], jammed[i1]);
            }
            let jammed_string: String = jammed.into_iter().collect();
            let fixed = Cube::bruteforce_fixer(jammed_string.clone());
            if fixed == original {
                success_counter += 1;
            } else if fixed == jammed_string {
                no_result += 1;
            } else {
                false_positives += 1;
            }
        }
        println!("Bruteforce fixer managed to fix {success_counter} out of 100 jammed configs ({false_positives} false positives, {no_result} without result)");
    }
}
