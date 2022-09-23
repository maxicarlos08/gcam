//! Camera settings tree

use gphoto2::{
    widget::{Widget, WidgetType, WidgetValue},
    Error,
};
use std::collections::{BTreeMap, HashMap};

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
    pub children_id_names: HashMap<String, u32>,
}

impl TryFrom<Widget> for CameraSettings {
    type Error = Error;

    fn try_from(widget: Widget) -> Result<Self, Self::Error> {
        let children = {
            let mut children: BTreeMap<u32, CameraSettings> = BTreeMap::new();

            for child in widget.children_iter()? {
                children.insert(child.id()? as u32, child.try_into()?);
            }

            children
        };

        Ok(Self {
            id: widget.id()? as u32,
            name: widget.name()?.into(),
            label: widget.label().ok().map(Into::into),
            widget_type: widget.widget_type()?,
            value: widget.value().ok().map(|(value, _)| value).flatten(),
            readonly: widget.readonly()?,
            children_id_names: children
                .iter()
                .map(|(_, child)| (child.name.clone(), child.id))
                .collect(),
            children,
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
