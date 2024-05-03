use std::collections::HashMap;
use std::iter;
use std::process::Command;

use crate::classification::{Classification, Point};

pub struct Cube {
    // The scan order will always be the same,
    // so insted of complicated code it's better to hardcode it
    pub scan_order: Vec<usize>,
    // Current facelet number
    pub curr_idx: usize,
    // Stores RGB values in the order of the standard notatio
    pub facelet_rgb_values: Vec<Point>,
    pub next_faces: [char; 4], // Faces that can be accessed by simply flipping. First one is the one currently down
    // right and left from the sensor POV
    pub right_face: char,
    pub left_face: char,
}

impl Cube {
    pub fn init() -> Self {
        Self {
            // NOTE: THIS NEEDS TO BE VERIFIED, I PROBABLY MADE A MISTAKE
            scan_order: vec![4, 7, 8, 5, 2, 1, 0, 3, 6, // U
                             22, 25, 26, 23, 20, 19, 18, 21, 24, // F
                             31, 34, 35, 32, 29, 28, 27, 30, 33, // D
                             49, 46, 45, 48, 51, 52, 53, 50, 47,// B
                             13, 16, 17, 14, 11, 10, 9, 12, 15, // R
                             40, 37, 36, 39, 42, 43, 44, 41, 38],// L
            curr_idx: 0,
            facelet_rgb_values: iter::repeat(Point { x: 0., y: 0., z: 0., index: 0 }).take(54).collect(),
            next_faces: ['R', 'F', 'L', 'B'],
            right_face: 'D',
            left_face: 'U',
        }
    }
    pub fn to_notation(&self) -> String {
        // we clone so that a fonction named to_smthng doesnt have side effects
        let facelets = self.facelet_rgb_values.clone();
        let mut centres = vec![];
        let mut sides = vec![];
        let centre_index = [4, 22, 31, 49, 13, 40];
        for centre in centre_index {
            let face = facelets.get(centre).unwrap();
            centres.push(face.clone());
        }
        for side in 0..54 {
            if !centre_index.contains(&side) {
                let face = facelets.get(side).unwrap();
                sides.push(face.clone());
            }
        }
        let mut classification = Classification::init(centres, sides, 8);
        let res = classification.classify();
        let centre_to_face: HashMap<usize, char> = HashMap::from([(4, 'U'), (22, 'F'), (31, 'D'), (49, 'B'), (13, 'R'), (40, 'L')]);
        let mut string: Vec<char> = iter::repeat(' ').take(54).collect();
        for key in res.keys() {
            let facechar = centre_to_face.get(&key.index).unwrap().clone();
            string[key.index] = facechar;
            for point in res.get(key).unwrap() {
                string[point.1.index] = facechar;
            }
        }
        string.iter().collect()
    }

    pub fn solve_cube(&self) -> String {
        Self::solve_cube_notation(self.to_notation())
    }

    pub fn solve_cube_notation(cube_notation: String) -> String {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("./kociemba {}", cube_notation))
            .output()
            .expect("Failed to execute Kociemba executable");
        String::from_utf8(output.stdout).expect("Could not convert Kociemba output to string")
    }
}






