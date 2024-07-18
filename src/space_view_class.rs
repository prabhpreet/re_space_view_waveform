use std::{
    collections::{HashMap, HashSet},
    error::Error,
    ops::Bound,
};

use egui::{Color32, Layout, RichText};
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, PlotBounds};
use itertools::Itertools;
use re_format::next_grid_tick_magnitude_ns;
use re_log_types::{TimeInt, TimeType, TimeZone};
use re_space_view::controls;
use re_types::View;
use re_viewer_context::{
    SpaceViewClass, SpaceViewSpawnHeuristics, SpaceViewState, SpaceViewStateExt,
};

use crate::visualizer_system::WaveformSystem;

use super::{annotation_context::AnnotationWaveformContext, DiscreteTransition, WaveformDomain};

type PlotTime = f64;
type PlotPositionIndex = usize;
#[derive(Clone, Default)]
pub struct WaveformSpaceViewState {
    /// Total samples last viewed in the waveform
    last_frame_sample_count: usize,

    /// Secondary marker position for the waveform
    second_marker: Option<PlotTime>,

    /// Domain order index
    domain_index: HashMap<WaveformDomain, PlotPositionIndex>,

    /// Selected mode- displays only entities that are selected
    selected_mode: SelectedMode,
}

#[derive(Debug, Clone, Default)]
pub enum SelectedMode {
    Selected(HashSet<re_log_types::EntityPath>),
    #[default]
    Unselected,
}

impl SelectedMode {
    pub fn selected(&self) -> bool {
        match self {
            SelectedMode::Selected(_) => true,
            SelectedMode::Unselected => false,
        }
    }
    pub fn toggle(&mut self, paths: &HashSet<re_log_types::EntityPath>) {
        *self = match self {
            SelectedMode::Selected(_) => SelectedMode::Unselected,
            SelectedMode::Unselected if !paths.is_empty() => SelectedMode::Selected(paths.clone()),
            SelectedMode::Unselected => SelectedMode::Unselected,
        };
    }

    pub fn filter_path(&self, path: &re_log_types::EntityPath) -> bool {
        match self {
            //Only selected paths are displayed
            SelectedMode::Selected(paths) => paths.contains(path),
            //All paths are displayed
            SelectedMode::Unselected => true,
        }
    }
}

impl SpaceViewState for WaveformSpaceViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug)]
pub struct WaveformView;

impl re_types::SizeBytes for WaveformView {
    fn heap_size_bytes(&self) -> u64 {
        0
    }

    fn is_pod() -> bool {
        true
    }
}

impl re_types::View for WaveformView {
    #[inline]
    fn identifier() -> re_types::SpaceViewClassIdentifier {
        "Waveform".into()
    }
}

type ViewType = WaveformView;

#[derive(Default, Debug, Clone)]
pub struct WaveformSpaceViewDrawError(String);

impl std::fmt::Display for WaveformSpaceViewDrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WaveformSpaceViewDrawError: {}", self.0)
    }
}

impl Error for WaveformSpaceViewDrawError {}

const CURSOR_TIME_TOLERANCE: i64 = 1; //1ns tolerance for cursor time
const X_AXIS_FONT_SIZE_PX: f32 = 16.0; //Font size for x-axis labels (12pt)
const DEFAULT_WAVEFORM_PADDING_PC: f64 = 0.2; //Percentage increase in waveform padding on reset
const DEFAULT_SIDE_PANEL_WIDTH_PC: f32 = 0.3; //Percentage of width side panel occupies
const DISCRETE_ALL_PADDING_PC: f64 = 0.95; //Padding for discrete waveform superimposed on continuous waveform
const DISCRETE_BOX_PADDING_PC: f64 = 0.8; //Padding between discrete box waveforms
const DISCRETE_STROKE_WIDTH_PC: f32 = 0.1; //Percent of stroke width of the discrete box for line
const DISCRETE_STROKE_WIDTH_MIN: f32 = 1.0; //Minimum stroke width for line

#[derive(Default)]
pub struct WaveformSpaceView;

impl SpaceViewClass for WaveformSpaceView {
    fn identifier() -> re_types::SpaceViewClassIdentifier
    where
        Self: Sized,
    {
        ViewType::identifier()
    }

