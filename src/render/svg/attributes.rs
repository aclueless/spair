use super::SvgElementUpdater;
use crate::{
    component::Component,
    render::base::{ElementUpdater, ElementUpdaterMut, F64AttributeValue, StringAttributeValue},
};

#[cfg(feature = "queue-render")]
use crate::queue_render::val::{QrVal, QrValMapWithState};

make_traits_for_attribute_values! {
    LengthPercentage {
        i32, set_i32_attribute qr_attribute qrmws_attribute,
        f64, set_f64_attribute qr_attribute qrmws_attribute,
        &str, set_str_attribute NO_QUEUE_RENDER NO_QUEUE_RENDER,
        String, set_string_attribute qr_attribute qrmws_attribute,
    }
}

pub trait SamsHandMade<C: Component>: Sized + ElementUpdaterMut<C> {
    fn class(mut self, class_name: &str) -> Self {
        self.element_updater_mut().class(class_name);
        self
    }

    fn class_if(mut self, class_on: bool, class_name: &str) -> Self {
        self.element_updater_mut().class_if(class_on, class_name);
        self
    }

    /// Set the `first_class` if `first` is true, otherwise, set the `second_class`
    fn class_or(mut self, first: bool, first_class: &str, second_class: &str) -> Self {
        self.element_updater_mut()
            .class_or(first, first_class, second_class);
        self
    }

    fn focus(mut self, value: bool) -> Self {
        self.element_updater_mut().focus(value);
        self
    }

    /// This method only accepts a &Route. If you want set `href` with a str, please use `href_str()`.
    /// It is possible to make this method accept both a Route and a str, but I intentionally make
    /// them two separate methods. The purpose is to remind users to use a Route when it's possible.
    fn href(mut self, route: &C::Routes) -> Self {
        self.element_updater_mut().href(route);
        self
    }

    fn id(mut self, id: &str) -> Self {
        self.element_updater_mut().id(id);
        self
    }

    fn scroll_to_top_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_updater()
                .element()
                .ws_element()
                .scroll_to_view_with_bool(true);
        }
        self
    }

    fn scroll_to_bottom_if(self, need_to_scroll: bool) -> Self {
        if need_to_scroll {
            self.element_updater()
                .element()
                .ws_element()
                .scroll_to_view_with_bool(false);
        }
        self
    }
}

#[cfg(test)]
use crate::render::svg::TestSvgMethods;
make_trait_for_attribute_methods! {
    TestStructs: (TestSvgMethods)
    TraitName: SamsForDistinctNames
    attributes:
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
        //str          class
        str          clip
        // ambiguous
        // str          clip_path "clip-path"
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
        LengthPercentage          cx
        LengthPercentage          cy
        str          d
        str          decelerate
        str          descent
        str          diffuse_constant "diffuseConstant"
        str          direction
        str          display
        str          divisor
        str          dominant_baseline "dominant-baseline"
        str          dur
        LengthPercentage          dx
        LengthPercentage          dy
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
        LengthPercentage          height
        str          horiz_adv_x "horiz-adv-x"
        str          horiz_origin_x "horiz-origin-x"
        str          href
        str          hreflang
        //str          id
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
        // ambiguous
        // str          mask
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
        // ambiguous
        // str          path
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
        LengthPercentage          width
        str          widths
        str          word_spacing "word-spacing"
        str          writing_mode "writing-mode"
        LengthPercentage          x
        LengthPercentage          x1
        LengthPercentage          x2
        str          x_height "x-height"
        str          x_channel_selector "xChannelSelector"
        // str          xml_base "xml:base"
        str          xml_lang "xml:lang"
        // str          xml_space "xml:space"
        // str          xmlns
        // str          xmlns_xlink "xmlns:xlink"
        LengthPercentage          y
        LengthPercentage          y1
        LengthPercentage          y2
        str          y_channel_selector "yChannelSelector"
        str          z
        str          zoom_and_pan "zoomAndPan"
}

pub struct SvgAttributesOnly<'er, C: Component>(ElementUpdater<'er, C>);
pub struct SvgStaticAttributesOnly<'er, C: Component>(ElementUpdater<'er, C>);
pub struct SvgStaticAttributes<'er, C: Component>(ElementUpdater<'er, C>);

impl<'er, C: Component> SvgAttributesOnly<'er, C> {
    pub(super) fn new(er: ElementUpdater<'er, C>) -> Self {
        Self(er)
    }
    pub(super) fn into_inner(self) -> ElementUpdater<'er, C> {
        self.0
    }

    pub fn static_attributes_only(self) -> SvgStaticAttributesOnly<'er, C> {
        SvgStaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> SvgStaticAttributesOnly<'er, C> {
    pub(super) fn new(mut er: ElementUpdater<'er, C>) -> Self {
        er.set_static_mode();
        Self(er)
    }
    pub(super) fn into_inner(self) -> ElementUpdater<'er, C> {
        self.0
    }
}

impl<'er, C: Component> SvgStaticAttributes<'er, C> {
    pub(super) fn new(mut er: ElementUpdater<'er, C>) -> Self {
        er.set_static_mode();
        Self(er)
    }
    pub(super) fn into_inner(self) -> ElementUpdater<'er, C> {
        self.0
    }
    pub fn static_attributes_only(self) -> SvgStaticAttributesOnly<'er, C> {
        SvgStaticAttributesOnly::new(self.0)
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for SvgAttributesOnly<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.0
    }
    fn element_updater_mut(&mut self) -> &'er mut ElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for SvgStaticAttributesOnly<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.0
    }
    fn element_updater_mut(&mut self) -> &'er mut ElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> ElementUpdaterMut<C> for SvgStaticAttributes<'er, C> {
    fn element_updater(&self) -> &ElementUpdater<C> {
        &self.0
    }
    fn element_updater_mut(&mut self) -> &'er mut ElementUpdater<C> {
        &mut self.0
    }
}

impl<'er, C: Component> SamsHandMade<C> for SvgElementUpdater<'er, C> {}
impl<'er, C: Component> SamsForDistinctNames<C> for SvgElementUpdater<'er, C> {}

impl<'er, C: Component> SamsForDistinctNames<C> for SvgStaticAttributes<'er, C> {}
impl<'er, C: Component> SamsHandMade<C> for SvgStaticAttributes<'er, C> {}

impl<'er, C: Component> SamsForDistinctNames<C> for SvgStaticAttributesOnly<'er, C> {}
impl<'er, C: Component> SamsHandMade<C> for SvgStaticAttributesOnly<'er, C> {}

impl<'er, C: Component> SamsForDistinctNames<C> for SvgAttributesOnly<'er, C> {}
impl<'er, C: Component> SamsHandMade<C> for SvgAttributesOnly<'er, C> {}
