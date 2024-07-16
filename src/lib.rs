pub mod types;
use std::collections::BTreeMap;

use re_sdk::{EntityPath, EntityPathPart};

mod annotation_context;
mod space_view_class;
mod visualizer_system;

pub use space_view_class::WaveformSpaceView;

type WaveformTime = i64;
type WaveformDomain = EntityPathPart;

#[derive(Clone, Debug)]
struct WaveformSeries {
    pub entity_path: EntityPath,
    pub min_time: WaveformTime,
    pub max_time: WaveformTime,
    pub analog_points: AnalogPoints,
    pub discrete_points: DiscretePoints,
    pub color: egui::Color32,
}

impl WaveformSeries {
    pub fn len_series(&self) -> usize {
        self.analog_points.len() + self.discrete_points.len()
    }
}

#[derive(Clone, Debug, Default)]
struct WaveformEvents {
    pub event_markers: BTreeMap<WaveformTime, Vec<EventMarker>>,
}

impl WaveformEvents {
    pub fn len(&self) -> usize {
        self.event_markers.len()
    }

    pub fn get(&self, time: &WaveformTime) -> Option<&Vec<EventMarker>> {
        self.event_markers.get(&time)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&WaveformTime, &Vec<EventMarker>)> {
        self.event_markers.iter()
    }

    pub fn push(&mut self, time: WaveformTime, event_marker: EventMarker) {
        self.event_markers
            .entry(time)
            .or_insert_with(Vec::new)
            .push(event_marker);
    }
}

#[derive(Clone, Debug)]
struct AnalogPoints {
    pub points: BTreeMap<WaveformTime, AnalogPoint>,
    /// Optional min and max values for the analog points when points is non-empty
    pub y_range: Option<(f64, f64)>,
}

impl AnalogPoints {
    pub fn len(&self) -> usize {
        self.points.len()
    }
    pub fn get(&self, time: &WaveformTime) -> Option<&AnalogPoint> {
        self.points.get(&time)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&WaveformTime, &AnalogPoint)> {
        self.points.iter()
    }

    pub fn push(&mut self, time: WaveformTime, value: f64) {
        self.points.insert(time, AnalogPoint { value });
    }
}

#[derive(Clone, Debug, Default)]
struct DiscretePoints {
    ///Transitions denoting events
    pub transitions: BTreeMap<WaveformTime, DiscreteTransition>,
    /// Init state
    pub init: Option<DiscreteTransition>,
}

impl DiscretePoints {
    pub fn len(&self) -> usize {
        self.transitions.len()
    }

    pub fn get_box(&self, time: &WaveformTime) -> Option<&DiscreteTransition> {
        self.transitions.get(&time)
    }

    pub fn iter_box(&self) -> impl Iterator<Item = (&WaveformTime, &DiscreteTransition)> {
        self.transitions.iter()
    }

    pub fn push_box(
        &mut self,
        time: WaveformTime,
        label: Option<String>,
        color: egui::Color32,
        kind: DiscreteTransitionKind,
    ) {
        self.transitions
            .insert(time, DiscreteTransition { label, color, kind });
    }

    pub fn is_empty(&self) -> bool {
        self.transitions.is_empty()
    }
}

#[derive(Clone, Debug)]
struct AnalogPoint {
    pub value: f64,
}

#[derive(Clone, Debug)]
enum DiscreteTransitionKind {
    Line,
    Box,
}

#[derive(Clone, Debug)]
struct DiscreteTransition {
    pub label: Option<String>,
    pub color: egui::Color32,
    pub kind: DiscreteTransitionKind,
}

#[derive(Clone, Debug)]
struct EventMarker {
    pub entity_path: EntityPath,
    pub label: Option<String>,
    pub color: egui::Color32,
}
