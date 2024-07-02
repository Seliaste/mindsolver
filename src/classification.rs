use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct ColorPoint {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub index: usize,
}

impl Hash for ColorPoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl Eq for ColorPoint {}

impl PartialEq for ColorPoint {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl ColorPoint {
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.r - other.r).powi(2) / 2. + (self.g - other.g).powi(2) + (self.b - other.b).powi(2))
            .sqrt()
    }

    /// Returns an array of the three coordinates
    pub fn to_array(&self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

/// This is a DIY classification technique.
/// In an n-dimension cloud, we want to separate red points and black points. Red points will represent a class.
/// Black points need to be linked to a single red point. We don't want red points to be linked between them.
/// Here, central face colors are red points and side colors are black points. Each red point has to be linked to k black points.
/// We store distances of every red points for each black point.
/// We start assigning black points that have the lowest distance to a red point, and once a red point has 8 elements we remove it from the assignable to.
/// We are done once all the black points are assigned. We are then sure every point got assigned to a red point that has 8 elements or fewer.
pub struct Classification {
    /// Centroids
    red_points: Vec<ColorPoint>,
    /// To get classified
    black_points: Vec<ColorPoint>,
    /// Number of elements per red points
    k: i32,
}

impl Classification {
    pub fn init(red_points: Vec<ColorPoint>, black_points: Vec<ColorPoint>) -> Self {
        Classification {
            k: (black_points.len() / red_points.len()) as i32,
            red_points,
            black_points,
        }
    }

    fn calc_distances(&mut self) -> Vec<(f64, ColorPoint, ColorPoint)> {
        let mut res: Vec<(f64, ColorPoint, ColorPoint)> = vec![];
        for bp in &self.black_points {
            for rp in &self.red_points {
                res.push((bp.distance_to(&rp), bp.clone(), rp.clone()))
            }
        }
        res
    }

    /// Will return a hashmap with red points as keys and vectors of assigned black points.
    pub fn classify(&mut self) -> HashMap<ColorPoint, Vec<(f64, ColorPoint)>> {
        let mut distances = self.calc_distances();
        distances.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut added = vec![];
        let mut res: HashMap<ColorPoint, Vec<(f64, ColorPoint)>> = HashMap::new();
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
    use crate::classification::{Classification, ColorPoint};

    pub fn rand_cloud(k: usize, bound: f64) -> Vec<ColorPoint> {
        let mut res = vec![];
        for i in 0..k {
            let (x, y, z) = (
                rand::random::<f64>() % bound,
                rand::random::<f64>() % bound,
                rand::random::<f64>() % bound,
            );
            res.push(ColorPoint {
                r: x,
                g: y,
                b: z,
                index: i,
            });
        }
        res
    }

    #[test]
    fn test_classify() {
        let cloud = rand_cloud(54, 100.);
        let (rp, bp) = cloud.split_at(6);
        let mut clas = Classification::init(Vec::from(rp), Vec::from(bp));
        let res = clas.classify();
        for result in res {
            assert_eq!(result.1.len(), 8)
        }
    }
}
