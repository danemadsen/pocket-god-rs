//! `BCGradient`: a vertical colour ramp (`docs/re/rendering.md` §7).

use bevy_ecs::prelude::*;

use crate::display::{self, model};
use crate::gl::GlContext;
use crate::library::{GradientColor, GradientDef, Library};
use crate::object::{Class, ObjectWorldExt};

/// `BCGradient` ivars. The original points at the library's shared def and mutates it in
/// place (`setGradient:`, `fadeGradient:percent:`); the port keeps the gradient's own copy
/// and writes changes back to the library def as the original does (see rendering.md §7.3).
///
/// `def_id` is `None` when the def did not exist at creation (a NULL `mpGradientDef`: no
/// geometry). While it is set, the library's entry is the shared def that reads use.
#[derive(Component, Clone, Debug, Default)]
pub struct Gradient {
    pub def_id: Option<String>,
    pub def: GradientDef,
    pub model_width: f32,
    pub model_height: f32,
    pub vertices: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
}

pub const BC_GRADIENT: Class = Class {
    name: "BCGradient",
    superclass: Some(&model::BC_MODEL),
    draw: draw_gradient,
    draw_in_multi_model: draw_gradient_in_multi_model,
    ..model::BC_MODEL
};

/// `+[BCGradient createGradientDef:]` @ 0x100178500: parses `"pos:0xAARRGGBB,…"` stops in
/// string order (`sscanf("%f:%x")` per comma-separated component; unparsed fields stay 0).
pub fn create_gradient_def(spec: &str) -> GradientDef {
    let mut colors = Vec::new();
    for component in spec.split(',') {
        let (pos, packed) = scan_stop(component.as_bytes());
        let red = ((packed >> 16) & 0xff) as f32 / 255.0;
        let green = ((packed >> 8) & 0xff) as f32 / 255.0;
        let blue = (packed & 0xff) as f32 / 255.0;
        let alpha = (packed >> 24) as f32 / 255.0;
        let color = GradientColor {
            pos,
            r: red,
            g: green,
            b: blue,
            a: alpha,
        };
        colors.push(color);
    }
    GradientDef { colors }
}

/// `sscanf(component, "%f:%x", &pos, &packed)` with both outputs pre-set to 0.
fn scan_stop(text: &[u8]) -> (f32, u32) {
    let Some((pos, after_pos)) = scan_float(text) else {
        return (0.0, 0);
    };
    if text.get(after_pos) != Some(&b':') {
        return (pos, 0);
    }
    let packed = scan_hex(&text[after_pos + 1..]).unwrap_or(0);
    (pos, packed)
}

/// Skips C `isspace` characters (`' '`, `\t`, `\n`, `\v`, `\f`, `\r`).
fn skip_white_space(text: &[u8], mut index: usize) -> usize {
    while index < text.len() && matches!(text[index], b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r') {
        index += 1;
    }
    index
}

fn skip_digits(text: &[u8], mut index: usize, radix: u32) -> usize {
    while index < text.len() && (text[index] as char).is_digit(radix) {
        index += 1;
    }
    index
}

/// scanf `%f` (decimal form): the value and the index after the matched text.
fn scan_float(text: &[u8]) -> Option<(f32, usize)> {
    let start = skip_white_space(text, 0);
    let mut index = start;
    if index < text.len() && (text[index] == b'+' || text[index] == b'-') {
        index += 1;
    }
    let integer_end = skip_digits(text, index, 10);
    let mut digit_count = integer_end - index;
    index = integer_end;
    if index < text.len() && text[index] == b'.' {
        let fraction_end = skip_digits(text, index + 1, 10);
        digit_count += fraction_end - (index + 1);
        index = fraction_end;
    }
    if digit_count == 0 {
        return None;
    }
    if index < text.len() && (text[index] == b'e' || text[index] == b'E') {
        let mut exponent = index + 1;
        if exponent < text.len() && (text[exponent] == b'+' || text[exponent] == b'-') {
            exponent += 1;
        }
        let exponent_end = skip_digits(text, exponent, 10);
        if exponent_end > exponent {
            index = exponent_end;
        }
    }
    let literal = std::str::from_utf8(&text[start..index]).ok()?;
    let value = literal.parse::<f32>().ok()?;
    Some((value, index))
}

