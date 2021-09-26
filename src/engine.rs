use watertender::prelude::*;
use anyhow::{Result, Context};
use crate::settings::Settings;
use std::ffi::CString;
use std::path::Path;

static VERTEX_SHADER_SPV: &[u8] = include_bytes!("shaders/builtin.vert.spv");

pub struct Engine {
    cfg: Settings,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    scene_ubo: FrameDataUbo<SceneData>,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pub core: SharedCore,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SceneData {
    pub resolution_x: f32,
    pub resolution_y: f32,
    pub time: f32,
    // TODO: Add mouse in interactive mode!
}

impl Engine {
    pub fn new(
        core: SharedCore, 
        cfg: Settings, 
        render_pass: vk::RenderPass,
    ) -> Result<Self> {
        // Load fragment shader
        let fragment_spv = load_fragment_shader(&cfg.shader)?;

        // Scene data
        let scene_ubo = FrameDataUbo::new(core.clone(), cfg.frames_in_flight)?;

        // Create descriptor set layout
        const FRAME_DATA_BINDING: u32 = 0;
        const TEX_DATA_BINDING: u32 = 1;
        let bindings = [
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(FRAME_DATA_BINDING)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS),
            vk::DescriptorSetLayoutBindingBuilder::new()
                .binding(TEX_DATA_BINDING)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        ];

        let descriptor_set_layout_ci =
            vk::DescriptorSetLayoutCreateInfoBuilder::new().bindings(&bindings);

        let descriptor_set_layout = unsafe {
            core.device
                .create_descriptor_set_layout(&descriptor_set_layout_ci, None, None)
        }
        .result()?;

        // Create descriptor pool
        let pool_sizes = [
            vk::DescriptorPoolSizeBuilder::new()
                ._type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(cfg.frames_in_flight as _),
            vk::DescriptorPoolSizeBuilder::new()
                ._type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(cfg.frames_in_flight as _),
        ];

        let create_info = vk::DescriptorPoolCreateInfoBuilder::new()
            .pool_sizes(&pool_sizes)
            .max_sets((cfg.frames_in_flight * 2) as _);

        let descriptor_pool =
            unsafe { core.device.create_descriptor_pool(&create_info, None, None) }.result()?;

        // Create descriptor sets
        let layouts = vec![descriptor_set_layout; cfg.frames_in_flight];
        let create_info = vk::DescriptorSetAllocateInfoBuilder::new()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets =
            unsafe { core.device.allocate_descriptor_sets(&create_info) }.result()?;

        // Write descriptor sets
        for (frame, &descriptor_set) in descriptor_sets.iter().enumerate() {
            let frame_data_bi = [scene_ubo.descriptor_buffer_info(frame)];
            let writes = [
                vk::WriteDescriptorSetBuilder::new()
                    .buffer_info(&frame_data_bi)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(descriptor_set)
                    .dst_binding(FRAME_DATA_BINDING)
                    .dst_array_element(0),
            ];

            unsafe {
                core.device.update_descriptor_sets(&writes, &[]);
            }
        }

        let descriptor_set_layouts = [descriptor_set_layout];

        // Pipeline layout
        let push_constant_ranges = [vk::PushConstantRangeBuilder::new()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<[f32; 4 * 4]>() as u32)];

        let create_info = vk::PipelineLayoutCreateInfoBuilder::new()
            .push_constant_ranges(&push_constant_ranges)
            .set_layouts(&descriptor_set_layouts);

        let pipeline_layout =
            unsafe { core.device.create_pipeline_layout(&create_info, None, None) }.result()?;

        // Pipeline
        let pipeline = shader(
            &core,
            VERTEX_SHADER_SPV,
            &fragment_spv,
            vk::PrimitiveTopology::TRIANGLE_LIST,
            render_pass,
            pipeline_layout,
        )?;

        Ok(Self {
            cfg,
            descriptor_set_layout,
            descriptor_sets,
            descriptor_pool,
            pipeline_layout,
            scene_ubo,
            pipeline,
            core,
        })
    }

    pub fn write_commands(&mut self, command_buffer: vk::CommandBuffer, frame: usize, scene: &SceneData) -> Result<()> {
        // TODO: Factor this out?
        self.scene_ubo.upload(
            frame,
            scene,
        )?;

        unsafe {
            self.core.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[frame]],
                &[],
            );

            self.core.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.core.device.cmd_draw(command_buffer, 3, 1, 0, 0);
        }

        Ok(())
    }

    pub fn cfg(&self) -> &Settings {
        &self.cfg
    }
}

unsafe impl bytemuck::Zeroable for SceneData {}
unsafe impl bytemuck::Pod for SceneData {}

