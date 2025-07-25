use full_palette::{ORANGE, PINK_300, PINK_200};
use plotters::prelude::*;
use plotters::style::RGBColor;

use crate::backend::DESTINATION_RADIUS;
use crate::backend::device::Device;
use crate::backend::mathphysics::{Frequency, Meter, Point3D, Position};
use crate::backend::networkmodel::attack::{AttackerDevice, AttackType};

use super::{
    DeviceColoring, Pixel, PlottersUnit, PlottersPoint3D, PlotResolution, 
    meters_to_pixels, 
};


const COMMAND_CENTER_RADIUS: Meter = 5.0;

const CIRCLE_SIZE_COEF: Pixel = 400;

const PLOTTERS_DESTINATION_COLOR: RGBColor    = YELLOW;
const PLOTTERS_COMMAND_CENTER_COLOR: RGBColor = GREEN;


type PlottersCircle = Circle<(PlottersUnit, PlottersUnit, PlottersUnit), Pixel>; 


#[must_use]
pub fn destination_primitive( 
    destination: &Point3D,
    plot_resolution: PlotResolution
) -> PlottersCircle {
    let point  = PlottersPoint3D::from(destination);
    let radius = meters_to_pixels(
        DESTINATION_RADIUS,
        plot_resolution
    );

    Circle::new(point.into(), radius, PLOTTERS_DESTINATION_COLOR)
}

#[must_use]
pub fn command_device_primitive(
    command_device: &Device,
    plot_resolution: PlotResolution
) -> PlottersCircle {
    let point  = PlottersPoint3D::from(command_device.position());
    let radius = meters_to_pixels(
        COMMAND_CENTER_RADIUS,
        plot_resolution
    );  
    
    Circle::new(point.into(), radius, PLOTTERS_COMMAND_CENTER_COLOR)
}

#[must_use]
pub fn device_primitive(
    device: &Device,
    coloring: DeviceColoring,
    plot_resolution: PlotResolution
) -> PlottersCircle {
    let point = PlottersPoint3D::from(device.position());
    let color = device_color(device, coloring);
    let size  = device_size(plot_resolution); 
    let style = Into::<ShapeStyle>::into(color).filled();

    Circle::new(point.into(), size, style)
}

fn device_color(device: &Device, coloring: DeviceColoring) -> RGBColor {
    match coloring {
        DeviceColoring::Infection            => {
            color_by_infection(device.is_infected())
        },
        DeviceColoring::SingleColor(r, g, b) => {
            RGBColor(r, g, b)
        }
    }
}

fn color_by_infection(infected: bool) -> RGBColor {
    if infected {
        PINK_200
    } else {
        BLACK
    }
}

fn device_size(plot_resolution: PlotResolution) -> Pixel {
    if plot_resolution.width() < CIRCLE_SIZE_COEF {
        return 1;  
    } 

    plot_resolution.width() / CIRCLE_SIZE_COEF
}

#[must_use]
pub fn attacker_device_primitive_on_all_frequencies(
    attacker_device: &AttackerDevice,
    plot_resolution: PlotResolution,
) -> Vec<PlottersCircle> {
    attacker_device
        .device()
        .tx_signal_quality_map()
        .keys()
        .map(|frequency|
            attacker_device_primitive(
                attacker_device, 
                *frequency, 
                plot_resolution
            )
        )
        .collect()
}

#[must_use]
pub fn attacker_device_primitive(
    attacker_device: &AttackerDevice,
    frequency: Frequency,
    plot_resolution: PlotResolution
) -> PlottersCircle {
    let point = PlottersPoint3D::from(
        attacker_device.device().position()
    );
    let radius = attacker_device
        .device()
        .area_radius_on(frequency);
    let attacker_device_coverage = meters_to_pixels(radius, plot_resolution);
    let area_color = attacker_device_area_color(attacker_device, frequency);

    Circle::new(point.into(), attacker_device_coverage, area_color)
}

fn attacker_device_area_color(
    attacker_device: &AttackerDevice,
    frequency: Frequency
) -> RGBColor {
    let spoofs_gps = matches!(
        attacker_device.attack_type(), 
        AttackType::GPSSpoofing(_)
    );
    let spreads_malware = matches!(
        attacker_device.attack_type(), 
        AttackType::MalwareDistribution(_)
    );
    
    match frequency {
        Frequency::GPS if spoofs_gps          => ORANGE,
        Frequency::GPS                        => RED,
        Frequency::Control if spreads_malware => PINK_300,
        Frequency::Control                    => BLUE,
    }
}
