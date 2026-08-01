use std::time::Duration;

use bevy::app::{App, Plugin};
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Query, Res, SystemParam};
use bevy::time::{self, Time};
use egui_plot::PlotPoints;
use traffloat_proto::proto;

use crate::dock;
use crate::scene::{building, conduit, corridor, facility, resident, vehicle};
use crate::util::new_id;

pub struct Plug;

impl Plugin for Plug {
    fn build(&self, _app: &mut App) {}
}

pub struct Tab {
    pub name:        String,
    buffers:         Vec<Buffer>,
    pub buffer_size: usize,
    pub targets:     Vec<Target>,

    pub probe_period: Duration,
    last_probe_time:  Option<Duration>,
}

impl Tab {
    pub fn new(name: String, target: Target) -> Self {
        let buffer_size = 1200;
        Self {
            name,
            buffers: [Buffer { data: Vec::with_capacity(buffer_size), offset: 0 }].into(),
            buffer_size,
            targets: [target].into(),
            probe_period: Duration::from_millis(100),
            last_probe_time: None,
        }
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.push(target);
        self.buffers.push(Buffer { data: Vec::with_capacity(self.buffer_size), offset: 0 });
    }
}

#[derive(SystemParam)]
pub struct BeforeRenderSystemParam<'w, 's> {
    time: Res<'w, Time<time::Virtual>>, // TODO check if this is correct when we support tmultiplayer

    building_query: Query<'w, 's, &'static building::Info>,
    corridor_query: Query<'w, 's, &'static corridor::Info>,
    facility_query: Query<'w, 's, &'static facility::Info>,
    conduit_query:  Query<'w, 's, &'static conduit::Info>,
    resident_query: Query<'w, 's, &'static resident::Info>,
    vehicle_query:  Query<'w, 's, &'static vehicle::Info>,
}

impl dock::Tab for Tab {
    type TitleSystemParam<'w, 's> = ();

    fn title(&self, (): Self::TitleSystemParam<'_, '_>) -> String { format!("Plot: {}", self.name) }

    type UiSystemParam<'w, 's> = ();

    fn ui(&mut self, (): Self::UiSystemParam<'_, '_>, ui: &mut egui::Ui, dock: dock::Context) {
        if self.buffers.iter().all(|buf| buf.data.is_empty()) {
            ui.label("Waiting for initial data...");
            return;
        }
        egui_plot::Plot::new(new_id!(dock.id)).show(ui, |plot_ui| {
            for buf in &self.buffers {
                if buf.data.is_empty() {
                    continue;
                }

                #[expect(
                    clippy::cast_possible_truncation,
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    reason = "all involved numbers are bounded by buf_size"
                )]
                let points = PlotPoints::from_parametric_callback(
                    |t| {
                        let index = t.floor() as usize;
                        let y = buf
                            .get(index)
                            .or_else(|| buf.iter().next_back())
                            .expect("checked !buf.data.is_empty()");
                        let x =
                            (t - (buf.data.len() as f64) + 1.0) * self.probe_period.as_secs_f64();
                        (x, f64::from(y))
                    },
                    0.0..=((buf.data.len() - 1) as f64),
                    buf.data.len(),
                );
                plot_ui.line(egui_plot::Line::new(&self.name, points));
            }
        });
    }

    type OnCloseSystemParam<'w, 's> = ();

    type BeforeRenderSystemParam<'w, 's> = BeforeRenderSystemParam<'w, 's>;

    fn before_render(
        &mut self,
        _contexts: &mut bevy_egui::prelude::EguiContexts,
        param: BeforeRenderSystemParam<'_, '_>,
    ) {
        let last_probe_time = self.last_probe_time.replace(param.time.elapsed());
        let times = if let Some(last_probe_time) = last_probe_time {
            let last = last_probe_time.as_millis() / self.probe_period.as_millis();
            let curr = param.time.elapsed().as_millis() / self.probe_period.as_millis();
            curr.saturating_sub(last)
        } else {
            1
        };
        if times > 0 {
            for (target, buffer) in self.targets.iter().zip(&mut self.buffers) {
                let value = target.resolve_metric(&param).unwrap_or(f32::NAN);
                for _ in 0..times {
                    buffer.push(value);
                }
            }
        }
    }
}

struct Buffer {
    data:   Vec<f32>,
    offset: usize,
}

