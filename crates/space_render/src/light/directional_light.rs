use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use space_shaders::{PointLightUniform, ShaderUniform};
use nalgebra as na;
use wgpu::{TextureDimension, TextureFormat};
use wgpu::util::DeviceExt;
use space_core::RenderBase;

use space_shaders::LightCamera;

