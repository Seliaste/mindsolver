use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::{fs, iter};

use kewb::fs::read_table;
use kewb::{CubieCube, FaceCube, Solution, Solver};
use paris::info;

use crate::classification::{Classification, Point};
use crate::constants::{CORNER_FACELET, EDGE_FACELET, SIDE_INDEXES};

/// Represents the cube faces and state
pub struct Cube {
    // Current facelet number
    pub curr_idx: usize,
    /// Stores RGB values in the order of the standard notation
    pub facelet_rgb_values: Vec<Point>,
    /// Faces that can be accessed by simply flipping. First one is the one currently down
    pub next_faces: [char; 4],
    // right and left from the sensor POV
    pub right_face: char,
    pub left_face: char,
}
impl Cube {
    pub fn init() -> Self {
        Self {
            curr_idx: 0,
            facelet_rgb_values: iter::repeat(Point {
                x: 0.,
                y: 0.,
                z: 0.,
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
        for key in res_sides.keys() {
            let face_char = centre_to_face.get(&key.index).unwrap().clone();
            string[key.index] = face_char;
            for point in res_sides.get(key).unwrap() {
                string[point.1.index] = face_char;
            }
        }
        for key in res_corners.keys() {
            let face_char = centre_to_face.get(&key.index).unwrap().clone();
            string[key.index] = face_char;
            for point in res_corners.get(key).unwrap() {
                string[point.1.index] = face_char;
            }
        }
        string.iter().collect()
    }

    pub fn solve(&self) -> Solution {
        let table = read_table("./cache_file").unwrap();
        let mut solver = Solver::new(&table, 23, Some(5.));
        let face_cube = FaceCube::try_from(self.to_notation().as_str())
            .expect("Could not convert string to faces");
        let state = CubieCube::try_from(&face_cube).expect("Invalid cube");
        return solver.solve(state).expect("Could not solve cube");
    }

    /// Saves the scan to file. Used for debugging
    pub fn export(&self) {
        fs::create_dir_all("scans").ok();
        let mut file = File::create(format!(
            "scans/{}",
            chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S")
        ))
        .unwrap();
        let mut string = String::new();
        for point in self.facelet_rgb_values.iter().map(Point::export) {
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
            self.facelet_rgb_values[pos] = Point {
                x: rgb[0],
                y: rgb[1],
                z: rgb[2],
                index: pos,
            };
        }
        info!("Loaded scan from file");
        Ok(())
    }

    pub fn print_facelets(&self) {
        let notation: Vec<char> = self.to_notation().chars().collect();
        let mut corners = String::new();
        let mut edges = String::new();
        for corner in CORNER_FACELET {
            corners += format!("[{},{},{}],",notation[corner[0]],notation[corner[1]],notation[corner[2]]).as_str();
        }
        for edge in EDGE_FACELET {
            edges += format!("[{},{}],",notation[edge[0]],notation[edge[1]]).as_str();
        }
        info!("Edges are: {edges}");
        info!("Corners are: {corners}");
    }
}