impl Buffer {
    fn push(&mut self, value: f32) {
        if self.data.len() < self.data.capacity() {
            self.data.push(value);
        } else {
            self.data[self.offset] = value;
            self.offset = (self.offset + 1) % self.data.capacity();
        }
    }

    fn resize(&mut self, new_size: usize) {
        if new_size < self.data.len() {
            let mut new = Vec::with_capacity(new_size);
            if new_size < self.offset {
                new.extend_from_slice(&self.data[self.offset - new_size..self.offset]);
            } else {
                let skip = self.data.len() - new_size;
                new.extend_from_slice(&self.data[self.offset + skip..]);
                new.extend_from_slice(&self.data[..self.offset]);
            }
            self.data = new;
            self.offset = 0;
        } else if new_size > self.data.len() {
            let mut new = Vec::with_capacity(new_size);
            new.extend(self.iter());
            self.data = new;
            self.offset = 0;
        }
    }

    fn get(&self, index: usize) -> Option<f32> {
        if index < self.data.len() {
            self.data.get((self.offset + index) % self.data.len()).copied()
        } else {
            None
        }
    }

    fn iter(&self) -> impl DoubleEndedIterator<Item = f32> + '_ {
        self.data[self.offset..].iter().chain(self.data[..self.offset].iter()).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    BuildingAmbient { building: Entity, metric: FluidMetric },
    CorridorAmbient { corridor: Entity, metric: FluidMetric },
    FacilityStorage { facility: Entity, metric: FluidMetric },
    ReactorEfficiency { facility: Entity },
    PipeStorage { conduit: Entity, metric: FluidMetric },
    ResidentAttr { resident: Entity, ty: usize },
    VehicleCompartmentFluid { vehicle: Entity, compartment: usize, metric: FluidMetric },
}

impl Target {
    fn resolve_metric(&self, param: &BeforeRenderSystemParam<'_, '_>) -> Option<f32> {
        match *self {
            Self::BuildingAmbient { building, metric } => {
                let info = param.building_query.get(building).ok()?;
                metric.resolve(info.ambient_fluid.as_ref()?)
            }
            Self::CorridorAmbient { corridor, metric } => {
                let info = param.corridor_query.get(corridor).ok()?;
                metric.resolve(info.ambient_fluid.as_ref()?)
            }
            Self::FacilityStorage { facility, metric } => {
                let info = param.facility_query.get(facility).ok()?;
                metric.resolve(info.stored_fluid.as_ref()?)
            }
            Self::ReactorEfficiency { facility } => {
                let info = param.facility_query.get(facility).ok()?;
                Some(info.reactor.as_ref()?.efficiency)
            }
            Self::PipeStorage { conduit, metric } => {
                let info = param.conduit_query.get(conduit).ok()?;
                metric.resolve(info.stored_fluid.as_ref()?)
            }
            Self::ResidentAttr { resident, ty } => {
                let info = param.resident_query.get(resident).ok()?;
                info.attributes.get(ty).copied().flatten()
            }
            Self::VehicleCompartmentFluid { vehicle, compartment, metric } => {
                let info = param.vehicle_query.get(vehicle).ok()?;
                let compartment_info = info.compartments.get(compartment)?;
                metric.resolve(compartment_info.fluid.as_ref()?)
            }
        }
    }

    pub fn focused_entities(&self) -> impl IntoIterator<Item = Entity> + '_ {
        match *self {
            Self::BuildingAmbient { building, .. } => Some(building),
            Self::CorridorAmbient { corridor, .. } => Some(corridor),
            Self::FacilityStorage { facility, .. } => Some(facility),
            Self::ReactorEfficiency { facility } => Some(facility),
            Self::PipeStorage { conduit, .. } => Some(conduit),
            Self::ResidentAttr { resident, .. } => Some(resident),
            Self::VehicleCompartmentFluid { vehicle, .. } => Some(vehicle),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidMetric {
    Pressure,
    Temperature,
    Moles { ty: usize },
    MolarConc { ty: usize },
}

impl FluidMetric {
    fn resolve(&self, fluid: &proto::FluidStorageDetail) -> Option<f32> {
        match *self {
            FluidMetric::Pressure => fluid.pressure,
            FluidMetric::Temperature => fluid.temperature,
            FluidMetric::Moles { ty } => fluid.types.as_ref()?.get(ty).copied(),
            FluidMetric::MolarConc { ty } => Some(*fluid.types.as_ref()?.get(ty)? / fluid.volume),
        }
    }
}
