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
        let res: f64 =
            (self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2);
        res.sqrt()
    }

    #[allow(dead_code)]
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

/*
This is an experimental classification technique.
In an n-dimension cloud, we want to separate red points and black points. Red points will represent a class.
Black points need to be linked to a single red point. We don't want red points to be linked between them.
Here, central face colors are red points and side colors are black points. Each red point has to be linked to k black points.

Here is Frank's algorithm:
Instead of storing distances of every black points for each red point, we store distances of every red points for each black point.
We start assigning black points that have the lowest distance to a red point, and once a red point has 8 elements we remove it from the assignable to.
We are done once all the black points are assigned. We are then sure every point got assigned to a red point that has 8 elements or fewer.
 */
pub struct Classification {
    red_points: Vec<Point>,   // centroids
    black_points: Vec<Point>, // to classify
    k: i32,                   // nb of elements per red points
}

impl Classification {
    pub fn init(red_points: Vec<Point>, black_points: Vec<Point>) -> Self {
        Classification {
            k: (black_points.len() / red_points.len()) as i32,
            red_points,
            black_points,
        }
    }

    fn calc_distance(&mut self) -> Vec<(f64, Point, Point)> {
        let mut res: Vec<(f64, Point, Point)> = vec![];
        for bp in &self.black_points {
            for rp in &self.red_points {
                res.push((bp.distance(rp), bp.clone(), rp.clone()))
            }
        }
        res
    }

    pub fn classify(&mut self) -> HashMap<Point, Vec<(f64, Point)>> {
        let mut distances = self.calc_distance();
        distances.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut added = vec![];
        let mut res: HashMap<Point, Vec<(f64, Point)>> = HashMap::new();
        for rp in self.red_points.clone() {
            res.insert(rp, Vec::new());
        }
        for dist in distances {
            if added.contains(&dist.1) {
                continue;
            }
            let arr = res.get_mut(&dist.2).unwrap();
            if arr.len() < self.k as usize {
                arr.push((dist.0, dist.1));
                added.push(dist.1);
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::classification::{Classification, Point};

    #[test]
    fn test_classify() {
        let cloud = Point::rand_cloud(54, 100.);
        let (rp, bp) = cloud.split_at(6);
        let mut clas = Classification::init(Vec::from(rp), Vec::from(bp));
        let res = clas.classify();
        for result in res {
            assert_eq!(result.1.len(), 8)
        }
    }
}