    fn display_name(&self) -> &'static str {
        "Waveform"
    }

    fn help_text(&self, egui_ctx: &egui::Context) -> egui::WidgetText {
        let mut layout = re_ui::LayoutJobBuilder::new(egui_ctx);

        layout.add("Pan by dragging, or scroll (+ ");
        layout.add(controls::HORIZONTAL_SCROLL_MODIFIER);
        layout.add(" for horizontal).\n");

        layout.add("Zoom with pinch gesture or scroll + ");
        layout.add(controls::ZOOM_SCROLL_MODIFIER);
        layout.add(".\n");

        layout.add("Scroll + ");
        layout.add(controls::ASPECT_SCROLL_MODIFIER);
        layout.add(" to zoom only the temporal axis while holding the y-range fixed.\n");

        layout.add("Drag ");
        layout.add(controls::SELECTION_RECT_ZOOM_BUTTON);
        layout.add(" to zoom in/out using a selection.\n");

        layout.add_button_text(controls::RESET_VIEW_BUTTON_TEXT);
        layout.add(" to reset the view.\n");

        layout.add(egui::Modifiers {
            shift: true,
            ..Default::default()
        });
        layout.add("+ ");
        layout.add(egui::PointerButton::Primary);
        layout.add(" to set the timeline cursor.\n");

        layout.add(egui::Modifiers {
            shift: true,
            ..Default::default()
        });
        layout.add("+ ");
        layout.add(egui::PointerButton::Secondary);
        layout.add(" to set a secondary timeline marker.\n");

        layout.add(egui::Modifiers {
            ctrl: true,
            ..Default::default()
        });
        layout.add("+ ");
        layout.add(egui::PointerButton::Primary);
        layout.add(" to select multiple waveforms.\n");

        layout.add(egui::Modifiers {
            ctrl: true,
            ..Default::default()
        });
        layout.add("+ ");
        layout.add(egui::Key::Enter);
        layout.add(" to toggle selected mode once waveforms have been selected.\n");

        layout.layout_job.into()
    }

    fn on_register(
        &self,
        system_registry: &mut re_viewer_context::SpaceViewSystemRegistrator<'_>,
    ) -> Result<(), re_viewer_context::SpaceViewClassRegistryError> {
        system_registry.register_context_system::<AnnotationWaveformContext>()?;

        system_registry.register_visualizer::<WaveformSystem>()?;

        Ok(())
    }

    fn new_state(&self) -> Box<dyn SpaceViewState> {
        Box::<WaveformSpaceViewState>::default()
    }

    fn layout_priority(&self) -> re_viewer_context::SpaceViewClassLayoutPriority {
        re_viewer_context::SpaceViewClassLayoutPriority::High
    }

    fn spawn_heuristics(
        &self,
        _ctx: &re_viewer_context::ViewerContext<'_>,
    ) -> re_viewer_context::SpaceViewSpawnHeuristics {
        SpaceViewSpawnHeuristics::root()
    }

    fn ui(
        &self,
        ctx: &re_viewer_context::ViewerContext<'_>,
        ui: &mut egui::Ui,
        state: &mut dyn SpaceViewState,
        query: &re_viewer_context::ViewQuery<'_>,
        system_output: re_viewer_context::SystemExecutionOutput,
    ) -> Result<(), re_viewer_context::SpaceViewSystemExecutionError> {
        let WaveformSpaceViewState {
            last_frame_sample_count,
            second_marker,
            domain_index,
            selected_mode,
        } = state.downcast_mut::<WaveformSpaceViewState>()?;

        //Global inputs

        let (mut current_time, time_type, timeline) = {
            // Avoid holding the lock for long
            let time_ctrl = ctx.rec_cfg.time_ctrl.read();
            let current_time = time_ctrl.time_i64();
            let time_type = time_ctrl.time_type();
            let timeline = *time_ctrl.timeline();
            (current_time, time_type, timeline)
        };

        let reset_click = ui.input(|i| {
            i.pointer
                .button_double_clicked(egui::PointerButton::Primary)
        });

        let ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
        let shift_pressed = ui.ctx().input(|i| i.modifiers.shift);
        let space_pressed = ui.ctx().input(|i| i.key_pressed(egui::Key::Space));

        // Global effects from inputs

        let mut plot_item_id_to_entity_path = HashMap::new();
        let mut hovered_entity_paths = HashSet::new();
        let mut selected_entity_paths = HashSet::new();

        ctx.hovered().iter().for_each(|(item, _item_space_ctx)| {
            if let Some(entity_path) = item.entity_path() {
                hovered_entity_paths.insert(entity_path.clone());
            }
        });

        ctx.selection().iter().for_each(|(item, _item_space_ctx)| {
            if let Some(entity_path) = item.entity_path() {
                selected_entity_paths.insert(entity_path.clone());
            }
        });

        //Toggle selected mode on ctrl+ space
        if ctrl_pressed && space_pressed {
            selected_mode.toggle(&selected_entity_paths);
        }

        //Change time and second_marker on shift+click
        let timeline_click_mode = shift_pressed;

        //Cursor lookup mode on shift or ctrl
        let lookup_cursor_in_panel = ctrl_pressed || shift_pressed;

        //Local defaults
        let timeline_trace_color = egui::Color32::from_rgb(255, 140, 0);

        let timeline_name = timeline.name().to_string();

        let WaveformSystem {
            all_series,
            all_events,
        } = system_output.view_systems.get::<WaveformSystem>()?;

        let min_time = all_series
            .iter()
            .flat_map(|(_, series)| series.iter().map(|s| s.min_time))
            .min()
            .unwrap_or(0);

        let max_time = all_series
            .iter()
            .flat_map(|(_, series)| series.iter().map(|s| s.max_time))
            .max()
            .unwrap_or(min_time);

        let current_sample_count: usize = all_series
            .iter()
            .flat_map(|(_, series)| series.iter().map(|s| s.len_series()))
            .sum::<usize>()
            + all_events.len();

        // …then use that as an offset to avoid nasty precision issues with
        // large times (nanos since epoch does not fit into a f64).
        let time_offset = if timeline.typ() == TimeType::Time {
            // In order to make the tick-marks on the time axis fall on whole days, hours, minutes etc,
            // we need to round to a whole day:
            round_ns_to_start_of_day(min_time)
        } else {
            min_time
        };

        let time_zone_for_timestamps = ctx.app_options.time_zone;

        // Ensure all entries have an index in domain_index or get a new index (len)
        for (domain, _) in all_series.iter() {
            let latest_len = domain_index.len();
            domain_index.entry(domain.clone()).or_insert(latest_len);
        }

        // Convert all_series to vec
        let mut all_series: Vec<_> = all_series
            .into_iter()
            .map(|(d, s)| {
                (
                    d,
                    s.into_iter()
                        .filter(|s| selected_mode.filter_path(&s.entity_path))
                        .collect::<Vec<_>>(),
                )
            })
            .filter(|(_, s)| s.iter().any(|s| s.len_series() > 0))
            .collect();

        // Sort all_series by domain_index

        all_series.sort_by(|(a, _), (b, _)| domain_index[a].cmp(&domain_index[b]));

        let has_new_samples = current_sample_count != *last_frame_sample_count;

        let mut lookup_cursor_value = None;

        let pixels_per_point = ui.ctx().pixels_per_point();
        let axis_height = X_AXIS_FONT_SIZE_PX / pixels_per_point;
        let plot_height = (ui.available_height() - (2.0 * axis_height)) / (all_series.len() as f32);

        let plot_width = ui.available_width();
        let side_panel_width = plot_width * DEFAULT_SIDE_PANEL_WIDTH_PC;
        let plot_width = plot_width - side_panel_width;

        let axis_group_id = egui::Id::new(("axis_group", query.space_view_id, &timeline_name));

        let cursor_group_id = egui::Id::new(("cursor_group", query.space_view_id, &timeline_name));

        if all_series.is_empty() {
            ui.label("No data available");
            return Ok(());
        }
        let last_time_index = all_series.len() - 1;

        let min_x = (min_time - time_offset) as f64;
        let max_x = (max_time - time_offset) as f64;
        let current_time_copy = current_time;
        let second_marker_plot_copy = *second_marker;

        ui.horizontal_centered(|ui| {
            ui.style_mut().spacing.item_spacing = [0.0, 0.0].into();
            let mut plot_heights = vec![];

            ui.vertical(|ui| -> Result<(), re_viewer_context::SpaceViewSystemExecutionError> {
                        let bounds_data: Vec<_> = all_series
                            .iter()
                            .map(|(_domain, domain_series)| {
                                let min_y = domain_series.iter().fold(None, |i: Option<f64>, s| {
                                    if let Some(i) = i {
                                        s.analog_points.y_range.map(|(min, _)| i.min(min))
                                    } else {
                                        s.analog_points.y_range.map(|(min, _)| min)
                                    }
                                });

                                let max_y = domain_series.iter().fold(None, |i: Option<f64>, s| {
                                    if let Some(i) = i {
                                        s.analog_points.y_range.map(|(_, max)| i.max(max))
                                    } else {
                                        s.analog_points.y_range.map(|(_, max)| max)
                                    }
                                });

                                let (min_y, max_y, any_analog_points) = match (min_y, max_y) {
                                    (None, None) => (-1.0, 1.0, false),
                                    (None, Some(v)) => (v - 0.5, v + 0.5, true),
                                    (Some(v), None) => (v - 0.5, v + 0.5, true),
                                    (Some(x), Some(y)) if x < y => (x, y, true),
                                    (Some(x), Some(y)) if x == y => (x - 0.5, x + 0.5, true),
                                    _ => {
                                        return Err(
                                            re_viewer_context::SpaceViewSystemExecutionError::DrawDataCreationError(
                                                Box::new(WaveformSpaceViewDrawError(
                                                    "Invalid analog y range".to_string(),
                                                )),
                                            ),
                                        );
                                    }
                                };

                                let new_plot_bounds = if reset_click || has_new_samples {
                                    let spread_y = max_y - min_y;
                                    let delta = spread_y * DEFAULT_WAVEFORM_PADDING_PC;

                                    let min_y_new = min_y - delta;
                                    let max_y_new = max_y + delta;

                                    let new_bounds =
                                        egui_plot::PlotBounds::from_min_max([min_x, min_y_new], [max_x, max_y_new]);
                                    Some(new_bounds)
                                } else {
                                    None
                                };

                                Ok((any_analog_points, new_plot_bounds))
                            })
                            .try_collect()?;

                ui.spacing_mut().item_spacing = [0.0, 0.0].into();
                ui.set_width(plot_width);

                for (i, (domain, domain_series)) in all_series.iter().enumerate() {
                    let (any_analog_points, new_plot_bounds) = bounds_data[i];
                    let height = ui
                        .horizontal(|ui| {
                            let plot_id = ("plot", query.space_view_id, &timeline_name, &domain);
                            if i == 0 || i == last_time_index {
                                ui.set_height(plot_height + axis_height);
                            } else {
                                ui.set_height(plot_height);
                            }
                            ui.spacing_mut().item_spacing = [0.0, 0.0].into();


                            let mut plot = egui_plot::Plot::new(plot_id)
                                .min_size([0.0, 0.0].into())
                                //TODO: Turn this on once we figure out how to set bounds on viewport reset
                                //.auto_bounds([false, false].into())
                                .allow_double_click_reset(false)
                                .allow_scroll([true, false])
                                .allow_zoom([true, true])
                                .allow_drag([true, true])
                                .link_axis(axis_group_id, true, false)
                                .link_cursor(cursor_group_id, true, false)
                                .label_formatter(|_name, value| {
                                    let timezone_now = time_type.format(
                                        TimeInt::new_temporal(
                                            (value.x as i64).saturating_add(time_offset),
                                        ),
                                        time_zone_for_timestamps,
                                    );
                                    if let Some(current_time) = current_time_copy {
                                        let out_str = if any_analog_points {
                                           format!("{}\n", value.y)
                                        } else {
                                            "".to_string()
                                        };

                                        let out_str =  format!("{out_str}{timezone_now}\n");
                                        let out_str = if let Some(delta_timeline) = (value.x as i64)
                                            .checked_add(time_offset)
                                            .map(|t| t.checked_sub(current_time)).flatten()
                                        {
                                            let delta_timeline = format_time(
                                                time_type,
                                                delta_timeline.abs(),
                                                time_zone_for_timestamps,
                                            );

                                            format!("{out_str}ΔT: {delta_timeline}\n")
                                        } else {
                                            out_str
                                        };

                                        if let Some(second_marker_delta) =
                                            second_marker_plot_copy.map(|d| {
                                                (value.x - d) as i64
                                            })
                                        {
                                            format!(
                                                "{out_str}ΔM: {}\n",
                                                format_time(
                                                    time_type,
                                                    second_marker_delta.abs(),
                                                    time_zone_for_timestamps
                                                )
                                            )
                                        } else {
                                            out_str
                                        }
                                    } else {
                                        format!("{timezone_now}")
                                    }
                                });

                            if !any_analog_points {
                                plot = plot.auto_bounds([false, true].into());
                            }

                            plot = match i {
                                i if i == 0 || i == last_time_index => plot
                                    .show_axes([true, false])
                                    .x_axis_formatter(move |time, _| {
                                        format_time(
                                            time_type,
                                            (time.value as i64).saturating_add(time_offset),
                                            time_zone_for_timestamps,
                                        )
                                    }),
                                _ => plot.show_axes([false, false]),
                            };

                            plot = match i {
                                0 => plot.x_axis_position(egui_plot::VPlacement::Top),
                                i if i == last_time_index => {
                                    plot.x_axis_position(egui_plot::VPlacement::Bottom)
                                }
                                _ => plot,
                            };

                            if timeline.typ() == TimeType::Time {
                                let canvas_size = ui.available_size();
                                plot = plot.x_grid_spacer(move |spacer| {
                                    ns_grid_spacer(canvas_size, &spacer)
                                });
                            }

                            let egui_plot::PlotResponse {
                                inner: _,
                                response,
                                transform: _,
                                hovered_plot_item,
                            } = plot.show(ui, |plot_ui| {
                                let mut current_bounds = plot_ui.plot_bounds();
                                if let Some(new_bounds) = new_plot_bounds {
                                    plot_ui.set_plot_bounds(new_bounds);
                                    current_bounds = new_bounds;
                                }

                                //Cursor x coordinate in plot domain
                                let pointer_pl_x = plot_ui .pointer_coordinate().map(|p| p.x);

                                let pointer_y = plot_ui.pointer_coordinate().map(|p| p.y);

                                let pointer_wf_x = pointer_pl_x
                                    .map(|p| (p as i64 + time_offset));


                                if timeline_click_mode {
                                    current_time = if let (true, Some(pointer_x)) =
                                        (plot_ui.response().clicked(), pointer_wf_x)
                                    {
                                        {
                                            let mut time_ctrl_write = ctx.rec_cfg.time_ctrl.write();
                                            let timeline = *time_ctrl_write.timeline();
                                            time_ctrl_write.set_timeline_and_time(timeline, pointer_x);
                                            time_ctrl_write.pause();
                                        }
                                        {
                                            let time_ctrl = ctx.rec_cfg.time_ctrl.read();
                                            time_ctrl.time_i64()
                                        }
                                    } else {
                                        current_time_copy
                                    };


                                    if plot_ui.response().secondary_clicked() {
                                        *second_marker = pointer_pl_x;
                                    }
                                }

                                if lookup_cursor_in_panel {
                                    lookup_cursor_value = pointer_wf_x;
                                }

                                // Plot timeline trace
                                if let Some(x) = current_time {
                                    let x = x.saturating_sub(time_offset) as f64;
                                    plot_ui.vline(
                                        egui_plot::VLine::new(x as f64)
                                            .color(timeline_trace_color)
                                            .highlight(true),
                                    );
                                }

                                //Plot cursor marker
                                if let Some(x) = second_marker {
                                    plot_ui.vline(
                                        egui_plot::VLine::new(*x)
                                            .color(egui::Color32::YELLOW)
                                            .style(egui_plot::LineStyle::dashed_dense()),
                                    );
                                }

                                // Plot analog points
                                for series in domain_series.iter() {
                                    let analog_points = series
                                        .analog_points
                                        .iter()
                                        .map(|(t, a)| [(t - time_offset) as f64, a.value])
                                        .collect_vec();

                                    let highlight =  selected_entity_paths.get(&series.entity_path).is_some();

                                    let color = color_hover(series.color, hovered_entity_paths.get(&series.entity_path).is_some());

                                    let analog_line_id =
                                        egui::Id::new(("analog", series.entity_path.hash()));

                                    plot_ui.line(
                                        egui_plot::Line::new(analog_points)
                                            .name(series.entity_path.to_string())
                                            .color(color)
                                            .id(analog_line_id)
                                            .highlight(highlight),
                                    );

                                    plot_item_id_to_entity_path.insert(analog_line_id, series.entity_path.clone());
                                }

                                let discrete_bounds = if any_analog_points {
                                    current_bounds
                                } else {
                                    PlotBounds::from_min_max(
                                        [current_bounds.min()[0], -1.0],
                                        [current_bounds.max()[0], 1.0],
                                    )
                                };
                                let total_height = discrete_bounds.height().abs();

                                // Discrete series is not empty
                                let discrete_series_of_interest =
                                    domain_series.iter().filter_map(|series| {
                                        if series.discrete_points.is_empty() {
                                            None
                                        } else {
                                            Some(series)
                                        }
                                    });

                                let discrete_series_count =
                                    discrete_series_of_interest.clone().count();
                                let discrete_all_padding_height =
                                    total_height * DISCRETE_ALL_PADDING_PC;
                                let discrete_series_height_step =
                                    discrete_all_padding_height / (discrete_series_count as f64);
                                let discrete_series_box_width =
                                    discrete_series_height_step * DISCRETE_BOX_PADDING_PC;
                                let discrete_series_max = discrete_bounds.max()[1] - (discrete_series_height_step/2.0);

                                for (l, series) in discrete_series_of_interest.enumerate() {
                                    let y_offset = discrete_series_max
                                        - (l as f64 * discrete_series_height_step);

                                    let init = if let Some(init) = &series.discrete_points.init {
                                       vec![(&min_time,init)]
                                    }
                                    else {
                                        vec![]
                                    };

                                    let box_elements = init.into_iter().chain(series
                                        .discrete_points
                                        .iter_box())
                                        .chain([(&max_time, &DiscreteTransition{label: None, color: Color32::TRANSPARENT, kind: crate::DiscreteTransitionKind::Line})]) // Plot to the max time to represent state
                                        .tuple_windows::<(_, _)>()
                                        .filter_map(
                                            |(
                                                (t, DiscreteTransition { label, color: c , kind}),
                                                (t_end, _),
                                            )| {
                                                let t = (*t - time_offset) as f64;
                                                let t_end = (*t_end - time_offset) as f64;
                                                let color = color_hover(*c, hovered_entity_paths.get(&series.entity_path).is_some());
                                                let highlight = selected_entity_paths.get(&series.entity_path).is_some();
                                                let stroke_width = (discrete_series_box_width as f32*DISCRETE_STROKE_WIDTH_PC).max(DISCRETE_STROKE_WIDTH_MIN);

                                                match kind {
                                                    crate::DiscreteTransitionKind::Box =>

                                                Some(
                                                    egui_plot::BoxElem::new(
                                                        y_offset,
                                                        egui_plot::BoxSpread::new(
                                                            t, t, t, t_end, t_end,
                                                        ),
                                                    )
                                                    .name(if let Some(label) = label {
                                                        format!(
                                                            "{}:{}",
                                                            series.entity_path.to_string(),
                                                            label.as_str()
                                                        )
                                                    } else {
                                                        format!(
                                                            "{}",
                                                            series.entity_path.to_string(),
                                                        )
                                                    })
                                                    .fill(color)
                                                    .box_width(discrete_series_box_width)
                                                    .stroke(egui::Stroke::new(
                                                        stroke_width,
                                                        series.color,
                                                    )),
                                                ),
                                                    crate::DiscreteTransitionKind::Line => {

                                                        let id = egui::Id::new(("discrete_line", series.entity_path.hash()));

                                                        let points = vec![[min_x, y_offset], [max_x, y_offset]];
                                                        plot_ui.line(Line::new(points)
                                                            .id(id)
                                                            .color(color)
                                                            .stroke(egui::Stroke::new(stroke_width, color))
                                                            .highlight(highlight)
                                                            .allow_hover(true)
                                                        );

                                                        plot_item_id_to_entity_path.insert(id, series.entity_path.clone());
                                                        None
                                                    },
                                                }
                                            },
                                        )
                                        .collect_vec();

                                    if !box_elements.is_empty() {
                                        let highlight =  selected_entity_paths.get(&series.entity_path).is_some();
                                        let id = egui::Id::new(("discrete_box", series.entity_path.hash()));

                                        plot_ui.box_plot(
                                            egui_plot::BoxPlot::new(box_elements)
                                                .id(id)
                                                .horizontal()
                                                .element_formatter(Box::new(|elem, _s| {
                                                    elem.name.clone()
                                                }))
                                                .highlight(highlight)
                                            ,
                                        );

                                        plot_item_id_to_entity_path.insert(id, series.entity_path.clone());
                                    }
                                }


                                //Plot event markers
                                for (t, events) in all_events.iter() {
                                    for (i,event_marker) in events.iter().enumerate() {
                                        let highlight = selected_entity_paths.get(&event_marker.entity_path).is_some() || hovered_entity_paths.get(&event_marker.entity_path).is_some();

                                        let id = egui::Id::new(("event_marker",t, i ));
                                        let x = (*t - time_offset) as f64;

                                        if let (Some(pointer_y), Some(label), true, true) = (pointer_y, &event_marker.label, highlight, plot_ui.response().hovered()) {
                                            plot_ui.text(egui_plot::Text::new([x, pointer_y].into(), RichText::new(label).color(event_marker.color)));
                                        }


                                        plot_ui.vline(
                                            egui_plot::VLine::new(x)
                                                .id(id)
                                                .color(event_marker.color)
                                                .style(egui_plot::LineStyle::dashed_loose())
                                                .highlight(highlight)
                                        );

                                        plot_item_id_to_entity_path.insert(id, event_marker.entity_path.clone());
                                    }
                                }

                            });

                            if !reset_click {
                                if let Some(hovered_entity_path) = hovered_plot_item.and_then(|item| plot_item_id_to_entity_path.get(&item)) {
                                    hovered_entity_paths.insert(hovered_entity_path.clone());

                                    ctx.select_hovered_on_click(&response, re_viewer_context::Item::DataResult(query.space_view_id, hovered_entity_path.clone().into()));
                                }
                                else {
                                   ctx.select_hovered_on_click(&response, re_viewer_context::Item::SpaceView(query.space_view_id));
                                }
                            }

                        })
                        .response
                        .rect
                        .height();

                    plot_heights.push(height);
                }
                Ok(())
            });

            let frame_stroke_color = if timeline_click_mode {
                egui::Color32::DARK_GRAY.lerp_to_gamma(
                timeline_trace_color, 0.7)
            }
            else {
                egui::Color32::DARK_GRAY
            };


            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.set_width(side_panel_width);
                ui.style_mut().spacing.item_spacing = [0.0, 0.0].into();

                ui.with_layout(Layout::top_down(egui::Align::Center),|ui| {
                    ui.set_height(axis_height);
                    if selected_mode.selected(){
                        ui.label( RichText::new("SELECTED MODE").strong());
                    }
                    else if timeline_click_mode {
                        ui.label( RichText::new("TIME CURSOR MODE").strong());
                    }
                });
                for (i, (_domain, domain_series)) in all_series.iter().enumerate() {
                    egui::Frame::none()
                        .stroke(egui::Stroke::new(1.0, frame_stroke_color))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(plot_height);
                            ui.style_mut().spacing.item_spacing = [0.0, 0.0].into();
                            ui.push_id(("table", i), |ui| {
                                TableBuilder::new(ui)
                                    .striped(true)
                                    .column(Column::initial(side_panel_width / 2.0))
                                    .column(Column::initial(side_panel_width / 2.0))
                                    .min_scrolled_height(plot_height)
                                    .max_scroll_height(plot_height)
                                    .cell_layout(egui::Layout::top_down(egui::Align::Center))
                                    .sense(egui::Sense::click().union(egui::Sense::hover()))
                                    .body(|mut body| {
                                        if let Some(seek_time) =
                                            lookup_cursor_value.or(current_time)
                                        {
                                            for series in domain_series.iter() {
                                                let mut is_interpolated = false;
                                                let analog_point = series
                                                    .analog_points
                                                    .points
                                                    .range((
                                                        Bound::Included(
                                                            &(seek_time - CURSOR_TIME_TOLERANCE),
                                                        ),
                                                        Bound::Included(
                                                            &(seek_time + CURSOR_TIME_TOLERANCE),
                                                        ),
                                                    ))
                                                    .next()
                                                    .map(|p| p.1.value)
                                                    //Lazy evaluation
                                                    .or_else(|| {
                                                        let point_before = series
                                                            .analog_points
                                                            .points
                                                            .range((
                                                                Bound::Unbounded,
                                                                Bound::Excluded(&seek_time),
                                                            ))
                                                            .last();

                                                        let point_after = series
                                                            .analog_points
                                                            .points
                                                            .range((
                                                                Bound::Excluded(&seek_time),
                                                                Bound::Unbounded,
                                                            ))
                                                            .next();

                                                        if let (Some((t1, p1)), Some((t2, p2))) =
                                                            (point_before, point_after)
                                                        {
                                                            let t1 = *t1 as f64;
                                                            let t2 = *t2 as f64;

                                                            //Slope
                                                            let slope =
                                                                (p2.value - p1.value) / (t2 - t1);

                                                            let value = p1.value
                                                                + (slope * (seek_time as f64 - t1));

                                                            is_interpolated = true;
                                                            Some(value)
                                                        } else {
                                                            None
                                                        }
                                                    });

                                                let discrete_point = series
                                                    .discrete_points
                                                    .transitions
                                                    .range((
                                                        Bound::Unbounded,
                                                        Bound::Included(&seek_time),
                                                    ))
                                                    .last();

                                                body.row(12.0, |mut row| {
                                                    let hovered = hovered_entity_paths
                                                        .get(&series.entity_path)
                                                        .is_some();

                                                    let selected_path = selected_entity_paths
                                                        .get(&series.entity_path)
                                                        .is_some();

                                                    if selected_path {
                                                       row.set_selected(selected_path);
                                                    }

                                                    let mut responses = vec![];

                                                    let (_, response) = row.col(|ui| {
                                                        ui.style_mut().wrap_mode =
                                                            Some(egui::TextWrapMode::Truncate);

                                                        let text_color = series.color ;
                                                        responses.push(
                                                        ui.label(
                                                            RichText::new(series.entity_path.clone())
                                                            .color(tcolor_hover(text_color, hovered)),
                                                        ));
                                                    });

                                                    responses.push(response);

                                                    let (_, response) =  row.col(|ui| { ui.style_mut().wrap_mode =
                                                            Some(egui::TextWrapMode::Truncate);

                                                        let mut labels = vec![];
                                                        if let Some(analog_point) = analog_point {
                                                            //Limited to 3rd decimal place precision
                                                            labels.push(
                                                                RichText::new(format!(
                                                                    "{:.3} {}",
                                                                    analog_point,
                                                                    if is_interpolated {
                                                                        " (I)"
                                                                    } else {
                                                                        ""
                                                                    }

                                                                ))
                                                                .color(tcolor_hover(series.color, hovered)),
                                                            );
                                                        }

                                                        if let Some((
                                                            &t,
                                                            DiscreteTransition {
                                                                label: Some(label),
                                                                color,
                                                                kind: _
                                                            },
                                                        )) = discrete_point
                                                        {
                                                            if !labels.is_empty() {
                                                                labels.push(RichText::new(" | "));
                                                            }
                                                            labels.push(
                                                                RichText::new(label.clone()).color(tcolor_hover(*color, hovered))
                                                            );

                                                            labels.push(
                                                                RichText::new(
                                                                    format!(" ({})",
                                                                    time_type.format(
                                                                        TimeInt::new_temporal(t,),
                                                                        time_zone_for_timestamps,
                                                                    ))
                                                                )
                                                                .color(tcolor_hover( Color32::DARK_GRAY, hovered)));
                                                        }

                                                        let response = ui.horizontal(|ui| {
                                                        for label in labels {
                                                            responses.push( ui.label(label));
                                                        }
                                                        }).response;
                                                        responses.push(response);
                                                    });

                                                    responses.push(response);
                                                    let response = responses.into_iter().fold(row.response(),
                                                        |mut acc, r| {
                                                            acc = acc.union(r);
                                                            acc
                                                    });

                                                    ctx.select_hovered_on_click(&response, re_viewer_context::Item::DataResult(query.space_view_id,series.entity_path.clone().into()));

                                                });
                                            }
                                        }
                                    });
                            });
                        });
                }
                ui.with_layout(Layout::top_down(egui::Align::Center),|ui| {
                    ui.set_height(axis_height);

                    if let Some(second_marker_delta) = second_marker_plot_copy.map(|m| (m as i64).checked_add(time_offset)).flatten().map(|m| current_time.map(|c| c.checked_sub(m)).flatten()).flatten() {
                    ui.label(format!(
                        "T-M: {}",
                        format_time(
                            time_type,
                            second_marker_delta,
                            time_zone_for_timestamps
                        )
                    ));
}
                });
            });
        });

        *last_frame_sample_count = current_sample_count;

        Ok(())
    }
}

