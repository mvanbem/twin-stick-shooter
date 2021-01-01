use std::collections::HashMap;

use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub struct Model {
    draw_fn: Box<dyn Fn(&CanvasRenderingContext2d, bool)>,
}

impl Model {
    pub fn new(draw_fn: Box<dyn Fn(&CanvasRenderingContext2d, bool)>) -> Model {
        Model { draw_fn }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, is_hit_flashing: bool) {
        (self.draw_fn)(ctx, is_hit_flashing);
    }
}

pub struct ModelManager {
    models: HashMap<String, Model>,
    fallback: Model,
}

impl ModelManager {
    pub fn insert(&mut self, name: String, model: Model) {
        self.models.insert(name, model);
    }

    pub fn get(&self, name: &str) -> &Model {
        self.models.get(name).unwrap_or(&self.fallback)
    }
}

impl Default for ModelManager {
    fn default() -> Self {
        let mut model_manager = ModelManager {
            models: HashMap::new(),
            fallback: fallback_model(),
        };
        model_manager.insert("ships/player".to_string(), player_ship());
        model_manager.insert("ships/station".to_string(), station());
        model_manager.insert("test/target".to_string(), target());
        model_manager.insert("shots/lemon".to_string(), lemon());
        model_manager
    }
}

fn fallback_model() -> Model {
    Model::new(Box::new(|ctx, _is_hit_flashing| {
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 25.0, 0.0, std::f64::consts::TAU).unwrap();
        ctx.set_fill_style(&JsValue::from_str("#f0f"));
        ctx.fill();

        ctx.begin_path();
        ctx.move_to(-15.0, -15.0);
        ctx.line_to(15.0, 15.0);
        ctx.move_to(-15.0, 15.0);
        ctx.line_to(15.0, -15.0);
        ctx.set_stroke_style(&JsValue::from_str("#000"));
        ctx.set_line_width(5.0);
        ctx.stroke();

        ctx.set_font("10px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_stroke_style(&JsValue::from_str("#000"));
        ctx.set_line_width(3.0);
        ctx.set_line_cap("round");
        ctx.set_line_join("round");
        ctx.set_fill_style(&JsValue::from_str("#fff"));
        ctx.stroke_text("MISSING", 0.0, 0.0).unwrap();
        ctx.fill_text("MISSING", 0.0, 0.0).unwrap();
    }))
}

fn player_ship() -> Model {
    Model::new(Box::new(|ctx, is_hit_flashing| {
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 20.0, 0.0, std::f64::consts::TAU).unwrap();
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#4f4"
        }));
        ctx.fill();

        ctx.set_stroke_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#282"
        }));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }))
}

fn station() -> Model {
    Model::new(Box::new(|ctx, is_hit_flashing| {
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 50.0, 0.0, std::f64::consts::TAU).unwrap();
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#4f4"
        }));
        ctx.fill();

        ctx.set_stroke_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#282"
        }));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }))
}

fn target() -> Model {
    Model::new(Box::new(|ctx, is_hit_flashing| {
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 20.0, 0.0, std::f64::consts::TAU).unwrap();
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#44f"
        }));
        ctx.fill();

        ctx.set_stroke_style(&JsValue::from_str(if is_hit_flashing {
            "#fff"
        } else {
            "#228"
        }));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }))
}

fn lemon() -> Model {
    Model::new(Box::new(|ctx, _is_hit_flashing| {
        ctx.begin_path();
        ctx.arc(0.0, 0.0, 5.0, 0.0, std::f64::consts::TAU).unwrap();

        ctx.set_fill_style(&JsValue::from_str("#ff4"));
        ctx.fill();

        ctx.set_stroke_style(&JsValue::from_str("#cc2"));
        ctx.set_line_width(2.0);
        ctx.stroke();
    }))
}
