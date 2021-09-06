use bevy::{
    prelude::*,
    render::{
        pipeline::PipelineDescriptor,
        shader::{ShaderStage, ShaderStages},
    },
};

pub const VERTEX: &str = r#"
#version 450
layout(location = 0) in uint Vertex_Position;
layout(location = 0) out vec3 v_color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
const vec3 NORMALS[6] = {
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, -1.0, 0.0),
    vec3(1.0, 0.0, 0.0),
    vec3(-1.0, 0.0, 0.0),
    vec3(0.0, 0.0, 1.0),
    vec3(0.0, 0.0, -1.0),
};
const vec3 up = vec3(0.0, 1.0, 0.0);
void main() {
    vec3 position = vec3((Vertex_Position >> 27) & 31, (Vertex_Position >> 22) & 31, (Vertex_Position >> 17) & 31);
    gl_Position = ViewProj * Model * vec4(position, 1.0);
    vec3 color = vec3(((Vertex_Position >> 11) & 7) / 7.0, 
                    ((Vertex_Position >>  8) & 7) / 7.0, 
                    ((Vertex_Position >>  5) & 7) / 7.0) * ((Vertex_Position & 31) / 31.0);
    vec3 normal = NORMALS[(Vertex_Position >> 14) & 7];
    v_color = color.xyz * (0.5 + dot(up, normal) * 0.5);
}
"#;

pub const FRAGMENT: &str = r#"
#version 450
layout(location = 0) out vec4 o_Target;
layout(location = 0) in vec3 v_color;
void main() {
    o_Target = vec4(v_color, 1.0);
}
"#;

pub struct Pipeline {
    pub handle: Handle<PipelineDescriptor>,
}

pub fn pipeline_setup(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT))),
    }));
    commands.insert_resource(Pipeline {
        handle: pipeline_handle.clone(),
    });
}
