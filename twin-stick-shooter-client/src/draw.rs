use cgmath::num_traits::clamp;
use cgmath::{EuclideanSpace, InnerSpace, Transform};
use js_sys::Array;
use legion::IntoQuery;
use twin_stick_shooter_core::collision::Shape;
use twin_stick_shooter_core::game::Game;
use twin_stick_shooter_core::health::HealthComponent;
use twin_stick_shooter_core::hitbox::{HitboxComponent, HitboxEffect, HurtboxComponent};
use twin_stick_shooter_core::interpolate::InterpolateComponent;
use twin_stick_shooter_core::model::ModelComponent;
use twin_stick_shooter_core::physics::VelocityComponent;
use twin_stick_shooter_core::player::PlayerComponent;
use twin_stick_shooter_core::resource::Input;
use twin_stick_shooter_core::util::clamp_magnitude;
use twin_stick_shooter_core::{translation, Mat3, Pt2};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::model::ModelManager;
use crate::DebugState;

pub fn draw(
    canvas: &HtmlCanvasElement,
    ctx: &CanvasRenderingContext2d,
    model_manager: &ModelManager,
    game: &Game,
    input: &Input,
    debug: &DebugState,
) {
    let w = canvas.width() as f64;
    let h = canvas.height() as f64;
    ctx.reset_transform().unwrap();
    ctx.set_fill_style(&JsValue::from_str("#000"));
    ctx.fill_rect(0.0, 0.0, w, h);
    ctx.translate(0.5 * w, 0.5 * h).unwrap();

    let s = w.min(h) / 800.0;
    ctx.scale(s, s).unwrap();

    if debug.draw_hitboxes {
        debug_draw_hitboxes(ctx, game);
        debug_draw_hurtboxes(ctx, game);
    } else {
        draw_models(ctx, model_manager, game);
        draw_players(ctx, game, input);
    }
}

fn draw_players(ctx: &CanvasRenderingContext2d, game: &Game, input: &Input) {
    // Draw players.
    for (
        &InterpolateComponent {
            interpolated_pos: pos,
            ..
        },
        &VelocityComponent(vel),
        player,
    ) in
        <(&InterpolateComponent, &VelocityComponent, &PlayerComponent)>::query().iter(game.world())
    {
        if player.docked_to.is_some() {
            continue;
        }

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

fn draw_models(ctx: &CanvasRenderingContext2d, model_manager: &ModelManager, game: &Game) {
    for (
        &InterpolateComponent {
            interpolated_pos: pos,
            ..
        },
        model,
        health,
    ) in <(
        &InterpolateComponent,
        &ModelComponent,
        Option<&HealthComponent>,
    )>::query()
    .iter(game.world())
    {
        ctx.save();
        let xform = <Mat3 as Transform<Pt2>>::concat(&translation(pos.to_vec()), &model.transform);
        ctx.transform(
            xform.x.x as f64,
            xform.x.y as f64,
            xform.y.x as f64,
            xform.y.y as f64,
            xform.z.x as f64,
            xform.z.y as f64,
        )
        .unwrap();
        model_manager.get(&model.name).draw(
            ctx,
            health
                .map(|health| health.is_hit_flashing())
                .unwrap_or(false),
        );
        ctx.restore();
    }
}

fn debug_draw_hurtboxes(ctx: &CanvasRenderingContext2d, game: &Game) {
    for (
        &InterpolateComponent {
            interpolated_pos: pos,
            ..
        },
        hurtbox,
    ) in <(&InterpolateComponent, &HurtboxComponent)>::query().iter(game.world())
    {
        let fill_style = JsValue::from_str(&"#44f8");
        match &hurtbox.shape {
            Shape::Circle(circle) => {
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
                ctx.set_fill_style(&fill_style);
                ctx.fill();
            }
            _ => (),
        }
    }
}

fn debug_draw_hitboxes(ctx: &CanvasRenderingContext2d, game: &Game) {
    for (
        &InterpolateComponent {
            interpolated_pos: pos,
            ..
        },
        hitbox,
    ) in <(&InterpolateComponent, &HitboxComponent)>::query().iter(game.world())
    {
        let fill_style = JsValue::from_str(match hitbox.effect {
            HitboxEffect::None => "#8888",
            HitboxEffect::Damage(_) => "#f448",
            HitboxEffect::StationDock => "#f4f8",
        });
        match &hitbox.shape {
            Shape::Circle(circle) => {
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

                ctx.set_fill_style(&fill_style);
                ctx.fill();
            }
            _ => (),
        }
    }
}
