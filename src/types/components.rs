use itertools::Itertools;
use re_types::{components::ClassId, external::arrow2};

#[derive(Clone, Debug, PartialEq)]
pub struct Scalar(pub re_types::components::Scalar);
impl re_types::SizeBytes for Scalar {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <re_types::components::Scalar>::is_pod()
    }
}

impl<T: Into<re_types::components::Scalar>> From<T> for Scalar {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

re_types::macros::impl_into_cow!(Scalar);

impl re_types::Loggable for Scalar {
    type Name = re_types::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "wf.components.Scalar".into()
    }

    #[allow(clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> re_types::external::arrow2::datatypes::DataType {
        re_types::components::Scalar::arrow_datatype()
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
    ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
    where
        Self: 'a,
    {
        let data = data.into_iter().map(|d| d.map(|d| d.into().0));
        re_types::components::Scalar::to_arrow_opt(data)
    }

    fn from_arrow_opt(
        data: &dyn arrow2::array::Array,
    ) -> re_types::DeserializationResult<Vec<Option<Self>>> {
        re_types::components::Scalar::from_arrow_opt(data)
            .map(|v| v.into_iter().map(|v| v.map(|v| Scalar(v))).collect_vec())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteState(pub ClassId);

impl re_types::SizeBytes for DiscreteState {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <String>::is_pod()
    }
}

impl<T: Into<ClassId>> From<T> for DiscreteState {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

re_types::macros::impl_into_cow!(DiscreteState);

impl re_types::Loggable for DiscreteState {
    type Name = re_types::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "wf.components.DiscreteState".into()
    }

    #[allow(clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> re_types::external::arrow2::datatypes::DataType {
        ClassId::arrow_datatype()
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
    ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
    where
        Self: 'a,
    {
        let data = data.into_iter().map(|d| d.map(|d| d.into().0));
        ClassId::to_arrow_opt(data)
    }

    fn from_arrow_opt(
        data: &dyn arrow2::array::Array,
    ) -> re_types::DeserializationResult<Vec<Option<Self>>> {
        ClassId::from_arrow_opt(data).map(|v| {
            v.into_iter()
                .map(|v| v.map(|v| DiscreteState(v)))
                .collect_vec()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteStateInit(pub ClassId);

impl re_types::SizeBytes for DiscreteStateInit {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <String>::is_pod()
    }
}

impl<T: Into<ClassId>> From<T> for DiscreteStateInit {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

re_types::macros::impl_into_cow!(DiscreteStateInit);

impl re_types::Loggable for DiscreteStateInit {
    type Name = re_types::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "wf.components.DiscreteStateInit".into()
    }

    #[allow(clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> re_types::external::arrow2::datatypes::DataType {
        ClassId::arrow_datatype()
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
    ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
    where
        Self: 'a,
    {
        let data = data.into_iter().map(|d| d.map(|d| d.into().0));
        ClassId::to_arrow_opt(data)
    }

    fn from_arrow_opt(
        data: &dyn arrow2::array::Array,
    ) -> re_types::DeserializationResult<Vec<Option<Self>>> {
        ClassId::from_arrow_opt(data).map(|v| {
            v.into_iter()
                .map(|v| v.map(|v| DiscreteStateInit(v)))
                .collect_vec()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteStateNormal(pub ClassId);

impl re_types::SizeBytes for DiscreteStateNormal {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <String>::is_pod()
    }
}

impl<T: Into<ClassId>> From<T> for DiscreteStateNormal {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

re_types::macros::impl_into_cow!(DiscreteStateNormal);

impl re_types::Loggable for DiscreteStateNormal {
    type Name = re_types::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "wf.components.DiscreteStateNormal".into()
    }

    #[allow(clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> re_types::external::arrow2::datatypes::DataType {
        ClassId::arrow_datatype()
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
    ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
    where
        Self: 'a,
    {
        let data = data.into_iter().map(|d| d.map(|d| d.into().0));
        ClassId::to_arrow_opt(data)
    }

    fn from_arrow_opt(
        data: &dyn arrow2::array::Array,
    ) -> re_types::DeserializationResult<Vec<Option<Self>>> {
        ClassId::from_arrow_opt(data).map(|v| {
            v.into_iter()
                .map(|v| v.map(|v| DiscreteStateNormal(v)))
                .collect_vec()
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event(pub ClassId);

impl re_types::SizeBytes for Event {
    #[inline]
    fn heap_size_bytes(&self) -> u64 {
        self.0.heap_size_bytes()
    }

    #[inline]
    fn is_pod() -> bool {
        <String>::is_pod()
    }
}

impl<T: Into<ClassId>> From<T> for Event {
    fn from(v: T) -> Self {
        Self(v.into())
    }
}

re_types::macros::impl_into_cow!(Event);

impl re_types::Loggable for Event {
    type Name = re_types::ComponentName;

    #[inline]
    fn name() -> Self::Name {
        "wf.components.Event".into()
    }

    #[allow(clippy::wildcard_imports)]
    #[inline]
    fn arrow_datatype() -> re_types::external::arrow2::datatypes::DataType {
        ClassId::arrow_datatype()
    }

    fn to_arrow_opt<'a>(
        data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
    ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
    where
        Self: 'a,
    {
        let data = data.into_iter().map(|d| d.map(|d| d.into().0));
        ClassId::to_arrow_opt(data)
    }

    fn from_arrow_opt(
        data: &dyn arrow2::array::Array,
    ) -> re_types::DeserializationResult<Vec<Option<Self>>> {
        ClassId::from_arrow_opt(data)
            .map(|v| v.into_iter().map(|v| v.map(|v| Event(v))).collect_vec())
    }
}
