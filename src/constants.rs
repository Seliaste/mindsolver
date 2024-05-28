#![allow(dead_code)]

use std::collections::HashSet;

#[rustfmt::skip]
pub const SCAN_ORDER: [usize; 54] = [
    4,   7,  8,  5,  2,  1,  0,  3,  6, // U
    22, 25, 26, 23, 20, 19, 18, 21, 24, // F
    31, 34, 35, 32, 29, 28, 27, 30, 33, // D
    49, 46, 45, 48, 51, 52, 53, 50, 47, // B
    13, 16, 17, 14, 11, 10,  9, 12, 15, // R
    40, 37, 36, 39, 42, 43, 44, 41, 38, // L
];

pub const SIDE_INDEXES: [usize; 24] = [
    7, 5, 1, 3, 25, 23, 19, 21, 34, 32, 28, 30, 46, 48, 52, 50, 16, 14, 10, 12, 37, 39, 43, 41,
];

#[rustfmt::skip]
pub enum Facelet {
    U1, U2, U3, U4, _U5, U6, U7, U8, U9,
    R1, R2, R3, R4, _R5, R6, R7, R8, R9,
    F1, F2, F3, F4, _F5, F6, F7, F8, F9,
    D1, D2, D3, D4, _D5, D6, D7, D8, D9,
    L1, L2, L3, L4, _L5, L6, L7, L8, L9,
    B1, B2, B3, B4, _B5, B6, B7, B8, B9,
}

#[rustfmt::skip]
pub const CORNER_FACELET: [[usize; 3]; 8] = [
    /*UBL=*/ [Facelet::U1 as usize, Facelet::L1 as usize, Facelet::B3 as usize],
    /*UBR=*/ [Facelet::U3 as usize, Facelet::B1 as usize, Facelet::R3 as usize],
    /*UFR=*/ [Facelet::U9 as usize, Facelet::R1 as usize, Facelet::F3 as usize],
    /*UFL=*/ [Facelet::U7 as usize, Facelet::F1 as usize, Facelet::L3 as usize],
    /*DFL=*/ [Facelet::D1 as usize, Facelet::L9 as usize, Facelet::F7 as usize],
    /*DFR=*/ [Facelet::D3 as usize, Facelet::F9 as usize, Facelet::R7 as usize],
    /*DBR=*/ [Facelet::D9 as usize, Facelet::R9 as usize, Facelet::B7 as usize],
    /*DBL=*/ [Facelet::D7 as usize, Facelet::B9 as usize, Facelet::L7 as usize],
];

pub const EDGE_FACELET: [[usize; 2]; 12] = [
    /*BL=*/ [Facelet::B6 as usize, Facelet::L4 as usize],
    /*BR=*/ [Facelet::B4 as usize, Facelet::R6 as usize],
    /*FR=*/ [Facelet::F6 as usize, Facelet::R4 as usize],
    /*FL=*/ [Facelet::F4 as usize, Facelet::L6 as usize],
    /*UB=*/ [Facelet::U2 as usize, Facelet::B2 as usize],
    /*UR=*/ [Facelet::U6 as usize, Facelet::R2 as usize],
    /*UF=*/ [Facelet::U8 as usize, Facelet::F2 as usize],
    /*UL=*/ [Facelet::U4 as usize, Facelet::L2 as usize],
    /*DF=*/ [Facelet::D2 as usize, Facelet::F8 as usize],
    /*DR=*/ [Facelet::D6 as usize, Facelet::R8 as usize],
    /*DB=*/ [Facelet::D8 as usize, Facelet::B8 as usize],
    /*DL=*/ [Facelet::D4 as usize, Facelet::L8 as usize],
];

pub fn get_corner_colors() -> [HashSet<char>; 8] {
    [
        /*UBL=*/ HashSet::from(['U', 'L', 'B']),
        /*UBR=*/ HashSet::from(['U', 'B', 'R']),
        /*UFR=*/ HashSet::from(['U', 'R', 'F']),
        /*UFL=*/ HashSet::from(['U', 'F', 'L']),
        /*DFL=*/ HashSet::from(['D', 'L', 'F']),
        /*DFR=*/ HashSet::from(['D', 'F', 'R']),
        /*DBR=*/ HashSet::from(['D', 'R', 'B']),
        /*DBL=*/ HashSet::from(['D', 'B', 'L']),
    ]
}

pub fn get_edge_colors() -> [HashSet<char>; 12] {
    [
        /*BL=*/ HashSet::from(['B', 'L']),
        /*BR=*/ HashSet::from(['B', 'R']),
        /*FR=*/ HashSet::from(['F', 'R']),
        /*FL=*/ HashSet::from(['F', 'L']),
        /*UB=*/ HashSet::from(['U', 'B']),
        /*UR=*/ HashSet::from(['U', 'R']),
        /*UF=*/ HashSet::from(['U', 'F']),
        /*UL=*/ HashSet::from(['U', 'L']),
        /*DF=*/ HashSet::from(['D', 'F']),
        /*DR=*/ HashSet::from(['D', 'R']),
        /*DB=*/ HashSet::from(['D', 'B']),
        /*DL=*/ HashSet::from(['D', 'L']),
    ]
}
