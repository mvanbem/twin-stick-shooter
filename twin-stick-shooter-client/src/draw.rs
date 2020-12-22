use cgmath::num_traits::clamp;
use cgmath::InnerSpace;
use js_sys::Array;
use legion::IntoQuery;
use twin_stick_shooter_core::collision::Shape;
use twin_stick_shooter_core::component::{
    Hitbox, HitboxState, Hurtbox, HurtboxState, InterpolatedPosition, Player, Velocity,
};
use twin_stick_shooter_core::resource::Input;
use twin_stick_shooter_core::util::clamp_magnitude;
use twin_stick_shooter_core::Game;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::style::{ACCENT_COLOR, PRIMARY_COLOR};

pub fn draw(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
    game: &Game,
    input: &Input,
) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    ctx.reset_transform().unwrap();
    ctx.set_fill_style(&JsValue::from_str("#000"));
    ctx.fill_rect(0.0, 0.0, w, h);
    ctx.translate(0.5 * w, 0.5 * h).unwrap();

    let s = w.min(h) / 800.0;
    ctx.scale(s, s).unwrap();

    draw_players(ctx, game, input);
    draw_hitboxes(ctx, game);
    draw_hurtboxes(ctx, game);
}

fn draw_players(ctx: &CanvasRenderingContext2d, game: &Game, input: &Input) {
    // Draw players.
    for (&InterpolatedPosition(pos), &Velocity(vel), _) in
        <(&InterpolatedPosition, &Velocity, &Player)>::query().iter(game.world())
    {
        // Draw a line indicating the aim direction.
        let end = {
            let bullet_vel = vel + input.aim.normalize_to(1000.0);
            pos + bullet_vel.normalize_to(500.0)
        };
        let alpha = clamp(input.aim.magnitude() * 2.0 - 1.0, 0.0, 1.0);
        let gradient = {
            let gradient = ctx
                .create_radial_gradient(
                    pos.x as f64,
                    pos.y as f64,
                    50.0,
                    pos.x as f64,
                    pos.y as f64,
                    500.0,
                )
                .unwrap();
            gradient
                .add_color_stop(0.0, &format!("rgba(153, 153, 153, {})", alpha))
                .unwrap();
            gradient
                .add_color_stop(1.0, "rgba(153, 153, 153, 0)")
                .unwrap();
            gradient
        };
        ctx.begin_path();
        ctx.move_to(pos.x as f64, pos.y as f64);
        ctx.line_to(end.x as f64, end.y as f64);
        ctx.set_stroke_style(gradient.as_ref());
        ctx.set_line_width(1.0);
        ctx.set_line_dash(
            &(&[10.0, 10.0])
                .iter()
                .map(|x| JsValue::from_f64(*x))
                .collect::<Array>(),
        )
        .unwrap();
        ctx.stroke();
        ctx.set_line_dash(&Array::new()).unwrap();

        // Draw a circle indicating the limit of the aim stick.
        ctx.begin_path();
        ctx.arc(pos.x as f64, pos.y as f64, 50.0, 0.0, std::f64::consts::TAU)
            .unwrap();
        ctx.close_path();
        ctx.set_stroke_style(&JsValue::from_str(&format!(
            "rgba(153, 153, 153, {})",
            alpha,
        )));
        ctx.stroke();

        // Draw the player ship.
        ctx.begin_path();
        ctx.arc(pos.x as f64, pos.y as f64, 20.0, 0.0, std::f64::consts::TAU)
            .unwrap();
        ctx.close_path();

        ctx.set_fill_style(&JsValue::from_str("#4f4"));
        ctx.fill();

        ctx.set_stroke_style(&JsValue::from_str("#282"));
        ctx.set_line_width(2.0);
        ctx.stroke();

        // Draw a circle indicating the aim stick position.
        let point = pos + 50.0 * clamp_magnitude(input.aim, 0.0, 1.0);
        ctx.begin_path();
        ctx.arc(
            point.x as f64,
            point.y as f64,
            10.0,
            0.0,
            std::f64::consts::TAU,
        )
        .unwrap();
        ctx.close_path();
        ctx.set_stroke_style(&JsValue::from_str("#999"));
        ctx.set_line_width(1.0);
        ctx.stroke();
    }
}

fn draw_hurtboxes(ctx: &CanvasRenderingContext2d, game: &Game) {
    for (&InterpolatedPosition(pos), hitbox, hitbox_state) in
        <(&InterpolatedPosition, &Hurtbox, &HurtboxState)>::query().iter(game.world())
    {
        match &hitbox.shape {
            Shape::Circle(circle) => {
                let color = if hitbox_state.hit_by_entities.is_empty() {
                    format!("{}80", PRIMARY_COLOR)
                } else {
                    "#fff8".to_string()
                };

                ctx.begin_path();
                ctx.arc(
                    pos.x as f64,
                    pos.y as f64,
                    circle.radius as f64,
                    0.0,
                    std::f64::consts::TAU,
                )
                .unwrap();
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&color));
                ctx.fill();
            }
        }
    }
}

fn draw_hitboxes(ctx: &CanvasRenderingContext2d, game: &Game) {
    for (&InterpolatedPosition(pos), hitbox, hitbox_state) in
        <(&InterpolatedPosition, &Hitbox, &HitboxState)>::query().iter(game.world())
    {
        match &hitbox.shape {
            Shape::Circle(circle) => {
                let color = if hitbox_state.hit_entities.is_empty() {
                    format!("{}80", ACCENT_COLOR)
                } else {
                    "#fff8".to_string()
                };

                ctx.begin_path();
                ctx.arc(
                    pos.x as f64,
                    pos.y as f64,
                    circle.radius as f64,
                    0.0,
                    std::f64::consts::TAU,
                )
                .unwrap();
                ctx.close_path();

                ctx.set_fill_style(&JsValue::from_str(&color));
                ctx.fill();
            }
        }
    }
}
