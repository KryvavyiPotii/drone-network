use full_palette::GREY;
use plotters::coord::Shift;
use plotters::coord::ranged3d::Cartesian3d;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;

use crate::backend::ITERATION_TIME;
use crate::backend::mathphysics::Point3D;
use crate::backend::networkmodel::NetworkModel;
use crate::backend::task::Task;

use primitives::{
    attacker_device_primitive_on_all_frequencies, command_device_primitive, 
    destination_primitive, device_primitive
};

pub use plotcfg::{
    Axes3DRanges, CameraAngle, DeviceColoring, Pixel, PlottersUnit, 
    PlottersPoint3D, PlotResolution, meters_to_pixels 
};

use plotcfg::{font_size, PLOT_MARGIN};


mod plotcfg;
mod primitives;


type PlottersChartContext<'a> = ChartContext<
    'a, 
    BitMapBackend<'a>, 
    Cartesian3d<RangedCoordf64, RangedCoordf64, RangedCoordf64>
>;


const FONT: &str = "sans-serif";


fn network_model_destinations(network_model: &NetworkModel) -> Vec<Point3D> {
    let mut destinations = Vec::new();

    let task_vec: Vec<Task> = network_model
        .device_map()
        .task_map()
        .values()
        .copied()
        .collect();

    for task in task_vec {
        let Some(destination) = task.destination() else {
            continue;
        };

        destinations.push(*destination);
    }

    destinations
}


pub struct PlottersRenderer<'a> {
    output_filename: String,
    caption: String,
    plot_resolution: PlotResolution,
    font_size: Pixel,
    axes_ranges: Axes3DRanges,
    camera_angle: CameraAngle,
    device_coloring: DeviceColoring,
    area: DrawingArea<BitMapBackend<'a>, Shift>, 
}

impl<'a> PlottersRenderer<'a> {
    /// # Panics
    ///
    /// Will panic if an error occurs during bitmap backend creation. 
    #[must_use]
    pub fn new(
        output_filename: &str,
        caption: &str,
        plot_resolution: PlotResolution,
        axes_ranges: Axes3DRanges,
        device_coloring: DeviceColoring,
        camera_angle: CameraAngle,
    ) -> Self {
        let font_size = font_size(plot_resolution);
        let area      = BitMapBackend::gif(
            output_filename, 
            plot_resolution.into(),
            ITERATION_TIME
                .try_into()
                .expect("Failed to convert i32 to u32")
        )
            .expect("Failed to create `BitMapBackend`")
            .into_drawing_area();

        Self {
            output_filename: output_filename.to_string(),
            caption: caption.to_string(),
            plot_resolution,
            font_size,
            axes_ranges,
            camera_angle,
            device_coloring,
            area,
        }
    }

    #[must_use]
    pub fn output_filename(&self) -> String {
        self.output_filename.clone()
    }

    /// # Panics
    ///
    /// Will panic if an error occurs during drawing.
    pub fn render(
        &mut self, 
        network_model: &NetworkModel
    ) {
        self.area
            .fill(&WHITE)
            .expect("Failed to fill an area");
        
        let mut chart_context = self.chart_context();

        self.draw_chart(&mut chart_context);
        self.draw_network_model(network_model, &mut chart_context);

        self.area
            .present()
            .expect("Failed to finalize drawing");
    }
    
    fn chart_context(&self) -> PlottersChartContext<'a> {
        let mut chart_builder = ChartBuilder::on(&self.area);

        if !self.caption.is_empty() {
            chart_builder.caption(
                &self.caption, 
                (FONT, self.font_size)
            );
        }

        chart_builder
            .margin(PLOT_MARGIN)
            .build_cartesian_3d(
                self.axes_ranges.x(),
                self.axes_ranges.y(),
                self.axes_ranges.z(),
            )
            .expect("Failed to create a chart")
    }

    fn draw_network_model(
        &self,
        network_model: &NetworkModel,
        chart_context: &mut PlottersChartContext<'a>
    ) {
        self.draw_destinations(network_model, chart_context);
        self.draw_command_device(network_model, chart_context);
        self.draw_devices(network_model, chart_context);
        self.draw_attacker_devices(network_model, chart_context);
    }

    fn draw_chart(&self, chart_context: &mut PlottersChartContext<'a>) {
        chart_context 
            .with_projection(|mut p| {
                p.pitch = self.camera_angle.pitch();
                p.yaw = self.camera_angle.yaw();
                p.into_matrix()
            })
            .configure_axes()
            .axis_panel_style(GREY.mix(0.1))
            .label_style((FONT, self.font_size / 2))
            .draw()
            .expect("Failed to draw a chart");
    }
    
    fn draw_destinations(
        &self, 
        network_model: &NetworkModel,
        chart_context: &mut PlottersChartContext<'a>
    ) {
        let destinations = network_model_destinations(network_model);
        let destination_primitives = destinations
            .iter()
            .map(|destination| 
                destination_primitive(
                    destination, 
                    self.plot_resolution
                )
            );

        chart_context
            .draw_series(destination_primitives)
            .expect("Failed to draw destination points");
    }
    
    fn draw_command_device(
        &self, 
        network_model: &NetworkModel,
        chart_context: &mut PlottersChartContext<'a>
    ) {
        let Some(command_device) = network_model.command_device() else {
            return;
        };
        let primitive = command_device_primitive(
            command_device, 
            self.plot_resolution
        );

        chart_context
            .draw_series([primitive])
            .expect("Failed to draw command device");
    }

    fn draw_devices(
        &self, 
        network_model: &NetworkModel,
        chart_context: &mut PlottersChartContext<'a>
    ) {
        let device_primitives = network_model
            .device_map()
            .devices()
            .map(|device|
                device_primitive(
                    device, 
                    self.device_coloring, 
                    self.plot_resolution
                )
            );

        chart_context
            .draw_series(device_primitives)
            .expect("Failed to draw devices");
    }

    fn draw_attacker_devices(
        &self, 
        network_model: &NetworkModel,
        chart_context: &mut PlottersChartContext<'a>
    ) {
        let attacker_device_primitives = network_model
            .attacker_devices()
            .iter()
            .flat_map(|attacker_device| {
                attacker_device_primitive_on_all_frequencies(
                    attacker_device, 
                    self.plot_resolution
                )
            });

        chart_context
            .draw_series(attacker_device_primitives)
            .expect("Failed to draw attacker devices");
    }
}
