use paris::info;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::{fs, iter};

use crate::classification::{Classification, Point};

/// Represents the cube faces and state
pub struct Cube {
    // The scan order will always be the same,
    // so instead of complicated code it's better to hardcode it
    pub scan_order: Vec<usize>,
    pub side_indexes: Vec<usize>,
    // Current facelet number
    pub curr_idx: usize,
    /// Stores RGB values in the order of the standard notation
    pub facelet_rgb_values: Vec<Point>,
    pub next_faces: [char; 4], // Faces that can be accessed by simply flipping. First one is the one currently down
    // right and left from the sensor POV
    pub right_face: char,
    pub left_face: char,
}

impl Cube {
    pub fn init() -> Self {
        Self {
            scan_order: vec![
                4, 7, 8, 5, 2, 1, 0, 3, 6, // U
                22, 25, 26, 23, 20, 19, 18, 21, 24, // F
                31, 34, 35, 32, 29, 28, 27, 30, 33, // D
                49, 46, 45, 48, 51, 52, 53, 50, 47, // B
                13, 16, 17, 14, 11, 10, 9, 12, 15, // R
                40, 37, 36, 39, 42, 43, 44, 41, 38, // L
            ],
            side_indexes: vec![
                7, 5, 1, 3, 25, 23, 19, 21, 34, 32, 28, 30, 46, 48, 52, 50, 16, 14, 10, 12, 37, 39,
                43, 41,
            ],
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
                if self.side_indexes.contains(&face.index) {
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
}