/// scanf `%x` into an `unsigned int` (optional sign and `0x` prefix; `strtoul` semantics:
/// an out-of-range value is `ULONG_MAX` whatever the sign, then truncated).
fn scan_hex(text: &[u8]) -> Option<u32> {
    let mut index = skip_white_space(text, 0);
    let mut negative = false;
    if index < text.len() && (text[index] == b'+' || text[index] == b'-') {
        negative = text[index] == b'-';
        index += 1;
    }
    let has_prefix = index + 1 < text.len() && text[index] == b'0' && (text[index + 1] == b'x' || text[index + 1] == b'X');
    if has_prefix && index + 2 < text.len() && text[index + 2].is_ascii_hexdigit() {
        index += 2;
    }
    let digits_end = skip_digits(text, index, 16);
    if digits_end == index {
        return None;
    }
    let mut value: u64 = 0;
    let mut overflow = false;
    for digit in &text[index..digits_end] {
        let nibble = (*digit as char).to_digit(16).unwrap_or(0);
        let shifted = value.checked_mul(16);
        let next = shifted.and_then(|shifted| shifted.checked_add(u64::from(nibble)));
        match next {
            Some(next) => value = next,
            None => overflow = true,
        }
    }
    if overflow {
        return Some(u32::MAX);
    }
    let magnitude = value as u32;
    if negative {
        Some(magnitude.wrapping_neg())
    } else {
        Some(magnitude)
    }
}

/// `-[BCGradient initWithId:width:height:gradientDef:]` @ 0x100178714.
pub fn spawn(world: &mut World, id: Option<&str>, width: f32, height: f32, def_id: Option<&str>) -> Entity {
    let shared = match def_id {
        Some(def_id) => world.resource::<Library>().gradient_def(def_id).cloned(),
        None => None,
    };
    let (def_id, def) = match shared {
        Some(def) => (def_id.map(str::to_owned), def),
        None => (None, GradientDef::default()),
    };
    let mut gradient = Gradient {
        def_id,
        def,
        model_width: width,
        model_height: height,
        vertices: Vec::new(),
        colors: Vec::new(),
    };
    create_model(&mut gradient);

    let entity = display::spawn(world, &BC_GRADIENT, id);
    world.entity_mut(entity).insert((model::Model::default(), gradient));
    entity
}

/// `-createModel` @ 0x1001788b4.
pub fn create_model(gradient: &mut Gradient) {
    let count = gradient.def.colors.len();
    gradient.vertices = vec![[0.0; 3]; count * 2];
    gradient.colors = vec![[0.0; 4]; count * 2];
    let stops = gradient.def.colors.clone();
    for (index, stop) in stops.iter().enumerate() {
        write_stop(gradient, index, stop);
    }
}

/// Vertices `2i` (x = 0) and `2i + 1` (x = width) at `y = height · pos`, both coloured with
/// the stop's straight colour.
fn write_stop(gradient: &mut Gradient, index: usize, stop: &GradientColor) {
    let y = gradient.model_height * stop.pos;
    let color = [stop.r, stop.g, stop.b, stop.a];
    let left = index * 2;
    let right = left + 1;
    if right >= gradient.vertices.len() {
        return;
    }
    gradient.colors[left] = color;
    gradient.vertices[left] = [0.0, y, 0.0];
    gradient.colors[right] = color;
    gradient.vertices[right] = [gradient.model_width, y, 0.0];
}

/// The def this gradient points at: the library's shared entry, else its own copy.
fn shared_def(world: &World, gradient: &Gradient) -> GradientDef {
    let library = world.resource::<Library>();
    let shared = gradient.def_id.as_deref().and_then(|id| library.gradient_def(id));
    match shared {
        Some(def) => def.clone(),
        None => gradient.def.clone(),
    }
}

/// Stores the mutated def in the gradient and back into the library's shared entry.
fn store_shared_def(world: &mut World, entity: Entity, def: GradientDef) {
    let def_id = world.obj::<Gradient>(entity).def_id.clone();
    if let Some(def_id) = def_id {
        let mut library = world.resource_mut::<Library>();
        if let Some(shared) = library.gradient_defs.get_mut(&def_id) {
            *shared = def.clone();
        }
    }
    world.obj_mut::<Gradient>(entity).def = def;
}

/// `-setGradient:` @ 0x100178be4: copies `def_id`'s stops over the gradient's own def (the
/// shared library def) and rebuilds the geometry. An unknown source def does nothing.
pub fn set_gradient(world: &mut World, entity: Entity, def_id: &str) {
    let Some(source) = world.resource::<Library>().gradient_def(def_id).cloned() else {
        return;
    };
    let mut own = shared_def(world, world.obj::<Gradient>(entity));
    {
        let mut gradient = world.obj_mut::<Gradient>(entity);
        for index in 0..own.colors.len() {
            let Some(stop) = source.colors.get(index) else {
                break;
            };
            own.colors[index] = *stop;
            write_stop(&mut gradient, index, stop);
        }
    }
    store_shared_def(world, entity, own);
}

