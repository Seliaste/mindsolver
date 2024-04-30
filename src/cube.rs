use std::collections::HashMap;
use std::iter;
use std::process::Command;

use nabo::{KDTree, NotNan};

use crate::Col;

pub struct Cube {
    // The scan order will always be the same,
    // so insted of complicated code it's better to hardcode it
    pub scan_order: Vec<usize>,
    // Current facelet number
    pub curr_idx: usize,
    // Stores RGB values in the order of the standard notatio
    pub facelet_rgb_values: Vec<Col>,
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
                             49, 52, 53, 50, 47, 46, 45, 48, 51,// B
                             13, 16, 17, 14, 11, 10, 9, 12, 15, // R
                             40, 37, 36, 39, 42, 43, 44, 41, 38],// L
            curr_idx: 0,
            facelet_rgb_values: iter::repeat(Col([NotNan::new(0.).unwrap(), NotNan::new(0.).unwrap(), NotNan::new(0.).unwrap()], ' ')).take(54).collect(),
            next_faces: ['R', 'F', 'L', 'B'],
            right_face: 'D',
            left_face: 'U',
        }
    }

    pub fn to_notation(&self) -> String {
        // we clone so that a fonction named to_smthng doesnt have side effects
        let mut facelets = self.facelet_rgb_values.clone();
        let tree = KDTree::new(&facelets);
        let centre_to_face: HashMap<usize, char> = HashMap::from([(4, 'U'), (22, 'F'), (31, 'D'), (49, 'B'), (13, 'R'), (40, 'L')]);
        for centre in [4, 22, 31, 49, 13, 40] {
            let face = centre_to_face.get(&centre).unwrap();
            facelets[centre].1 = face.clone();
            let neighbours = tree.knn(8, &facelets[centre]);
            for mut neighbour in neighbours {
                neighbour.point.1 = face.clone();
                facelets[neighbour.index as usize] = neighbour.point;
            }
        }
        facelets.iter().map(|x| x.1).collect()
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






