use wasm_bindgen::{JsCast, UnwrapThrowExt};

pub trait SvgAttributeSetter<C>: Sized + crate::dom::attributes::AttributeSetter {
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

    // Copied from crate::dom::html::attributes
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

pub struct SvgStaticAttributes<'a, C>(crate::dom::ElementUpdater<'a, C>);

impl<'a, C> From<crate::dom::ElementUpdater<'a, C>> for SvgStaticAttributes<'a, C> {
    fn from(eu: crate::dom::ElementUpdater<'a, C>) -> Self {
        Self(eu)
    }
}

impl<'a, C: crate::component::Component> SvgStaticAttributes<'a, C> {
    pub fn nodes(self) -> crate::dom::SvgNodesOwned<'a, C> {
        self.0.svg_nodes()
    }

    pub fn static_nodes(self) -> crate::dom::SvgStaticNodesOwned<'a, C> {
        self.0.svg_static_nodes()
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}

    pub fn render(self, value: impl super::SvgRender<C>) -> crate::dom::SvgNodesOwned<'a, C> {
        self.0.svg_render(value)
    }

    // pub fn render_ref(
    //     self,
    //     value: &impl crate::renderable::RenderRef<C>,
    // ) -> crate::dom::NodesOwned<'a, C> {
    //     self.0.render_ref(value)
    // }

    pub fn r#static(
        self,
        value: impl super::SvgStaticRender<C>,
    ) -> crate::dom::SvgNodesOwned<'a, C> {
        self.0.svg_static(value)
    }

    pub fn list<I>(self, items: impl IntoIterator<Item = I>, mode: crate::dom::ListElementCreation)
    where
        I: super::ListItem<C>,
    {
        self.0
            .list_with_render(items, mode, I::ROOT_ELEMENT_TAG, I::render);
    }

    pub fn list_with_render<I, R>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
        tag: &str,
        render: R,
    ) where
        for<'i, 'c> R: Fn(&'i I, crate::Element<'c, C>),
    {
        self.0.list_with_render(items, mode, tag, render)
    }

    #[cfg(feature = "keyed-list")]
    pub fn keyed_list<I>(
        self,
        items: impl IntoIterator<Item = I>,
        mode: crate::dom::ListElementCreation,
    ) where
        for<'k> I: crate::dom::KeyedListItem<'k, C>,
    {
        self.0.keyed_list(items, mode)
    }

    pub fn component<CC: crate::component::Component>(
        self,
        child: &crate::component::ChildComp<CC>,
    ) {
        self.0.component(child);
    }
}
