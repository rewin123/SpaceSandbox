


#[cfg(test)]
mod chunk_map_tests {
    use super::super::*;
    use super::super::solid_voxel_map::*;
    use bevy::{scene::serde::SceneDeserializer};
    use test_case::test_case;
    use serde::{Deserialize, de::DeserializeSeed};

    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn get_voxel_pos(map : impl VoxelMap<i32>) {

        let pos = Real::new(0.0, 0.0, 0.0);
        let _voxel_pos = map.get_grid_pos(&pos);
        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, 0.0);
        assert_eq!(pos.z, 0.0);
    }

    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn get_set(mut map : impl VoxelMap<i32>) {
        let pos = Real::new(11.0, 10.0, -9.0);
        assert_eq!(map.get_cloned(&pos), 0);

        map.set_voxel(&pos, 11);
        assert_eq!(map.get_cloned(&pos), 11);
    }


    #[test_case(SolidVoxelMap::<i32>::test_default())]
    fn fill(mut map : impl VoxelMap<i32>) {
        let start_pos = Real::new(0.0, 0.0, 0.0);
        let end_pos = Real::new(10.0, 10.0, 10.0);

        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            map.set_voxel(&(start_pos + t * (end_pos - start_pos)), 10);
        }

        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            assert_eq!(map.get_cloned(&(start_pos + t * (end_pos - start_pos))), 10);
        }
    }

    #[test]
    fn ron_save_load_solid_map() {
        let mut map = SolidVoxelMap::<i32>::test_default();
        let start_pos = Real::new(0.0, 0.0, 0.0);
        let end_pos = Real::new(10.0, 10.0, 10.0);

        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            super::super::VoxelMap::set_voxel(&mut map, &(start_pos + t * (end_pos - start_pos)), 10);
        }


        let disk = ron::to_string(&map).unwrap();
        let map_2 : SolidVoxelMap::<i32> = ron::from_str(&disk).unwrap();


        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            assert_eq!(map_2.get_cloned(&(start_pos + t * (end_pos - start_pos))), 10);
        }
    }

    #[test]
    fn world_save_load_solid_map() {
        let mut map = SolidVoxelMap::<i32>::test_default();
        let start_pos = Real::new(0.0, 0.0, 0.0);
        let end_pos = Real::new(10.0, 10.0, 10.0);

        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            super::super::VoxelMap::set_voxel(&mut map, &(start_pos + t * (end_pos - start_pos)), 10);
        }

        let disk = ron::to_string(&map).unwrap();
        let map_2 : SolidVoxelMap::<i32> = ron::from_str(&disk).unwrap();


        for idx in 0..100 {
            let t = idx as f64 / 100.0;
            assert_eq!(map_2.get_cloned(&(start_pos + t * (end_pos - start_pos))), 10);
        }
    }

    #[derive(Reflect, Component)]
    struct TestStruct {
        pub val : f32
    }

    #[test]
    fn simple_scene_load() {
        let mut world = World::default();

        let type_registry = AppTypeRegistry::default();
        type_registry.internal.write().register::<TestStruct>();

        world.spawn(TestStruct { val : 1.0});
        
        let dyn_scene = DynamicScene::from_world(&world);
        let scene_ron = dyn_scene.serialize_ron(&type_registry).unwrap();

        let mut des = ron::Deserializer::from_bytes(scene_ron.as_bytes()).unwrap();
        let result = SceneDeserializer {
            type_registry : &type_registry.read()
        }.deserialize(&mut des).unwrap();

        let new_world = Scene::from_dynamic_scene(&result, &type_registry).unwrap().world;

        assert_eq!(new_world.entities().len(), world.entities().len());
    }

}
