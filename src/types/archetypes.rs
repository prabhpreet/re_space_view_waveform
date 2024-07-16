use re_types::ComponentName;
#[derive(Clone, Debug, PartialEq)]
pub enum WaveformPoint {
    Scalar(super::components::Scalar),
    DiscreteState(super::components::DiscreteState),
    DiscreteStateInit(super::components::DiscreteStateInit),
    DiscreteStateNormal(super::components::DiscreteStateNormal),
    Event(super::components::Event),
}
impl WaveformPoint {
    #[inline]
    pub fn new_scalar(scalar: impl Into<super::components::Scalar>) -> Self {
        WaveformPoint::Scalar(scalar.into())
    }

    #[inline]
    pub fn new_discrete_state(state: impl Into<super::components::DiscreteState>) -> Self {
        WaveformPoint::DiscreteState(state.into())
    }

    #[inline]
    pub fn new_discrete_state_init(
        state_init: impl Into<super::components::DiscreteStateInit>,
    ) -> Self {
        WaveformPoint::DiscreteStateInit(state_init.into())
    }

    #[inline]
    pub fn new_discrete_state_normal(
        state_normal: impl Into<super::components::DiscreteStateNormal>,
    ) -> Self {
        WaveformPoint::DiscreteStateNormal(state_normal.into())
    }

    #[inline]
    pub fn new_event(event: impl Into<super::components::Event>) -> Self {
        WaveformPoint::Event(event.into())
    }
}

impl From<super::components::Scalar> for WaveformPoint {
    #[inline]
    fn from(value: super::components::Scalar) -> Self {
        WaveformPoint::Scalar(value)
    }
}

impl From<super::components::DiscreteState> for WaveformPoint {
    #[inline]
    fn from(value: super::components::DiscreteState) -> Self {
        WaveformPoint::DiscreteState(value)
    }
}

impl From<super::components::DiscreteStateInit> for WaveformPoint {
    #[inline]
    fn from(value: super::components::DiscreteStateInit) -> Self {
        WaveformPoint::DiscreteStateInit(value)
    }
}

impl From<super::components::Event> for WaveformPoint {
    #[inline]
    fn from(value: super::components::Event) -> Self {
        WaveformPoint::Event(value)
    }
}

impl From<super::components::DiscreteStateNormal> for WaveformPoint {
    #[inline]
    fn from(value: super::components::DiscreteStateNormal) -> Self {
        WaveformPoint::DiscreteStateNormal(value)
    }
}

impl re_types::SizeBytes for WaveformPoint {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        match self {
            WaveformPoint::Scalar(scalar) => scalar.heap_size_bytes(),
            WaveformPoint::DiscreteState(discrete_state) => discrete_state.heap_size_bytes(),
            WaveformPoint::DiscreteStateInit(discrete_state_init) => {
                discrete_state_init.heap_size_bytes()
            }
            WaveformPoint::DiscreteStateNormal(discrete_state_normal) => {
                discrete_state_normal.heap_size_bytes()
            }
            WaveformPoint::Event(event) => event.heap_size_bytes(),
        }
    }

    #[inline]
    fn is_pod() -> bool {
        false
    }
}

static REQUIRED_COMPONENTS: once_cell::sync::Lazy<[ComponentName; 1usize]> =
    once_cell::sync::Lazy::new(|| ["wf.components.WaveformPointIndicator".into()]);

static RECOMMENDED_COMPONENTS: once_cell::sync::Lazy<[ComponentName; 0usize]> =
    once_cell::sync::Lazy::new(|| []);

static OPTIONAL_COMPONENTS: once_cell::sync::Lazy<[ComponentName; 5usize]> =
    once_cell::sync::Lazy::new(|| {
        [
            "wf.components.Scalar".into(),
            "wf.components.DiscreteState".into(),
            "wf.components.DiscreteStateInit".into(),
            "wf.components.DiscreteStateNormal".into(),
            "wf.components.Event".into(),
        ]
    });

static ALL_COMPONENTS: once_cell::sync::Lazy<[ComponentName; 6usize]> =
    once_cell::sync::Lazy::new(|| {
        [
            "wf.components.WaveformPointIndicator".into(),
            "wf.components.Scalar".into(),
            "wf.components.DiscreteState".into(),
            "wf.components.DiscreteStateInit".into(),
            "wf.components.DiscreteStateNormal".into(),
            "wf.components.Event".into(),
        ]
    });

impl WaveformPoint {
    /// The total number of components in the archetype: 1 required, 1 recommended, 0 optional
    pub const NUM_COMPONENTS: usize = 2usize;
}

/// Indicator component for the [`WaveformPoint`] [`re_types::Archetype`]
pub type WaveformPointIndicator = re_types::GenericIndicatorComponent<WaveformPoint>;

impl re_types::Archetype for WaveformPoint {
    type Indicator = WaveformPointIndicator;

    fn name() -> re_sdk::ArchetypeName {
        "wf.archetypes.WaveformPoint".into()
    }

    fn required_components() -> std::borrow::Cow<'static, [ComponentName]> {
        REQUIRED_COMPONENTS.as_slice().into()
    }

    fn recommended_components() -> std::borrow::Cow<'static, [ComponentName]> {
        RECOMMENDED_COMPONENTS.as_slice().into()
    }

    fn optional_components() -> std::borrow::Cow<'static, [ComponentName]> {
        OPTIONAL_COMPONENTS.as_slice().into()
    }

    fn all_components() -> std::borrow::Cow<'static, [ComponentName]> {
        ALL_COMPONENTS.as_slice().into()
    }

    fn indicator() -> re_sdk::MaybeOwnedComponentBatch<'static> {
        re_sdk::MaybeOwnedComponentBatch::Owned(
            Box::<<Self as re_sdk::Archetype>::Indicator>::default(),
        )
    }

    fn display_name() -> &'static str {
        "WaveformPoint"
    }
}

impl re_types::AsComponents for WaveformPoint {
    fn as_component_batches(&self) -> Vec<re_sdk::MaybeOwnedComponentBatch<'_>> {
        re_tracing::profile_function!();
        use re_types::Archetype as _;
        match self {
            WaveformPoint::Scalar(scalar) => vec![
                Some(Self::indicator()),
                Some((scalar as &dyn re_types::ComponentBatch).into()),
            ]
            .into_iter()
            .flatten()
            .collect(),
            WaveformPoint::DiscreteState(state) => vec![
                Some(Self::indicator()),
                Some((state as &dyn re_types::ComponentBatch).into()),
            ]
            .into_iter()
            .flatten()
            .collect(),
            WaveformPoint::DiscreteStateInit(state_init) => vec![
                Some(Self::indicator()),
                Some((state_init as &dyn re_types::ComponentBatch).into()),
            ]
            .into_iter()
            .flatten()
            .collect(),
            WaveformPoint::DiscreteStateNormal(state_normal) => vec![
                Some(Self::indicator()),
                Some((state_normal as &dyn re_types::ComponentBatch).into()),
            ]
            .into_iter()
            .flatten()
            .collect(),
            WaveformPoint::Event(event) => vec![
                Some(Self::indicator()),
                Some((event as &dyn re_types::ComponentBatch).into()),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }
}
