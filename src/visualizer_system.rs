use re_log_types::ResolvedTimeRange;
use re_query::{PromiseResult, QueryError};
use re_sdk::Loggable;
use re_space_view::{range_with_blueprint_resolved_data, RangeResultsExt};
use re_types::{
    components::ClassId,
    datatypes::{AnnotationInfo, TimeRange, Utf8},
};
use re_viewer_context::{
    auto_color_egui, auto_color_for_entity_path, IdentifiedViewSystem, VisualizerQueryInfo,
    VisualizerSystem,
};
use std::collections::BTreeMap;

use crate::{
    annotation_context::AnnotationWaveformContext, AnalogPoint, AnalogPoints, DiscreteTransition,
    DiscreteTransitionKind, EventMarker,
};

use super::{types::archetypes::WaveformPoint, WaveformDomain, WaveformEvents, WaveformSeries};

#[derive(Default, Debug)]
pub struct WaveformSystem {
    pub all_series: BTreeMap<WaveformDomain, Vec<WaveformSeries>>,
    pub all_events: WaveformEvents,
}

impl IdentifiedViewSystem for WaveformSystem {
    fn identifier() -> re_viewer_context::ViewSystemIdentifier {
        "WaveformSystem".into()
    }
}

impl VisualizerSystem for WaveformSystem {
    fn visualizer_query_info(&self) -> re_viewer_context::VisualizerQueryInfo {
        VisualizerQueryInfo::from_archetype::<WaveformPoint>()
    }