fn format_time(time_type: TimeType, time_int: i64, time_zone_for_timestamps: TimeZone) -> String {
    if time_type == TimeType::Time {
        let time = re_log_types::Time::from_ns_since_epoch(time_int);
        time.format_time_compact(time_zone_for_timestamps)
    } else {
        time_type.format(TimeInt::new_temporal(time_int), time_zone_for_timestamps)
    }
}

fn ns_grid_spacer(
    canvas_size: egui::Vec2,
    input: &egui_plot::GridInput,
) -> Vec<egui_plot::GridMark> {
    let minimum_medium_line_spacing = 150.0; // ≈min size of a label
    let max_medium_lines = canvas_size.x as f64 / minimum_medium_line_spacing;

    let (min_ns, max_ns) = input.bounds;
    let width_ns = max_ns - min_ns;

    let mut small_spacing_ns = 1;
    while width_ns / (next_grid_tick_magnitude_ns(small_spacing_ns) as f64) > max_medium_lines {
        let next_ns = next_grid_tick_magnitude_ns(small_spacing_ns);
        if small_spacing_ns < next_ns {
            small_spacing_ns = next_ns;
        } else {
            break; // we've reached the max
        }
    }
    let medium_spacing_ns = next_grid_tick_magnitude_ns(small_spacing_ns);
    let big_spacing_ns = next_grid_tick_magnitude_ns(medium_spacing_ns);

    let mut current_ns = (min_ns.floor() as i64) / small_spacing_ns * small_spacing_ns;
    let mut marks = vec![];

    while current_ns <= max_ns.ceil() as i64 {
        let is_big_line = current_ns % big_spacing_ns == 0;
        let is_medium_line = current_ns % medium_spacing_ns == 0;

        let step_size = if is_big_line {
            big_spacing_ns
        } else if is_medium_line {
            medium_spacing_ns
        } else {
            small_spacing_ns
        };

        marks.push(egui_plot::GridMark {
            value: current_ns as f64,
            step_size: step_size as f64,
        });

        if let Some(new_ns) = current_ns.checked_add(small_spacing_ns) {
            current_ns = new_ns;
        } else {
            break;
        };
    }

    marks
}

fn round_ns_to_start_of_day(ns: i64) -> i64 {
    let ns_per_day = 24 * 60 * 60 * 1_000_000_000;
    (ns + ns_per_day / 2) / ns_per_day * ns_per_day
}

fn color_hover(color: Color32, highlight: bool) -> Color32 {
    if highlight {
        color.lerp_to_gamma(Color32::WHITE, 0.2)
    } else {
        color
    }
}

fn tcolor_hover(color: Color32, highlight: bool) -> Color32 {
    if highlight {
        color.lerp_to_gamma(Color32::WHITE, 0.2)
    } else {
        color
    }
}
