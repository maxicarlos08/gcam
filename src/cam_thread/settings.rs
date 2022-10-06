//! Camera settings tree

use gcam_lib::error::{AppError, AppResult};
use gphoto2::widget::Widget;
use std::{collections::BTreeMap, ops::RangeInclusive};

#[derive(Debug, Clone, PartialEq)]
pub enum StaticWidget {
    Group { children: BTreeMap<i32, CameraSettings>, id_by_names: BTreeMap<String, i32> },
    Text(String),
    Range { value: f32, range: RangeInclusive<f32>, step: f32 },
    Toggle { undefined: bool, value: bool },
    Radio { choices: Vec<String>, choice: String },
    Date { timestamp: i32 },
    Button,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CameraSettings {
    pub id: i32,
    pub name: String,
    pub label: String,
    pub widget: StaticWidget,
    pub readonly: bool,
}

impl TryFrom<Widget> for CameraSettings {
    type Error = AppError;

    fn try_from(widget: Widget) -> Result<Self, Self::Error> {
        Ok(CameraSettings {
            id: widget.id(),
            name: widget.name(),
            label: widget.label(),
            readonly: widget.readonly(),
            widget: match widget {
                Widget::Group(group_widget) => {
                    let children: BTreeMap<i32, CameraSettings> = group_widget
                        .children_iter()
                        .map(|child| child.try_into::<CameraSettings>().map(|res| (res.id, res)))
                        .collect::<AppResult<_>>()?;

                    StaticWidget::Group {
                        id_by_names: children
                            .iter()
                            .map(|(&id, child)| (child.name.clone(), id))
                            .collect(),
                        children,
                    }
                }
                Widget::Text(text_widget) => StaticWidget::Text(text_widget.value()),
                Widget::Range(range_widget) => {
                    let (range, step) = range_widget.range_and_step();

                    StaticWidget::Range { value: range_widget.value(), range, step }
                }
                Widget::Toggle(toggle_widget) => match toggle_widget.toggled() {
                    None => StaticWidget::Toggle { undefined: true, value: false },
                    Some(v) => StaticWidget::Toggle { undefined: false, value: v },
                },
                Widget::Radio(radio_widget) => {
                    let choices = radio_widget.choices_iter().collect();

                    StaticWidget::Radio { choices, choice: radio_widget.choice() }
                }
                Widget::Date(date_widget) => {
                    StaticWidget::Date { timestamp: date_widget.timestamp() }
                }
                Widget::Button(_) => StaticWidget::Button,
            },
        })
    }
}

impl CameraSettings {
    pub fn get_child(&self, name: &str) -> Option<&'_ CameraSettings> {
        if let StaticWidget::Group { children, id_by_names } = &self.widget {
            id_by_names.get(name).and_then(|id| children.get(id))
        } else {
            None
        }
    }
}
