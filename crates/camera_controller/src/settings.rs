//! Camera settings tree

use gphoto2::{
    widget::{Widget, WidgetType, WidgetValue},
    Error,
};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub struct CameraSettings {
    pub id: u32,
    pub name: String,
    pub label: Option<String>,
    pub widget_type: WidgetType,
    pub value: Option<WidgetValue>,
    pub readonly: bool,

    /// Children by name
    pub children: BTreeMap<u32, CameraSettings>,
}

impl TryFrom<Widget<'_>> for CameraSettings {
    type Error = Error;

    fn try_from(widget: Widget<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: widget.id()? as u32,
            name: widget.name()?.into(),
            label: widget.label().ok().map(Into::into),
            widget_type: widget.widget_type()?,
            value: widget.value().ok().map(|(value, _)| value).flatten(),
            readonly: widget.readonly()?,
            children: {
                let mut children = BTreeMap::new();

                for child in widget.children_iter()? {
                    children.insert(child.id()? as u32, child.try_into()?);
                }

                children
            },
        })
    }
}

impl CameraSettings {
    pub fn is_root(&self) -> bool {
        matches!(self.widget_type, WidgetType::Window)
    }

    pub fn is_section(&self) -> bool {
        matches!(self.widget_type, WidgetType::Section)
    }
}
