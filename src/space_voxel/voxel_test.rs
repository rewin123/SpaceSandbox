#[cfg(test)]
mod chunk_map_tests {
    use super::super::*;
    use super::super::solid_voxel_map::*;
    use test_case::test_case;

    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn get_voxel_pos(map : impl VoxelMap<i32>) {

        let pos = Vec3::new(0.0, 0.0, 0.0);
        let voxel_pos = map.get_grid_pos(&pos);
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn get_set(mut map : impl VoxelMap<i32>) {
        let pos = Vec3::new(11.0, 10.0, -9.0);
        assert_eq!(map.get_cloned(&pos), 0);

        map.set(&pos, 11);
        assert_eq!(map.get_cloned(&pos), 11);
    }


    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn fill(mut map : impl VoxelMap<i32>) {
        let start_pos = Vec3::new(0.0, 0.0, 0.0);
        let end_pos = Vec3::new(10.0, 10.0, 10.0);

        for idx in 0..100 {
            let t = idx as f32 / 100.0;
            map.set(&(start_pos + t * (end_pos - start_pos)), 10);
        }

        for idx in 0..100 {
            let t = idx as f32 / 100.0;
            assert_eq!(map.get_cloned(&(start_pos + t * (end_pos - start_pos))), 10);
        }
    }
}