    fn execute(
        &mut self,
        ctx: &re_viewer_context::ViewContext<'_>,
        query: &re_viewer_context::ViewQuery<'_>,
        context_systems: &re_viewer_context::ViewContextCollection,
    ) -> Result<Vec<re_renderer::QueueableDrawData>, re_viewer_context::SpaceViewSystemExecutionError>
    {
        match self.load_points(ctx, query, context_systems) {
            Ok(_) | Err(QueryError::PrimaryNotFound(_)) => Ok(vec![]),
            Err(e) => Err(e.into()),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_fallback_provider(&self) -> &dyn re_viewer_context::ComponentFallbackProvider {
        self
    }
}

re_viewer_context::impl_component_fallback_provider!(WaveformSystem => []);

impl WaveformSystem {
    fn load_points(
        &mut self,
        ctx: &re_viewer_context::ViewContext<'_>,
        query: &re_viewer_context::ViewQuery<'_>,
        context_systems: &re_viewer_context::ViewContextCollection,
    ) -> Result<(), QueryError> {
        use crate::types::components::*;

        self.all_series = Default::default();
        self.all_events = Default::default();

        let annotation_map = context_systems
            .get::<AnnotationWaveformContext>()
            .map_err(|e| QueryError::BadAccess)?;

        let resolver = ctx.recording().resolver();

        let mut series_iter = query.iter_visible_data_results(ctx, Self::identifier());

        series_iter.try_for_each(|series_result| -> Result<(), QueryError> {
            let entity_path = series_result.entity_path.clone();

            let Some(domain) = entity_path.iter().next() else {
                return Ok(());
            };

            let color = auto_color_for_entity_path(&entity_path);

            let mut min_time = None;
            let mut max_time = None;

            let mut series = WaveformSeries {
                entity_path: entity_path.clone(),
                min_time: i64::MAX,
                max_time: i64::MIN,
                analog_points: AnalogPoints {
                    points: BTreeMap::new(),
                    y_range: None,
                },
                discrete_points: Default::default(),
                color: color.into(),
            };

            let query_range = series_result.query_range();

            //TODO: Limit time range and plotted elements queried to the view- this returns nothing!
            let time_range = ResolvedTimeRange::from_relative_time_range(
                &match query_range {
                    re_viewer_context::QueryRange::TimeRange(range) => range.clone(),
                    re_viewer_context::QueryRange::LatestAt => TimeRange {
                        start: re_types::datatypes::TimeRangeBoundary::AT_CURSOR,
                        end: re_types::datatypes::TimeRangeBoundary::AT_CURSOR,
                    },
                },
                query.latest_at,
            );

            let time_range = ResolvedTimeRange::EVERYTHING;

            let range = re_data_store::RangeQuery::new(query.timeline, time_range);

            let scalar_points_result = range_with_blueprint_resolved_data(
                ctx,
                None,
                &range,
                &series_result,
                [Scalar::name()],
            );

            if let Some(all_scalars) =
                scalar_points_result.get_required_component_dense::<Scalar>(resolver)
            {
                let all_scalars = all_scalars?;

                let entry_range = all_scalars.entry_range();

                if !matches!(
                    all_scalars.status(),
                    (PromiseResult::Ready(()), PromiseResult::Ready(()))
                ) {}

                series.analog_points.points = all_scalars
                    .range_indices(entry_range.clone())
                    .zip(all_scalars.range_data(entry_range))
                    .filter_map(|((time, _), data)| {
                        if data.len() > 1 {
                            return None;
                        } else if data.is_empty() {
                            return None;
                        } else {
                            let time = time.as_i64();
                            min_time = min_time.map(|t: i64| t.min(time)).or(Some(time));
                            max_time = max_time.map(|t: i64| t.max(time)).or(Some(time));

                            let value = data.first().map_or(0.0, |s| s.0 .0);

                            if let Some((y_min_prev, y_max_prev)) = series.analog_points.y_range {
                                series.analog_points.y_range =
                                    Some((y_min_prev.min(value), y_max_prev.max(value)));
                            } else {
                                series.analog_points.y_range = Some((value, value));
                            }

                            return Some((time, AnalogPoint { value }));
                        }
                    })
                    .collect();
            }

            let discrete_points_result = range_with_blueprint_resolved_data(
                ctx,
                None,
                &range,
                &series_result,
                [
                    DiscreteState::name(),
                    DiscreteStateInit::name(),
                    DiscreteStateNormal::name(),
                ],
            );

            if let (Some(all_discretes), Some(all_discrete_init), Some(all_discrete_normal)) = (
                discrete_points_result.get_required_component_dense::<DiscreteState>(resolver),
                discrete_points_result.get_required_component_dense::<DiscreteStateInit>(resolver),
                discrete_points_result
                    .get_required_component_dense::<DiscreteStateNormal>(resolver),
            ) {
                let (all_discretes, all_discretes_init, all_discrete_normal) =
                    (all_discretes?, all_discrete_init?, all_discrete_normal?);

                let (entry_range, discrete_init_entry_range, discrete_normal_entry_range) = (
                    all_discretes.entry_range(),
                    all_discretes_init.entry_range(),
                    all_discrete_normal.entry_range(),
                );

                if !matches!(
                    all_discretes.status(),
                    (PromiseResult::Ready(()), PromiseResult::Ready(()))
                ) {}

                if !matches!(
                    all_discretes_init.status(),
                    (PromiseResult::Ready(()), PromiseResult::Ready(()))
                ) {}

                if !matches!(
                    all_discrete_normal.status(),
                    (PromiseResult::Ready(()), PromiseResult::Ready(()))
                ) {}

                let all_discretes_normal: Vec<ClassId> = all_discrete_normal
                    .range_data(discrete_normal_entry_range)
                    .filter_map(|data| {
                        if data.len() > 1 {
                            return None;
                        } else if data.is_empty() {
                            return None;
                        }

                        let value: Option<&DiscreteStateNormal> = data.first();
                        value.map(|d| d.0.clone())
                    })
                    .collect();

                let discrete_normal = all_discretes_normal.get(0).cloned();

                series.discrete_points.transitions = all_discretes
                    .range_indices(entry_range.clone())
                    .zip(all_discretes.range_data(entry_range))
                    .filter_map(|((time, _), data)| {
                        //Only one discrete point allowed for entity path, ignore if otherwise
                        if data.len() > 1 || data.is_empty() {
                            return None;
                        }

                        let value: Option<DiscreteState> = data.first().cloned();
                        Some((time.clone(), value))
                    })
                    .filter_map(|(time, value)| {
                        let time = time.as_i64();
                        min_time = min_time.map(|t: i64| t.min(time)).or(Some(time));
                        max_time = max_time.map(|t: i64| t.max(time)).or(Some(time));

                        if let Some(DiscreteState(class_id)) = value {
                            if let Some(annotation_info) = annotation_map
                                .get_annotation(&entity_path.clone(), Some(class_id))
                                .annotation_info
                            {
                                let label = annotation_info.label.clone().map(Utf8::into);
                                let kind = if Some(class_id) == discrete_normal {
                                    DiscreteTransitionKind::Line
                                } else {
                                    DiscreteTransitionKind::Box
                                };

                                return Some((
                                    time,
                                    DiscreteTransition {
                                        label,
                                        color: annotation_info_color(&annotation_info),
                                        kind,
                                    },
                                ));
                            }
                        }
                        return None;
                    })
                    .collect();

                let discrete_init_data: Option<_> = all_discretes_init
                    .range_data(discrete_init_entry_range)
                    .find_map(|(data)| {
                        if data.len() > 1 || data.is_empty() {
                            return None;
                        }

                        data.first().cloned()
                    });

                if let Some(DiscreteStateInit(class_id)) = discrete_init_data {
                    if let Some(annotation_info) = annotation_map
                        .get_annotation(&entity_path.clone(), Some(class_id))
                        .annotation_info
                    {
                        let kind = if Some(class_id) == discrete_normal {
                            DiscreteTransitionKind::Line
                        } else {
                            DiscreteTransitionKind::Box
                        };
                        series.discrete_points.init = Some(DiscreteTransition {
                            label: annotation_info.label.clone().map(Utf8::into),
                            color: annotation_info_color(&annotation_info),
                            kind,
                        });
                    }
                }
            }

            let event_points_result = range_with_blueprint_resolved_data(
                ctx,
                None,
                &range,
                &series_result,
                [Event::name()],
            );

            if let Some(all_events) =
                event_points_result.get_required_component_dense::<Event>(resolver)
            {
                let all_events = all_events?;

                let entry_range = all_events.entry_range();

                if !matches!(
                    all_events.status(),
                    (PromiseResult::Ready(()), PromiseResult::Ready(()))
                ) {}

                all_events
                    .range_indices(entry_range.clone())
                    .zip(all_events.range_data(entry_range))
                    .for_each(|((time, _), data)| {
                        if !data.is_empty() {
                            let time = time.as_i64();
                            min_time = min_time.map(|t: i64| t.min(time)).or(Some(time));
                            max_time = max_time.map(|t: i64| t.max(time)).or(Some(time));

                            for Event(class_id) in data.iter() {
                                if let Some(annotation_info) = annotation_map
                                    .get_annotation(&entity_path.clone(), Some(*class_id))
                                    .annotation_info
                                {
                                    self.all_events.push(
                                        time,
                                        EventMarker {
                                            entity_path: entity_path.clone(),
                                            label: annotation_info.label.clone().map(Utf8::into),
                                            color: annotation_info_color(&annotation_info),
                                        },
                                    );
                                }
                            }
                        }
                    });
            }

            if min_time.is_none() && max_time.is_none() {
                //Empty series
                return Ok(());
            } else if min_time.is_none() || max_time.is_none() {
                //Partial series
                let time = min_time.or(max_time);

                min_time = time;
                max_time = time;
            }

            series.min_time = min_time.unwrap();
            series.max_time = max_time.unwrap();

            self.all_series
                .entry(domain.clone())
                .and_modify(|v| {
                    v.push(series.clone());
                })
                .or_insert(vec![series.clone()]);

            Ok(())
        })?;

        Ok(())
    }
}

fn annotation_info_color(annotation_info: &AnnotationInfo) -> egui::Color32 {
    //This is how backup colors are currently auto assigned
    annotation_info
        .color
        .map(|c| c.into())
        .unwrap_or_else(|| auto_color_egui(annotation_info.id))
}
