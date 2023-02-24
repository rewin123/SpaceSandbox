use bevy::prelude::*;

#[derive(Resource)]
pub struct MulistageAssetLoading {
    pub pipelines : Vec<PipelineAsset>,

}


pub trait StageAsset {
    fn get_name(&self) -> String;
    fn get_output_asset(&self, world : &mut World) -> Option<HandleUntyped>;
}

pub struct PipelineAsset {
    pub stages : Vec<Box<dyn StageAsset + Send + Sync>>,
    pub cache : Vec<HandleUntyped>,
    pub current_stage : usize,
    pub finish_asset : Option<HandleUntyped>,
}

// impl PipelineAsset {
//     pub fn new(stages : Vec<Box<dyn StageAsset + Send + Sync>>, cache : Vec<HandleUntyped>, set_asset : Box<dyn Fn(&mut World, &HandleUntyped) + Send + Sync>) -> Self {
//         Self { stages, cache, current_stage: 0, finish_asset: None, set_asset }
//     }

//     pub fn finish(&mut self) {
        
//     }

//     pub fn update(&mut self, world : &mut World, asset_server : &mut AssetServer) {
//         if self.current_stage >= self.stages.len() {
//             // self.finish(world);
//         } else {
//             let stage = &mut self.stages[self.current_stage];
//             if let Some(asset) = stage.get_output_asset(world) {
//                 self.cache.push(asset);
//                 self.current_stage += 1;
//             }
//         }
//     }
// }

// fn asset_pipeline_system(
//     mut world : &mut World,
// ) {
//     let mut asset_server = world.get_resource_mut::<AssetServer>().unwrap();
//     let mut asset_loading = world.get_resource_mut::<MulistageAssetLoading>().unwrap();

//     asset_loading.pipelines.iter_mut().for_each(|pipeline| {
//         pipeline.update(&mut asset_server); 
//     });
// }