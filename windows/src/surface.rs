use std::collections::HashMap;
use std::{cmp::Ordering, path};

use ordered_float::OrderedFloat;
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::config;
use crate::winit_compat::MonitorHandle;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Surface {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    scale_factor: OrderedFloat<f64>,
    wallpaper: Option<path::PathBuf>,
}

impl PartialOrd for Surface {
    fn partial_cmp(&self, other: &Surface) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Surface {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl Surface {
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
        self.scale_factor.into()
    }
    #[inline]
    pub fn wallpaper(&self) -> &Option<path::PathBuf> {
        &self.wallpaper
    }
}

impl Surface {
    fn from_monitor(monitor: &MonitorHandle, wallpaper: &Option<path::PathBuf>) -> Self {
        Self {
            position: monitor.position(),
            size: monitor.size(),
            scale_factor: monitor.scale_factor().into(),
            wallpaper: wallpaper.clone(),
        }
    }

    fn merge(&mut self, surface: &Self) {
        // if self.scale_factor != surface.scale_factor {
        //     return None;
        // }

        let top_left = PhysicalPosition::new(
            self.position.x.min(surface.position.x),
            self.position.y.min(surface.position.y),
        );

        let bottom_right = PhysicalPosition::new(
            (self.position.x + self.size.width as i32)
                .max(surface.position.x + surface.size.width as i32),
            (self.position.y + self.size.height as i32)
                .max(surface.position.y + surface.size.height as i32),
        );

        self.position = top_left;
        self.size = PhysicalSize::new(
            top_left.x.abs_diff(bottom_right.x),
            top_left.y.abs_diff(bottom_right.y),
        );
    }
}

fn from_monitors(monitors: &[(MonitorHandle, Option<path::PathBuf>)]) -> Vec<Surface> {
    monitors
        .iter()
        .map(|(monitor, wallpaper)| Surface::from_monitor(monitor, wallpaper))
        .collect()
}

fn extend(surfaces: Vec<Surface>) -> Vec<Surface> {
    let mut grouping: HashMap<PhysicalSize<u32>, Surface> = HashMap::new();
    for surface in surfaces.into_iter() {
        grouping
            .entry(surface.size)
            .and_modify(|existing_surface| existing_surface.merge(&surface))
            .or_insert_with(|| surface);
    }
    let mut extended_surfaces = grouping.into_values().collect::<Vec<Surface>>();
    extended_surfaces.sort();
    extended_surfaces
}

fn fill(surfaces: Vec<Surface>) -> Vec<Surface> {
    let optional_surface = surfaces.into_iter().reduce(|mut a, b| {
        a.merge(&b);
        a
    });
    // Return a vec of one surface or an empty vec.
    if let Some(surface) = optional_surface {
        vec![surface]
    } else {
        vec![]
    }
}

pub fn build(
    monitors: &[(MonitorHandle, Option<path::PathBuf>)],
    fill_mode: config::FillMode,
) -> Vec<Surface> {
    let surfaces = from_monitors(monitors);

    use config::FillMode;
    match fill_mode {
        FillMode::None => surfaces,
        FillMode::Span => extend(surfaces),
        FillMode::Fill => fill(surfaces),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_does_not_extend_two_different_displays() {
        let display0 = Surface {
            position: (0, 0).into(),
            size: (3360, 2100).into(),
            scale_factor: 1.0.into(),
            wallpaper: None,
        };
        let display1 = Surface {
            position: (3360, 0).into(),
            size: (2560, 1440).into(),
            scale_factor: 1.0.into(),
            wallpaper: None,
        };

        assert_eq!(
            extend(vec![display0.clone(), display1.clone()]),
            vec![display0, display1]
        );
    }

    #[test]
    fn it_fills_all_displays() {
        let display0 = Surface {
            position: (-500, 0).into(),
            size: (1920, 1080).into(),
            scale_factor: 1.0.into(),
            wallpaper: None,
        };
        let display1 = Surface {
            position: (1420, 0).into(),
            size: (2560, 1440).into(),
            scale_factor: 1.0.into(),
            wallpaper: None,
        };
        assert_eq!(
            fill(vec![display0, display1]),
            vec![Surface {
                position: (-500, 0).into(),
                size: (4480, 1440).into(),
                scale_factor: 1.0.into(),
                wallpaper: None,
            }]
        );
    }
}
//
//     #[test]
//     fn it_partially_combines_two_1440p_displays_and_a_separate_laptop_display() {
//         // 1440p + 1440p + laptop
//         let display0 = Surface::from_bounds(Rect::new(-2560, 0, 2560, 1440), BASE_DPI as f64);
//         let display1 = Surface::from_bounds(Rect::new(0, 0, 2560, 1440), BASE_DPI as f64);
//         let display2 = Surface::from_bounds(Rect::new(2560, 0, 3360, 2100), BASE_DPI as f64);
//
//         assert_eq!(
//             Surface::combine_displays(&[display0, display1, display2]),
//             vec![
//                 Surface::from_bounds(Rect::new(-2560, 0, 5120, 1440), BASE_DPI as f64),
//                 display2
//             ]
//         );
//
//         // laptop + 1440p + 1440p
//         let display2 = Surface::from_bounds(Rect::new(-1920, 360, 1920, 1080), BASE_DPI as f64);
//         let display0 = Surface::from_bounds(Rect::new(0, 0, 2560, 1440), BASE_DPI as f64);
//         let display1 = Surface::from_bounds(Rect::new(2560, 0, 2560, 1440), BASE_DPI as f64);
//
//         assert_eq!(
//             Surface::combine_displays(&[display2, display0, display1]),
//             vec![
//                 display2,
//                 Surface::from_bounds(Rect::new(0, 0, 5120, 1440), BASE_DPI as f64),
//             ]
//         );
//     }
//
//     #[test]
//     fn it_combines_two_1440p_displays() {
//         let display0 = Surface::from_bounds(Rect::new(0, 0, 2560, 1440), BASE_DPI as f64);
//         let display1 = Surface::from_bounds(
//             Rect::new(display0.bounds.width() as i32, 0, 2560, 1440),
//             BASE_DPI as f64,
//         );
//
//         assert_eq!(
//             Surface::combine_displays(&[display0, display1]),
//             vec![Surface::from_bounds(
//                 Rect::new(0, 0, 5120, 1440),
//                 BASE_DPI as f64
//             )]
//         );
//     }
//
//     #[test]
//     fn it_combines_three_1440p_displays() {
//         let display0 = Surface::from_bounds(Rect::new(-2560, 0, 2560, 1440), BASE_DPI as f64);
//         let display1 = Surface::from_bounds(Rect::new(0, 0, 2560, 1440), BASE_DPI as f64);
//         let display2 = Surface::from_bounds(Rect::new(2560, 0, 2560, 1440), BASE_DPI as f64);
//
//         assert_eq!(
//             Surface::combine_displays(&[display0, display1, display2]),
//             vec![Surface::from_bounds(
//                 Rect::new(-2560, 0, 2560 * 3, 1440),
//                 BASE_DPI as f64
//             )]
//         );
//     }
//
//     #[test]
//     fn it_combines_a_grid_of_displays() {
//         let display0 = Surface::from_bounds(Rect::new(0, 0, 2560, 1440), BASE_DPI as f64);
//         let display1 = Surface::from_bounds(Rect::new(2560, 0, 2560, 1440), BASE_DPI as f64);
//         let display2 = Surface::from_bounds(Rect::new(0, 1440, 2560, 1440), BASE_DPI as f64);
//         let display3 = Surface::from_bounds(Rect::new(2560, 1440, 2560, 1440), BASE_DPI as f64);
//
//         assert_eq!(
//             Surface::combine_displays(&[display0, display1, display2, display3]),
//             vec![Surface::from_bounds(
//                 Rect::new(0, 0, 2560 * 2, 1440 * 2),
//                 BASE_DPI as f64
//             ),]
//         );
//
//         let laptop = Surface::from_bounds(Rect::new(2560 * 2, 0, 1920, 1080), BASE_DPI as f64);
//         assert_eq!(
//             Surface::combine_displays(&[display0, display1, display2, display3, laptop]),
//             vec![
//                 Surface::from_bounds(Rect::new(0, 0, 2560 * 2, 1440 * 2), BASE_DPI as f64),
//                 laptop
//             ]
//         );
//     }
// }
