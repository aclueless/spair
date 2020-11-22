use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub struct SvgStaticAttributes<'a, C>(super::SvgUpdater<'a, C>);

impl<'a, C: crate::component::Component> SvgStaticAttributes<'a, C> {
    pub(super) fn new(su: super::SvgUpdater<'a, C>) -> Self {
        Self(su)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    pub fn nodes(self) -> super::SvgNodesOwned<'a, C> {
        self.0.nodes()
    }

    pub fn static_nodes(self) -> super::SvgStaticNodesOwned<'a, C> {
        self.0.static_nodes()
    }
}

mod sealed {
    // TODO: Copied from dom::attributes. Should be a common trait?
    use wasm_bindgen::UnwrapThrowExt;

    pub trait AttributeSetter {
        fn ws_html_element(&self) -> &web_sys::HtmlElement;
        fn ws_element(&self) -> &web_sys::Element;
        fn require_set_listener(&mut self) -> bool;
        fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>);

        // Check if the attribute need to be set (and store the new value for the next check)
        fn check_bool_attribute(&mut self, value: bool) -> bool;
        fn check_str_attribute(&mut self, value: &str) -> bool;
        fn check_i32_attribute(&mut self, value: i32) -> bool;
        fn check_u32_attribute(&mut self, value: u32) -> bool;
        fn check_f64_attribute(&mut self, value: f64) -> bool;

        fn set_bool_attribute(&mut self, name: &str, value: bool) {
            if self.check_bool_attribute(value) {
                if value {
                    self.ws_element()
                        .set_attribute(name, "")
                        .expect_throw("Unable to set bool attribute");
                } else {
                    self.ws_element()
                        .remove_attribute(name)
                        .expect_throw("Unable to remove bool attribute");
                }
            }
        }