/// `-fadeGradient:percent:` @ 0x100178aa0: moves every field of every stop of the own def
/// `percent` of the way toward `def_id`'s and rebuilds the geometry.
pub fn fade_gradient(world: &mut World, entity: Entity, def_id: &str, percent: f32) {
    let Some(source) = world.resource::<Library>().gradient_def(def_id).cloned() else {
        return;
    };
    let mut own = shared_def(world, world.obj::<Gradient>(entity));
    {
        let mut gradient = world.obj_mut::<Gradient>(entity);
        for index in 0..own.colors.len() {
            let Some(target) = source.colors.get(index) else {
                break;
            };
            let current = own.colors[index];
            let faded = GradientColor {
                pos: current.pos + (target.pos - current.pos) * percent,
                r: current.r + (target.r - current.r) * percent,
                g: current.g + (target.g - current.g) * percent,
                b: current.b + (target.b - current.b) * percent,
                a: current.a + (target.a - current.a) * percent,
            };
            own.colors[index] = faded;
            write_stop(&mut gradient, index, &faded);
        }
    }
    store_shared_def(world, entity, own);
}

/// `-gradientColorAtIndex:` @ 0x1001787bc.
pub fn gradient_color_at_index(world: &World, entity: Entity, index: usize) -> crate::library::GradientColor {
    let gradient = world.obj::<Gradient>(entity);
    let def = shared_def(world, gradient);
    def.colors[index]
}

/// The vertex-coloured, untextured strip (colour array on, texturing off); blending is
/// left as the previous draw set it.
fn draw_mesh(world: &World, entity: Entity, gl: &mut GlContext) {
    let gradient = world.obj::<Gradient>(entity);
    gl.set_texture_2d(false);
    gl.draw_colored_strip(&gradient.vertices, &[], &gradient.colors);
    gl.set_texture_2d(true);
}

/// `BCGradient::drawOpenGL:` @ 0x100178d18.
fn draw_gradient(world: &mut World, entity: Entity, gl: &mut GlContext) {
    let display = display::object(world, entity);
    if display.hidden {
        return;
    }
    let pos = display.pos;
    let rot = display.rot;
    let scale = display.scale;
    let pivot = display.pivot_pos;
    let origin = display.origin_pos;
    let pivot_x = scale.x * pivot.x;
    let pivot_y = scale.y * pivot.y;
    let pivot_z = scale.z * pivot.z;

    gl.push_matrix();
    gl.translate(pos.x, pos.y, pos.z);
    gl.translate(pivot_x, pivot_y, pivot_z);
    gl.rotate(rot.x, rot.y, rot.z);
    gl.scale(scale.x, scale.y, scale.z);
    gl.translate(-pivot.x, -pivot.y, -pivot.z);
    gl.translate(origin.x, origin.y, origin.z);
    draw_mesh(world, entity, gl);
    gl.pop_matrix();
}

/// `BCGradient::drawInMultiModelOpenGL:…` @ 0x100178e70.
fn draw_gradient_in_multi_model(world: &mut World, entity: Entity, gl: &mut GlContext, _parent: Entity, _rgba: [f32; 4]) {
    draw_mesh(world, entity, gl);
}

#[cfg(test)]
mod tests {
    use super::{scan_float, scan_hex, scan_stop};

    #[test]
    fn scan_float_matches_the_decimal_prefix() {
        assert_eq!(scan_float(b" 0.25:"), Some((0.25, 5)));
        assert_eq!(scan_float(b"-1e2x"), Some((-100.0, 4)));
        assert_eq!(scan_float(b".5"), Some((0.5, 2)));
        assert_eq!(scan_float(b"1e"), Some((1.0, 1)));
        assert_eq!(scan_float(b"-."), None);
        assert_eq!(scan_float(b""), None);
    }

    #[test]
    fn scan_hex_reads_an_unsigned_int() {
        assert_eq!(scan_hex(b"0xff1F1221"), Some(0xff1f1221));
        assert_eq!(scan_hex(b"f5FDEFE"), Some(0x0f5fdefe));
        assert_eq!(scan_hex(b"0xg"), Some(0));
        assert_eq!(scan_hex(b"-1"), Some(u32::MAX));
        assert_eq!(scan_hex(b"\x0b1f"), Some(0x1f));
        assert_eq!(scan_hex(b"100000002"), Some(2));
        assert_eq!(scan_hex(b"-123456789abcdef012"), Some(u32::MAX));
        assert_eq!(scan_hex(b"zz"), None);
    }

    #[test]
    fn a_stop_without_the_colon_keeps_a_zero_colour() {
        assert_eq!(scan_stop(b"0.5 0xffffffff"), (0.5, 0));
        assert_eq!(scan_stop(b"0.5:"), (0.5, 0));
        assert_eq!(scan_stop(b":0xff"), (0.0, 0));
    }
}
