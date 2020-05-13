/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::XRViewBinding::{XREye, XRViewMethods};
use crate::dom::bindings::reflector::DomObject;
use crate::dom::bindings::reflector::{reflect_dom_object, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::utils::create_typed_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::xrrigidtransform::XRRigidTransform;
use crate::dom::xrsession::{cast_transform, ApiViewerPose, XRSession};
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use std::ptr::NonNull;
use webxr_api::{ApiSpace, View};

#[dom_struct]
pub struct XRView {
    reflector_: Reflector,
    session: Dom<XRSession>,
    eye: XREye,
    #[ignore_malloc_size_of = "mozjs"]
    proj: Heap<*mut JSObject>,
    #[ignore_malloc_size_of = "defined in rust-webxr"]
    view: View<ApiSpace>,
    transform: Dom<XRRigidTransform>,
}

impl XRView {
    fn new_inherited(
        session: &XRSession,
        transform: &XRRigidTransform,
        eye: XREye,
        view: View<ApiSpace>,
    ) -> XRView {
        XRView {
            reflector_: Reflector::new(),
            session: Dom::from_ref(session),
            eye,
            proj: Heap::default(),
            view,
            transform: Dom::from_ref(transform),
        }
    }

    pub fn view(&self) -> &View<ApiSpace> {
        &self.view
    }

    pub fn new<V: Copy>(
        global: &GlobalScope,
        session: &XRSession,
        view: &View<V>,
        eye: XREye,
        pose: &ApiViewerPose,
    ) -> DomRoot<XRView> {
        // XXXManishearth compute and cache projection matrices on the Display

        // this transform is the pose of the viewer in the eye space, i.e. it is the transform
        // from the viewer space to the eye space. We invert it to get the pose of the eye in the viewer space.
        let offset = view.transform.inverse();

        let transform = pose.pre_transform(&offset);
        let transform = XRRigidTransform::new(global, cast_transform(transform));

        reflect_dom_object(
            Box::new(XRView::new_inherited(
                session,
                &transform,
                eye,
                view.cast_unit(),
            )),
            global,
        )
    }

    pub fn session(&self) -> &XRSession {
        &self.session
    }
}

impl XRViewMethods for XRView {
    /// https://immersive-web.github.io/webxr/#dom-xrview-eye
    fn Eye(&self) -> XREye {
        self.eye
    }

    /// https://immersive-web.github.io/webxr/#dom-xrview-projectionmatrix
    fn ProjectionMatrix(&self, _cx: JSContext) -> NonNull<JSObject> {
        if self.proj.get().is_null() {
            let cx = self.global().get_cx();
            // row_major since euclid uses row vectors
            let proj = self.view.projection.to_row_major_array();
            create_typed_array(cx, &proj, &self.proj);
        }
        NonNull::new(self.proj.get()).unwrap()
    }

    /// https://immersive-web.github.io/webxr/#dom-xrview-transform
    fn Transform(&self) -> DomRoot<XRRigidTransform> {
        DomRoot::from_ref(&self.transform)
    }
}