        fn set_str_attribute(&mut self, name: &str, value: &str) {
            if self.check_str_attribute(value) {
                self.ws_element()
                    .set_attribute(name, value)
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_i32_attribute(&mut self, name: &str, value: i32) {
            if self.check_i32_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_u32_attribute(&mut self, name: &str, value: u32) {
            if self.check_u32_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }

        fn set_f64_attribute(&mut self, name: &str, value: f64) {
            if self.check_f64_attribute(value) {
                self.ws_element()
                    .set_attribute(name, &value.to_string())
                    .expect_throw("Unable to set string attribute");
            }
        }
    }
}


pub trait SvgAttributeSetter<C>: Sized + sealed::AttributeSetter {
    // TODO: Some items in this trait are copied from dom::attributes. Should be a common trait?
    create_methods_for_events! {
        on_focus Focus,
        on_blur Blur,

        on_aux_click AuxClick,
        on_click Click,
        on_double_click DoubleClick,
        on_mouse_enter MouseEnter,
        on_mouse_over MouseOver,
        on_mouse_move MouseMove,
        on_mouse_down MouseDown,
        on_mouse_up MouseUp,
        on_mouse_leave MouseLeave,
        on_mouse_out MouseOut,
        on_context_menu ContextMenu,

        on_wheel Wheel,
        on_select UiSelect,

        on_input Input,

        on_key_down KeyDown,
        on_key_press KeyPress,
        on_key_up KeyUp,

        on_change Change,
        on_reset Reset,
        on_submit Submit,
        on_pointer_lock_change PointerLockChange,
        on_pointer_lock_error PointerLockError,

        on_ended Ended,
    }

    fn bool_attr(mut self, name: &str, value: bool) -> Self {
        self.set_bool_attribute(name, value);
        self
    }

    fn str_attr(mut self, name: &str, value: &str) -> Self {
        self.set_str_attribute(name, value);
        self
    }

    fn i32_attr(mut self, name: &str, value: i32) -> Self {
        self.set_i32_attribute(name, value);
        self
    }

    fn u32_attr(mut self, name: &str, value: u32) -> Self {
        self.set_u32_attribute(name, value);
        self
    }

    fn f64_attr(mut self, name: &str, value: f64) -> Self {
        self.set_f64_attribute(name, value);
        self
    }

    create_methods_for_attributes! {
        str          accent_height "accent-height"
        str          accumulate
        str          additive
        str          alignment_baseline "alignment-baseline"
        str          allow_reorder "allowReorder"
        str          alphabetic
        str          amplitude
        str          arabic_form "arabic-form"
        str          ascent
        str          attribute_name "attributeName"
        str          attribute_type "attributeType"
        str          auto_reverse "autoReverse"
        str          azimuth
        str          base_frequency "baseFrequency"
        str          base_profile "baseProfile"
        str          baseline_shift "baseline-shift"
        str          bbox
        str          begin
        str          bias
        str          by
        str          calc_mode "calcMode"
        str          cap_height "cap-height"
        str          class
        str          clip
        str          clip_path "clip-path"
        str          clip_path_units "clipPathUnits"
        str          clip_rule "clip-rule"
        str          color
        str          color_interpolation "color-interpolation"
        str          color_interpolation_filters "color-interpolation-filters"
        str          color_profile "color-profile"
        str          color_rendering "color-rendering"
        str          content_script_type "contentScriptType"
        str          content_style_type "contentStyleType"
        str          cursor
        f64          cx
        f64          cy
        str          d
        str          decelerate
        str          descent
        str          diffuse_constant "diffuseConstant"
        str          direction
        str          display
        str          divisor
        str          dominant_baseline "dominant-baseline"
        str          dur
        f64          dx
        f64          dy
        str          edge_mode "edgeMode"
        str          elevation
        str          enable_background "enable-background"
        str          end
        str          exponent
        str          external_resources_required "externalResourcesRequired"
        str          fill
        str          fill_opacity "fill-opacity"
        str          fill_rule "fill-rule"
        str          filter_attr "filter"   // rename `filter` to `filter_attr` because conflicting with <filter> element
        str          filter_res "filterRes"
        str          filter_units "filterUnits"
        str          flood_color "flood-color"
        str          flood_opacity "flood-opacity"
        str          font_family "font-family"
        str          font_size "font-size"
        str          font_size_adjust "font-size-adjust"
        str          font_stretch "font-stretch"
        str          font_style "font-style"
        str          font_variant "font-variant"
        str          font_weight "font-weight"
        str          format
        str          fr
        str          from
        str          fx
        str          fy
        str          g1
        str          g2
        str          glyph_name "glyph-name"
        str          glyph_orientation_horizontal "glyph-orientation-horizontal"
        str          glyph_orientation_vertical "glyph-orientation-vertical"
        str          glyph_ref "glyphRef"
        str          gradient_transform "gradientTransform"
        str          gradient_units "gradientUnits"
        str          hanging
        f64          height
        str          horiz_adv_x "horiz-adv-x"
        str          horiz_origin_x "horiz-origin-x"
        str          href
        str          hreflang
        str          id
        str          ideographic
        str          image_rendering "image-rendering"
        str          r#in "in"
        str          in2
        str          intercept
        str          k
        str          k1
        str          k2
        str          k3
        str          k4
        str          kernel_matrix "kernelMatrix"
        str          kernel_unit_length "kernelUnitLength"
        str          kerning
        str          key_points "keyPoints"
        str          key_splines "keySplines"
        str          key_times "keyTimes"
        str          lang
        str          length_adjust "lengthAdjust"
        str          letter_spacing "letter-spacing"
        str          lighting_color "lighting-color"
        str          limiting_cone_angle "limitingConeAngle"
        str          local
        str          marker_end "marker-end"
        str          marker_mid "marker-mid"
        str          marker_start "marker-start"
        str          marker_height "markerHeight"
        str          marker_units "markerUnits"
        str          marker_width "markerWidth"
        str          mask
        str          mask_content_units "maskContentUnits"
        str          mask_units "maskUnits"
        str          mathematical
        str          max
        str          media
        str          method
        str          min
        str          mode
        str          name
        str          num_octaves "numOctaves"
        str          offset
        str          opacity
        str          operator
        str          order
        str          orient
        str          orientation
        str          origin
        str          overflow
        str          overline_position "overline-position"
        str          overline_thickness "overline-thickness"
        str          paint_order "paint-order"
        str          panose_1 "panose-1"
        str          path
        str          path_length "pathLength"
        str          pattern_content_units "patternContentUnits"
        str          pattern_transform "patternTransform"
        str          pattern_units "patternUnits"
        str          ping
        str          pointer_events "pointer-events"
        str          points
        str          points_at_x "pointsAtX"
        str          points_at_y "pointsAtY"
        str          points_at_z "pointsAtZ"
        str          preserve_alpha "preserveAlpha"
        str          preserve_aspect_ratio "preserveAspectRatio"
        str          primitive_units "primitiveUnits"
        f64          r
        str          radius
        str          ref_x "refX"
        str          ref_y "refY"
        str          referrer_policy "referrerPolicy"
        str          rel
        str          rendering_intent "rendering-intent"
        str          repeat_count "repeatCount"
        str          repeat_dur "repeatDur"
        str          required_extensions "requiredExtensions"
        str          required_features "requiredFeatures"
        str          restart
        str          result
        str          rotate
        str          rx
        str          ry
        str          scale
        str          seed
        str          shape_rendering "shape-rendering"
        str          slope
        str          spacing
        str          specular_constant "specularConstant"
        str          specular_exponent "specularExponent"
        str          speed
        str          spread_method "spreadMethod"
        str          start_offset "startOffset"
        f64          std_deviation "stdDeviation"
        str          stemh
        str          stemv
        str          stitch_tiles "stitchTiles"
        str          stop_color "stop-color"
        str          stop_opacity "stop-opacity"
        str          strikethrough_position "strikethrough-position"
        str          strikethrough_thickness "strikethrough-thickness"
        str          string
        str          stroke
        str          stroke_dasharray "stroke-dasharray"
        str          stroke_dashoffset "stroke-dashoffset"
        str          stroke_linecap "stroke-linecap"
        str          stroke_linejoin "stroke-linejoin"
        str          stroke_miterlimit "stroke-miterlimit"
        str          stroke_opacity "stroke-opacity"
        f64          stroke_width "stroke-width"
        str          style
        str          surface_scale "surfaceScale"
        str          system_language "systemLanguage"
        str          tabindex
        str          table_values "tableValues"
        str          target
        str          target_x "targetX"
        str          target_y "targetY"
        str          text_anchor "text-anchor"
        str          text_decoration "text-decoration"
        str          text_rendering "text-rendering"
        str          text_length "textLength"
        str          to
        str          transform
        str          transform_origin "transform-origin"
        str          r#type "type"
        str          u1
        str          u2
        str          underline_position "underline-position"
        str          underline_thickness "underline-thickness"
        str          unicode
        str          unicode_bidi "unicode-bidi"
        str          unicode_range "unicode-range"
        str          units_per_em "units-per-em"
        str          v_alphabetic "v-alphabetic"
        str          v_hanging "v-hanging"
        str          v_ideographic "v-ideographic"
        str          v_mathematical "v-mathematical"
        str          values
        str          vector_effect "vector-effect"
        str          version
        str          vert_adv_y "vert-adv-y"
        str          vert_origin_x "vert-origin-x"
        str          vert_origin_y "vert-origin-y"
        str          view_box "viewBox"
        str          view_target "viewTarget"
        str          visibility
        f64          width
        str          widths
        str          word_spacing "word-spacing"
        str          writing_mode "writing-mode"
        f64          x
        f64          x1
        f64          x2
        str          x_height "x-height"
        str          x_channel_selector "xChannelSelector"
        // str          xml_base "xml:base"
        str          xml_lang "xml:lang"
        // str          xml_space "xml:space"
        // str          xmlns
        // str          xmlns_xlink "xmlns:xlink"
        f64          y
        f64          y1
        f64          y2
        str          y_channel_selector "yChannelSelector"
        str          z
        str          zoom_and_pan "zoomAndPan"
    }

    // Copied from crate::dom::attributes
    // TODO: Simimilar code should be reuse
    fn class_if(mut self, class_name: &str, class_on: bool) -> Self {
        if self.check_bool_attribute(class_on) {
            if class_on {
                self.ws_element()
                    .class_list()
                    .add_1(class_name)
                    .expect_throw("Unable to add class");
            } else {
                self.ws_element()
                    .class_list()
                    .remove_1(class_name)
                    .expect_throw("Unable to remove class");
            }
        }
        self
    }
}

impl<'a, C: crate::component::Component> SvgAttributeSetter<C> for super::SvgUpdater<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> sealed::AttributeSetter for super::SvgUpdater<'a, C> {
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.element.ws_element.unchecked_ref()
    }

    fn ws_element(&self) -> &web_sys::Element {
        &self.element.ws_element
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.element.attributes.store_listener(self.index, listener);
        self.index += 1;
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        let rs = self
            .element
            .attributes
            .check_bool_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        let rs = self
            .element
            .attributes
            .check_str_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        let rs = self
            .element
            .attributes
            .check_i32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        let rs = self
            .element
            .attributes
            .check_u32_attribute(self.index, value);
        self.index += 1;
        rs
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        let rs = self
            .element
            .attributes
            .check_f64_attribute(self.index, value);
        self.index += 1;
        rs
    }
}

impl<'a, C: crate::component::Component> SvgAttributeSetter<C> for SvgStaticAttributes<'a, C> where
    C: crate::component::Component
{
}

impl<'a, C: crate::component::Component> sealed::AttributeSetter
    for super::SvgStaticAttributes<'a, C>
{
    fn ws_html_element(&self) -> &web_sys::HtmlElement {
        self.0.ws_html_element()
    }

    fn ws_element(&self) -> &web_sys::Element {
        sealed::AttributeSetter::ws_element(&self.0)
        //self.0.ws_element()
    }

    fn require_set_listener(&mut self) -> bool {
        true
    }

    fn store_listener(&mut self, listener: Box<dyn crate::events::Listener>) {
        self.0.store_listener(listener);
    }

    fn check_bool_attribute(&mut self, value: bool) -> bool {
        self.0.check_bool_attribute(value)
    }

    fn check_str_attribute(&mut self, value: &str) -> bool {
        self.0.check_str_attribute(value)
    }

    fn check_i32_attribute(&mut self, value: i32) -> bool {
        self.0.check_i32_attribute(value)
    }

    fn check_u32_attribute(&mut self, value: u32) -> bool {
        self.0.check_u32_attribute(value)
    }

    fn check_f64_attribute(&mut self, value: f64) -> bool {
        self.0.check_f64_attribute(value)
    }
}