pub fn shader(
    prelude: &Core,
    vertex_src: &[u8],
    fragment_src: &[u8],
    primitive: vk::PrimitiveTopology,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
) -> Result<vk::Pipeline> {
    // Create shader modules
    let vert_decoded = erupt::utils::decode_spv(vertex_src)?;
    let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&vert_decoded);
    let vertex = unsafe {
        prelude
            .device
            .create_shader_module(&create_info, None, None)
    }
    .result()?;

    let frag_decoded = erupt::utils::decode_spv(fragment_src)?;
    let create_info = vk::ShaderModuleCreateInfoBuilder::new().code(&frag_decoded);
    let fragment = unsafe {
        prelude
            .device
            .create_shader_module(&create_info, None, None)
    }
    .result()?;

    // Build pipeline
    let vertex_input = vk::PipelineVertexInputStateCreateInfoBuilder::new()
        .vertex_attribute_descriptions(&[])
        .vertex_binding_descriptions(&[]);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
        .topology(primitive)
        .primitive_restart_enable(false);

    let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
        .viewport_count(1)
        .scissor_count(1);

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic_state =
        vk::PipelineDynamicStateCreateInfoBuilder::new().dynamic_states(&dynamic_states);

    let rasterizer = vk::PipelineRasterizationStateCreateInfoBuilder::new()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_clamp_enable(false);

    let multisampling = vk::PipelineMultisampleStateCreateInfoBuilder::new()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlagBits::_1);

    let color_blend_attachments = [vk::PipelineColorBlendAttachmentStateBuilder::new()
        .color_write_mask(
            vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
        )
        .blend_enable(true)
        .color_blend_op(vk::BlendOp::ADD)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
    ];
    let color_blending = vk::PipelineColorBlendStateCreateInfoBuilder::new()
        .logic_op_enable(false)
        .attachments(&color_blend_attachments);

    let entry_point = CString::new("main")?;

    let shader_stages = [
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::VERTEX)
            .module(vertex)
            .name(&entry_point),
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .stage(vk::ShaderStageFlagBits::FRAGMENT)
            .module(fragment)
            .name(&entry_point),
    ];

    let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfoBuilder::new()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false);

    let create_info = vk::GraphicsPipelineCreateInfoBuilder::new()
        .stages(&shader_stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisampling)
        .color_blend_state(&color_blending)
        .depth_stencil_state(&depth_stencil_state)
        .dynamic_state(&dynamic_state)
        .layout(pipeline_layout)
        .render_pass(render_pass)
        .subpass(0);

    let pipeline = unsafe {
        prelude
            .device
            .create_graphics_pipelines(None, &[create_info], None)
    }
    .result()?[0];

    unsafe {
        prelude.device.destroy_shader_module(Some(fragment), None);
        prelude.device.destroy_shader_module(Some(vertex), None);
    }

    Ok(pipeline)
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe {
            self.core.device.device_wait_idle().unwrap();
            self.core.device.destroy_descriptor_pool(Some(self.descriptor_pool), None);
            self.core.device.destroy_descriptor_set_layout(Some(self.descriptor_set_layout), None);
            self.core.device.destroy_pipeline_layout(Some(self.pipeline_layout), None);
            self.core.device.destroy_pipeline(Some(self.pipeline), None);
        }
    }
}

fn load_fragment_shader(path: &Path) -> Result<Vec<u8>> {
    let source = std::fs::read_to_string(path).with_context(|| format!("Failed to find shader source at \"{}\"", path.display()))?;

    let source = doctor_source(source);

    //println!("{}", source);

    let mut compiler = shaderc::Compiler::new().context("Could not find shaderc compiler")?;

    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));

    let binary_result = compiler.compile_into_spirv(
        &source, 
        shaderc::ShaderKind::Fragment,
        path.to_str().expect("Non-utf8 shader name"), 
        "main", 
        Some(&options)
    )
    .context("Failed to compile shader")?;

    Ok(binary_result.as_binary_u8().to_vec())
}

fn doctor_source(source: String) -> String {
    "#version 450
    layout(binding = 0) uniform BosRenderSceneData {
        float resolution_x;
        float resolution_y;
        float u_time;
    };
    layout(location = 0) out vec4 bos_render_output_color;
    vec2 u_resolution = vec2(resolution_x, resolution_y);"
    .to_string()
        + &source
            .replace("uniform vec2 u_resolution;", "")
            .replace("uniform vec2 u_mouse;", "")
            .replace("uniform float u_time;", "")
            .replace("gl_FragColor", "bos_render_output_color")

}
