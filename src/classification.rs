/*
This is an experimental classification technique.
In an n-dimension cloud, we want to separate red points and black points. Red points will represent a class.
Black points need to be linked to a single red point. We don't want red points to be linked between them.
Here, central face colors are red points and side colors are black points. Each red point has to be linked to k black points.
To do this, we assign to every red point a list of the black points ordered by distance.
If a black point is present in multiple lists of the k first points,
then we keep only the one with the shortest distance and pop the other instances. We repeat this until there are no pops
The class of the black point is determined by which list it appears in.

Other way suggested by Frank:
Instead of storing distances of every black points for each red point, we store distances of every red points for each black point.
We start assigning black points that have the lowest distance to a red point, and once a red point has 8 elements we remove it from the assignable to.
We are done once all the black points are assigned. We are then sure every point got assigned to a red point that has 8 elements or fewer.
 */

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub(crate) x:f64,
    pub(crate) y:f64,
    pub(crate) z:f64,
    pub(crate) index:usize
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
    pub fn dist(&self,other:&Self) -> f64 {
        let res : f64 = (self.x-other.x).powi(2) + (self.y-other.y).powi(2) + (self.z-other.z).powi(2);
        res.sqrt()
    }

    pub fn rand_cloud(k:usize, bound: f64) -> Vec<Point> {
        let mut res = vec![];
        for i in 0..k{
            let (x,y,z) = (rand::random::<f64>()%bound,rand::random::<f64>()%bound,rand::random::<f64>()%bound);
            res.push(Point{x,y,z,index:i});
        }
        res
    }
}


pub struct Classification {
    red_points:Vec<Point>,
    black_points:Vec<Point>,
    distances:HashMap<Point,Vec<(f64,Point)>>,
    k:i32
}

impl Classification {
    pub fn init(red_points:Vec<Point>, black_points:Vec<Point>, k:i32) -> Self {
        let distances = HashMap::new();
        Classification{red_points,black_points,distances,k}
    }

    fn calc_distance_rp(&mut self) {
        for rp in &self.red_points{
            self.distances.insert(rp.clone(), vec![]);
            let vec: &mut Vec<(f64,Point)> = self.distances.get_mut(&rp).unwrap();
            for bp in &self.black_points{
                vec.push((bp.dist(rp),bp.clone()))
            }
        }
    }

    fn calc_distance_bp(&mut self) -> Vec<(f64,Point,Point)> {
        let mut res: Vec<(f64,Point,Point)> = vec![];
        for bp in &self.black_points{
            for rp in &self.red_points{
                res.push((bp.dist(rp),bp.clone(),rp.clone()))
            }
        }
        res
    }


    fn clear_doubles(&mut self) -> bool {
        let mut modif = false;
        for point in &self.black_points{
            let mut mindist = f64::INFINITY;
            for arr in self.distances.values(){
                for dist in arr {
                    if dist.1.eq(point) && dist.0 < mindist{
                        mindist = dist.0;
                    }
                }
            }
            for arr in self.distances.values_mut(){
                for idx in 0..self.k as usize{
                    let opt = arr.get(idx);
                    if opt.is_none(){continue}
                    let dist = opt.unwrap();
                    if dist.1.eq(point) && dist.0 > mindist{
                        arr.remove(idx);
                        modif = true;
                    }
                }
            }
        }
        return modif;
    }

    pub fn aena(&mut self) -> HashMap<Point,Vec<(f64,Point)>>{
        self.calc_distance_rp();
        for vec in self.distances.values_mut(){
            vec.sort_by(|a,b| a.0.total_cmp(&b.0));
        }
        let mut modif = true;
        while modif {
            modif = self.clear_doubles();
        }
        return self.distances.clone();
    }

    pub fn frank(&mut self) -> HashMap<Point,Vec<(f64,Point)>>{
        let mut distances = self.calc_distance_bp();
        distances.sort_by(|a,b|a.0.total_cmp(&b.0));
        let mut added = vec![];
        let mut res: HashMap<Point,Vec<(f64, Point)>> = HashMap::new();
        for rp in self.red_points.clone(){
            res.insert(rp,Vec::new());
        }
        for dist in distances {
            if added.contains(&dist.1){
                continue
            }
            let arr = res.get_mut(&dist.2).unwrap();
            if arr.len() < self.k as usize {
                arr.push((dist.0,dist.1));
                added.push(dist.1);
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use crate::classification::{Classification, Point};

    // #[test]
    // fn test_aena(){
    //     let cloud = Point::rand_cloud(54,100.);
    //     let (rp,bp) = cloud.split_at(6);
    //     let mut clas = Classification::init(Vec::from(rp), Vec::from(bp), 8);
    //     let res = clas.aena();
    //     for result in res{
    //         println!("{:?}: {:?}",result.0, result.1);
    //         // assert_eq!(result.1.len(),8)
    //     }
    // }

    #[test]
    fn test_frank(){
        let cloud = Point::rand_cloud(54,100.);
        let (rp,bp) = cloud.split_at(6);
        let mut clas = Classification::init(Vec::from(rp), Vec::from(bp), 8);
        let res = clas.frank();
        for result in res{
            // assert_eq!(result.1.len(),8)
        }
    }
}