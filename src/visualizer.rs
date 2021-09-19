use watertender::prelude::*;
use anyhow::Result;
use crate::{settings::Settings, engine::{Engine, SceneData}};
use std::time::Instant;

struct Visualizer {
    engine: Engine,
    camera: MultiPlatformCamera,
    starter_kit: StarterKit,
    start: Instant,
}

pub fn visualize(cfg: Settings, vr: bool) -> Result<()> {
    let info = AppInfo::default().validation(cfg.validation);
    launch::<Visualizer, Settings>(info, vr, cfg)
}

impl MainLoop<Settings> for Visualizer {
    fn new(core: &SharedCore, mut platform: Platform<'_>, cfg: Settings) -> Result<Self> {
        let starter_kit = StarterKit::new(core.clone(), &mut platform)?;
        let camera = MultiPlatformCamera::new(&mut platform);
        let engine = Engine::new(core.clone(), cfg, starter_kit.render_pass, starter_kit.current_command_buffer())?;

        let start = Instant::now();

        Ok(Self {
            camera,
            starter_kit,
            engine,
            start,
        })
    }

    fn frame(
        &mut self,
        frame: Frame,
        _core: &SharedCore,
        platform: Platform<'_>,
    ) -> Result<PlatformReturn> {
        let cmd = self.starter_kit.begin_command_buffer(frame)?;
        let command_buffer = cmd.command_buffer;

        // TODO: Remove me!
        let (ret, _) = self.camera.get_matrices(&platform)?;

        let extent = self.starter_kit.framebuffer.extent();

        let scene = SceneData {
            resolution_x: extent.width as f32,
            resolution_y: extent.height as f32,
            time: self.start.elapsed().as_secs_f32(),
        };

        self.engine.write_commands(command_buffer, self.starter_kit.frame, &scene)?;

        self.starter_kit.end_command_buffer(cmd)?;

        Ok(ret)
    }

    fn swapchain_resize(&mut self, images: Vec<vk::Image>, extent: vk::Extent2D) -> Result<()> {
        self.starter_kit.swapchain_resize(images, extent)
    }

    fn event(
        &mut self,
        mut event: PlatformEvent<'_, '_>,
        _core: &Core,
        mut platform: Platform<'_>,
    ) -> Result<()> {
        self.camera.handle_event(&mut event, &mut platform);
        starter_kit::close_when_asked(event, platform);
        Ok(())
    }
}

impl SyncMainLoop<Settings> for Visualizer {
    fn winit_sync(&self) -> (vk::Semaphore, vk::Semaphore) {
        self.starter_kit.winit_sync()
    }
}
