use std::num::NonZeroU32;

use sdl2::video::Window;
use sdl2::VideoSubsystem;

use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorHandle {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    scale_factor: f64,
}

impl MonitorHandle {
    #[inline]
    pub fn position(&self) -> PhysicalPosition<i32> {
        self.position
    }
    #[inline]
    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
    #[inline]
    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }
}

pub trait HasWinitWindow {
    fn inner_size(&self) -> PhysicalSize<u32>;
    fn scale_factor(&self) -> f64;
    fn current_monitor(&self) -> Option<MonitorHandle>;
}

impl HasWinitWindow for Window {
    fn inner_size(&self) -> PhysicalSize<u32> {
        let (w, h) = self.size();
        PhysicalSize::new(w, h)
    }

    fn scale_factor(&self) -> f64 {
        let id = self.display_index().unwrap();
        self.subsystem().display_dpi(id).unwrap().0 as f64 / 96.0
    }

    fn current_monitor(&self) -> Option<MonitorHandle> {
        self.display_index().ok().and_then(|id| {
            self.subsystem()
                .display_bounds(id)
                .ok()
                .map(|bounds| MonitorHandle {
                    position: PhysicalPosition::new(bounds.x, bounds.y),
                    size: bounds.size().into(),
                    scale_factor: self.subsystem().display_dpi(id).unwrap().0 as f64 / 96.0,
                })
        })
    }
}

pub trait HasMonitors {
    fn available_monitors(&self) -> impl Iterator<Item = MonitorHandle> + '_;
}

impl HasMonitors for VideoSubsystem {
    fn available_monitors(&self) -> impl Iterator<Item = MonitorHandle> + '_ {
        let monitor_count = self.num_video_displays().unwrap();
        (0..monitor_count).map(|id| {
            let bounds = self.display_bounds(id).unwrap();
            MonitorHandle {
                position: PhysicalPosition::new(bounds.x, bounds.y),
                size: bounds.size().into(),
                scale_factor: self.display_dpi(id).unwrap().0 as f64 / 96.0,
            }
        })
    }
}

/// [`winit::dpi::PhysicalSize<u32>`] non-zero extensions.
pub trait NonZeroU32PhysicalSize {
    /// Converts to non-zero `(width, height)`.
    fn non_zero(self) -> Option<(NonZeroU32, NonZeroU32)>;
}
impl NonZeroU32PhysicalSize for PhysicalSize<u32> {
    fn non_zero(self) -> Option<(NonZeroU32, NonZeroU32)> {
        let w = NonZeroU32::new(self.width)?;
        let h = NonZeroU32::new(self.height)?;
        Some((w, h))
    }
}
