use std::{marker::PhantomData, sync::Arc};

use bevy::{prelude::*, reflect::{TypeRegistry, GetTypeRegistration}, ecs::{system::EntityCommand, component::ComponentId}, utils::HashMap};
use std::any::TypeId;


#[derive(Clone)]
pub struct AddDefaultComponent {
    func : Arc<dyn Fn(Entity, &mut World) + Send + Sync>
}

impl EntityCommand for AddDefaultComponent {
    fn apply(self, id: Entity, world: &mut World) {
        (self.func)(id, world);
    }
}

impl AddDefaultComponent {
    pub fn new<T : Default + Component>() -> Self {
        Self { 
            func : Arc::new(move |id, world| {
                world.entity_mut(id).insert(T::default());
            })
        }
    }
}


#[derive(Default, Resource)]
pub struct EditorRegistry {
    pub registry : TypeRegistry,
    pub spawn_components : HashMap<TypeId, AddDefaultComponent>
}

impl EditorRegistry {
    pub fn register<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self) {
        self.registry.write().register::<T>();
        self.spawn_components.insert(
            T::get_type_registration().type_id(),
            AddDefaultComponent::new::<T>()
            
        );
    }

    pub fn get_spawn_command(&self, id : &TypeId) -> AddDefaultComponent {
        self.spawn_components.get(id).unwrap().clone()
        
    }
}

pub trait EditorRegistryExt {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self);
}

impl EditorRegistryExt for App {
    fn editor_registry<T : Component + Default + Send + 'static + GetTypeRegistration>(&mut self) {
        self.world.resource_mut::<EditorRegistry>().register::<T>();
    }
}