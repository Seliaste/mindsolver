/*
This is an experimental classification technique.
in an n-dimension cloud, we want to separate red points and black points. Red points will represent a class.
Black points need to be linked to a single red point. We don't want red points to be linked between them.
Here, central face colors are red points and side colors are black points. Each red point has to be linked to k black points.
To do this, we assign to every red point a list of the black points ordered by distance.
If a black point is present in multiple lists of the k first points,
then we keep only the one with the shortest distance and pop the other instances. We repeat this until there are no pops
The class of the black point is determined by which list it appears in.
 */

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct Point {
    x:f64,y:f64,z:f64,index:i32
}
impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}
impl Eq for Point {}
impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Point {
    fn dist(&self,other:&Self) -> f64 {
        let res : f64 = (self.x-other.x).powi(2) + (self.y-other.y).powi(2) + (self.z-other.z).powi(2);
        res.sqrt()
    }
}


pub struct Classification {
    red_points:Vec<Point>,
    black_points:Vec<Point>,
    distances:HashMap<Point,Vec<(f64,Point)>>,
    per_red_points:i32
}

impl Classification {
    fn init(red_points:Vec<Point>, black_points:Vec<Point>, per_red_points:i32) -> Self {
        let distances = HashMap::new();
        Classification{red_points,black_points,distances,per_red_points}
    }

    fn calc_distances(&mut self) {
        for rp in &self.red_points{
            self.distances.insert(rp.clone(), vec![]);
            let vec: &mut Vec<(f64,Point)> = self.distances.get_mut(&rp).unwrap();
            for bp in &self.black_points{
                vec.push((bp.dist(rp),bp.clone()))
            }
        }
    }
}