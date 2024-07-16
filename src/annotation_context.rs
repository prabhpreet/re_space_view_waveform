use re_sdk::{Archetype, EntityPath};
use re_types::{archetypes::AnnotationContext, components::ClassId};
use re_viewer_context::{
    AnnotationMap, IdentifiedViewSystem, ResolvedAnnotationInfo, ViewContextSystem,
};

#[derive(Default)]
pub struct AnnotationWaveformContext(pub AnnotationMap);

impl AnnotationWaveformContext {
    pub fn get_annotation(
        &self,
        entity_path: &EntityPath,
        class_id: Option<ClassId>,
    ) -> Option<ResolvedAnnotationInfo> {
        self.0
             .0
            .get(entity_path)
            .map(|a| a.resolved_class_description(class_id).annotation_info())
    }
}

impl IdentifiedViewSystem for AnnotationWaveformContext {
    fn identifier() -> re_viewer_context::ViewSystemIdentifier {
        "AnnotationWaveformContext".into()
    }
}

impl ViewContextSystem for AnnotationWaveformContext {
    fn compatible_component_sets(&self) -> Vec<re_types::ComponentNameSet> {
        vec![AnnotationContext::required_components()
            .iter()
            .map(ToOwned::to_owned)
            .collect()]
    }

    fn execute(
        &mut self,
        ctx: &re_viewer_context::ViewContext<'_>,
        query: &re_viewer_context::ViewQuery<'_>,
    ) {
        self.0.load(
            ctx.viewer_ctx,
            &query.latest_at_query(),
            query.iter_all_entities(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
