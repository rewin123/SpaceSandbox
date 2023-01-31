
extern crate test;

use bevy::prelude::*;
use super::{VoxelMap, solid_voxel_map::SolidVoxelMap};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VoxelVal<VoxelID> {
    None,
    Voxel(VoxelID),
    Object(Entity),
}


pub trait ObjectedVoxelMap<T> : VoxelMap<VoxelVal<T>>
    where T : Default + Clone
{
    fn set_object_by_idx(&mut self, e : Entity, pos : &IVec3, bbox : &IVec3) {
        for z in 0..bbox.z {
            for y in 0..bbox.y {
                for x in 0..bbox.x {
                    self.set_by_idx(&(*pos + IVec3 {x, y, z}), VoxelVal::Object(e));
                }
            }
        }
    }

    fn can_place_object(&mut self, pos : &IVec3, bbox : &IVec3) -> bool {
        for z in 0..bbox.z {
            for y in 0..bbox.y {
                for x in 0..bbox.x {
                    if let VoxelVal::None = self.get_by_idx(&(*pos + IVec3 {x, y, z})) {
                        
                    } else {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    fn erase_object(&mut self, pos : &IVec3, search_area : &IVec3) {
        for z in (pos.z - search_area.z)..=(pos.z + search_area.z) {
            for y in (pos.y - search_area.y)..=(pos.y + search_area.y) {
                for x in (pos.x - search_area.x)..=(pos.x + search_area.x) {
                    if let VoxelVal::Object(_) = self.get_by_idx(&IVec3{x, y, z}) {
                        self.set_by_idx(&IVec3{x, y, z}, VoxelVal::None);
                    }
                }
            }
        }
    }
}


impl<T> ObjectedVoxelMap<T> for SolidVoxelMap<VoxelVal<T>> 
    where T : Default + Clone
{

}

impl<T> Default for VoxelVal<T> {
    fn default() -> Self {
        VoxelVal::None
    }
}

#[cfg(test)]
mod tests {
    use crate::space_voxel::solid_voxel_map::SolidVoxelMap;

    use super::*;

    use test::{Bencher, black_box};


    #[test]
    fn solid_objected_map() {
        let mut map = SolidVoxelMap::<VoxelVal<i32>>::test_default();

        let e = Entity::from_raw(12356);

        let pos = IVec3::new(0,0,0);
        let bbox = IVec3::new(10, 5,3);
        //test can place
        let res = map.can_place_object(&pos, &bbox);
        assert_eq!(res, true);

        //place object
        map.set_object_by_idx(e, &pos, &bbox);

        //test can place again
        let res = map.can_place_object(&pos, &bbox);
        assert_eq!(res, false);


        //test can place half
        let res = map.can_place_object(&pos, &(bbox / 2));
        assert_eq!(res, false);

        //remove object
        map.erase_object(&pos, &bbox);

        //test can place again
        let res = map.can_place_object(&pos, &bbox);
        assert_eq!(res, true);
        
    }

    #[bench]
    fn add_bench_solid(b : &mut Bencher) {
        let mut map = SolidVoxelMap::<VoxelVal<i32>>::test_default();
        let e = Entity::from_raw(12356);
        let pos = IVec3::new(0,0,0);
        let bbox = IVec3::new(10, 5,3);

        b.iter( black_box(|| {
            map.can_place_object(&pos, &bbox);
            map.set_object_by_idx(e, &pos, &bbox);
        }));
    }

    #[bench]
    fn erase_bench_solid(b : &mut Bencher) {
        let mut map = SolidVoxelMap::<VoxelVal<i32>>::test_default();
        let e = Entity::from_raw(12356);
        let pos = IVec3::new(0,0,0);
        let bbox = IVec3::new(10, 5,3);

        b.iter( black_box(|| {
            map.erase_object(&pos, &bbox);
        }));
    }
}