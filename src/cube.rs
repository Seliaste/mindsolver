use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::{char, fs, iter};

use colored::Colorize;
use itertools::Itertools;
use kewb::fs::read_table;
use kewb::{CubieCube, FaceCube, Solution, Solver};
use paris::info;

use crate::classification::{Classification, ColorPoint};
use crate::constants::SIDE_INDEXES;

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

    pub fn print_graphical(nota: &str) {
        fn print_letter(idx: usize, chars: &Vec<char>) {
            let colors: HashMap<char, [u8; 3]> = HashMap::from([
                ('U', [255, 255, 255]),
                ('R', [0, 0, 255]),
                ('F', [255, 0, 0]),
                ('D', [255, 255, 0]),
                ('L', [0, 255, 0]),
                ('B', [255, 165, 0]),
            ]);
            let letter = chars[idx];
            let color = colors.get(&letter).unwrap();
            print!(
                "{}",
                letter
                    .to_string()
                    .as_str()
                    .truecolor(color[0], color[1], color[2])
            );
        }

        let chars = nota.chars().collect_vec();
        // up
        for i in 0..3 {
            print!("    ");
            for j in 0..3 {
                print_letter(3 * i + j, &chars);
            }
            println!();
        }
        println!();
        // left, front, right, back
        for i in 0..3 {
            for j in 0..3 {
                print_letter(36 + 3 * i + j, &chars)
            }
            print!(" ");
            for j in 0..3 {
                print_letter(18 + 3 * i + j, &chars);
            }
            print!(" ");
            for j in 0..3 {
                print_letter(9 + 3 * i + j, &chars);
            }
            print!(" ");
            for j in 0..3 {
                print_letter(45 + 3 * i + j, &chars);
            }
            println!();
        }
        println!();
        // down
        for i in 0..3 {
            print!("    ");
            for j in 0..3 {
                print_letter(27 + 3 * i + j, &chars);
            }
            println!();
        }
    }

    pub fn objective(&self, notation: String) -> f64 {
        let char_vec = notation.chars().collect_vec();
        let centre_position: Vec<usize> = vec![4, 22, 31, 49, 13, 40];
        let mut score = 0.0;
        for centre in centre_position {
            let letter = char_vec[centre];
            let facelet = self.facelet_rgb_values.get(centre).unwrap();
            for (i, char) in char_vec.iter().enumerate() {
                if i == centre {
                    continue;
                }
                if char.clone() == letter {
                    let facelet2 = self.facelet_rgb_values.get(i).unwrap();
                    score += facelet.distance_to(facelet2);
                }
            }
        }
        return score;
    }

    #[allow(dead_code)]
    pub fn fixer(&self, nota: String) -> (f64, String) {
        const BANNED: [usize; 6] = [4, 22, 31, 49, 13, 40];
        let chars = nota.chars().collect_vec();
        let charclone = chars.clone();
        let swap_options = (0..54)
            .filter(|x| !BANNED.contains(x))
            .combinations(2)
            .filter(move |x| charclone[x[0]] != charclone[x[1]])
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
                let facecube_option =
                    FaceCube::try_from(try_nota.iter().collect::<String>().as_str());
                if !facecube_option.is_err() {
                    let x = CubieCube::try_from(&facecube_option.unwrap());
                    if !x.is_err() && x.unwrap().is_solvable() {
                        let score = self.objective(try_nota.iter().collect::<String>());
                        if score < min.0 {
                            min = (score, try_nota.iter().collect::<String>());
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
}