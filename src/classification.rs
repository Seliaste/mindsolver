use crate::constants::{CORNER_FACELET, EDGE_FACELET};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub index: usize,
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
    pub fn distance(&self, other: &Self) -> f64 {
        let res: f64 = ((self.x - other.x) / 3.).powi(2)
            + (self.y - other.y).powi(2)
            + (self.z - other.z).powi(2);
        // TODO: The /3. is a complete hack. Should be written in a more explicit way.
        // The reason for this is that the red amount is the one we can trust the less.
        res.sqrt()
    }

    /// Used in the scan saving feature
    pub fn export(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    #[allow(dead_code)] // used for testing
    pub fn rand_cloud(k: usize, bound: f64) -> Vec<Point> {
        let mut res = vec![];
        for i in 0..k {
            let (x, y, z) = (
                rand::random::<f64>() % bound,
                rand::random::<f64>() % bound,
                rand::random::<f64>() % bound,
            );
            res.push(Point { x, y, z, index: i });
        }
        res
    }
}

/// This represents a DIY classification technique.
/// In an n-dimension cloud, we want to separate red points and black points. Red points will represent a class.
/// Black points need to be linked to a single red point. We don't want red points to be linked between them.
/// Here, central face colors are red points and side colors are black points. Each red point has to be linked to k black points.
///
/// Here is Frank's algorithm:
/// Instead of storing distances of every black points for each red point, we store distances of every red points for each black point.
/// We start assigning black points that have the lowest distance to a red point, and once a red point has 8 elements we remove it from the assignable to.
/// We are done once all the black points are assigned. We are then sure every point got assigned to a red point that has 8 elements or fewer.
pub struct Classification {
    /// Centroids
    red_points: Vec<(Point, char)>,
    /// To get classified
    black_points: Vec<Point>,
    /// Number of elements per red points
    k: i32,
}

impl Classification {
    pub fn init(red_points: Vec<(Point, char)>, black_points: Vec<Point>) -> Self {
        Classification {
            k: (black_points.len() / red_points.len()) as i32,
            red_points,
            black_points,
        }
    }

    fn calc_distance(&mut self) -> Vec<(f64, Point, (Point, char))> {
        let mut res: Vec<(f64, Point, (Point, char))> = vec![];
        for bp in &self.black_points {
            for rp in &self.red_points {
                res.push((bp.distance(&rp.0), bp.clone(), rp.clone()))
            }
        }
        res
    }

    /// Will return a hashmap with red points as keys and vectors of assigned black points.
    pub fn classify(&mut self) -> HashMap<Point, Vec<(f64, Point)>> {
        let mut distances = self.calc_distance();
        distances.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut added = vec![];
        let mut res: HashMap<Point, Vec<(f64, Point)>> = HashMap::new();
        // Specific to the cube. For example, we can deduce that if a face has been classified as U, we can ban its edges and corner facelets from being D
        let opposites = HashMap::from([
            ('U', 'D'),
            ('D', 'U'),
            ('L', 'R'),
            ('R', 'L'),
            ('F', 'B'),
            ('B', 'F'),
        ]);
        let mut banned: HashMap<usize, Vec<char>> = HashMap::new();
        for i in 0..54 {
            banned.insert(i, Vec::new());
        }
        for rp in self.red_points.clone() {
            res.insert(rp.0, Vec::new());
        }
        for dist in distances {
            if added.contains(&dist.1) || banned.get(&dist.1.index).unwrap().contains(&dist.2 .1) {
                continue;
            }
            let arr = res.get_mut(&dist.2 .0).unwrap();
            if arr.len() < self.k as usize {
                arr.push((dist.0, dist.1));
                added.push(dist.1);
                let corner = CORNER_FACELET.iter().find(|x| x.contains(&dist.1.index));
                let edge = EDGE_FACELET.iter().find(|x| x.contains(&dist.1.index));
                if corner.is_some() {
                    for elem in corner.unwrap() {
                        banned
                            .get_mut(elem)
                            .unwrap()
                            .push(opposites.get(&dist.2 .1).unwrap().clone())
                    }
                }
                if edge.is_some() {
                    for elem in edge.unwrap() {
                        banned
                            .get_mut(elem)
                            .unwrap()
                            .push(opposites.get(&dist.2 .1).unwrap().clone())
                    }
                }
            }
        }
        res
    }
}

// This is a terrible function that needs to get fixed as it could produce a classification with better constraints. In the meantime, we won't use it
/* pub fn classify_corners(centres: Vec<Point>, facelets: Vec<Point>) -> HashMap<String, Option<(f64, [Point;3])>>  {
    let face_to_rgb = HashMap::from([('U',centres[0]),('F',centres[1]),('D',centres[2]),('B',centres[3]),('R',centres[4]),('L',centres[5])]);
    let mut distances: HashMap<String, Option<(f64, [Point;3])>> = HashMap::new();
    let mut seen: Vec<[usize;3]> = Vec::new();
    for reference in get_corner_colors() {
        let reference_str: String = reference.iter().collect();
        let mut foundcorner = None;
        distances.insert(reference_str.clone(), None);
        for corner in CORNER_FACELET {
            if seen.contains(&corner) { continue }
            let corner_rgb: Vec<Point> = vec![facelets[corner[0]],facelets[corner[1]],facelets[corner[2]]];
            let reference_rgb: Vec<Point> = reference_str.chars().map(|x1| {face_to_rgb[&x1]}).collect();
            let permutations_idx = vec![[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,1,0],[2,0,1]];
            let permutations: Vec<[Point;3]> = permutations_idx.iter().map(|x2| {[reference_rgb[x2[0]],reference_rgb[x2[1]],reference_rgb[x2[2]]]}).collect();
            let mut minipermut = None;
            let mut minidist = 0.;
            for permut in permutations.clone() {
                let mut distsq = 0.;
                let vec1 = [corner_rgb[0].x,corner_rgb[0].y,corner_rgb[0].z,
                    corner_rgb[1].x,corner_rgb[1].y,corner_rgb[1].z,
                    corner_rgb[2].x,corner_rgb[2].y,corner_rgb[2].z];
                let vec2 = [permut[0].x,permut[0].y,permut[0].z,
                    permut[1].x,permut[1].y,permut[1].z,
                    permut[2].x,permut[2].y,permut[2].z];
                for i in 0..9 {
                    distsq += (vec1[i]-vec2[i]).powi(2);
                }
                let dist = distsq.sqrt();
                if minipermut.is_none() || dist < minidist {
                    minipermut = Some(permut);
                    minidist = dist;
                }
            }
            let current = distances.get_mut(&reference_str).unwrap();
            let permut_idx = permutations_idx.get(permutations.iter().position(|&x3| {x3 == minipermut.unwrap()}).unwrap()).unwrap();
            let mut candidate = (minidist,[corner_rgb[permut_idx[0]].clone(),corner_rgb[permut_idx[1]].clone(),corner_rgb[permut_idx[2]].clone()]);
            if current.is_none() || current.unwrap().0 > candidate.0 {
                *current = Some(candidate);
                foundcorner = Some(corner);
            }
        }
        seen.push(foundcorner.unwrap());
    }
    return distances;
}
 */

// #[cfg(test)]
// mod tests {
//     use crate::classification::{Classification, Point};
//
//     #[test]
//     fn test_classify() {
//         let cloud = Point::rand_cloud(54, 100.);
//         let (rp, bp) = cloud.split_at(6);
//         let mut clas = Classification::init(Vec::from(rp), Vec::from(bp));
//         let res = clas.classify();
//         for result in res {
//             assert_eq!(result.1.len(), 8)
//         }
//     }
// }
