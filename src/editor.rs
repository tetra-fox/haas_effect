use nih_plug::prelude::Editor;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::{assets, widgets::*};
use nih_plug_vizia::{create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::Arc;

mod param_knob;

use self::param_knob::ParamKnob;
use crate::{Channel, HaasHimselfPluginParams};

#[derive(Lens)]
struct Data {
    params: Arc<HaasHimselfPluginParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(size_fn)
}

fn size_fn() -> (u32, u32) {
    (300, 200)
}

pub(crate) fn create(
    params: Arc<HaasHimselfPluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        cx.add_theme(include_str!("editor/theme.css"));

        Data {
            params: params.clone(),
        }
        .build(cx);

        ResizeHandle::new(cx);

        VStack::new(cx, |cx| {
            // Label::new(cx, "swag plugin");
            HStack::new(cx, |cx| {
                ParamKnob::new(cx, Data::params, |p| &p.delay, false);
                ParamKnob::new(cx, Data::params, |p| &p.channel, true);
                // HStack::new(cx, |cx| {
                //     for channel in enum_iterator::all::<Channel>() {
                //     }
                // });
            });
        })
        .class("main")
        .child_space(Stretch(1.));
    })
}
