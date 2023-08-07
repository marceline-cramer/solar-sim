use std::f32::consts::TAU;

use ambient_api::{
    components::core::{
        app::main_scene,
        camera::aspect_ratio_from_window,
        primitives::sphere_radius,
        rendering::color,
        transform::{local_to_parent, lookat_target, translation, scale},
    },
    concepts::{make_perspective_infinite_reverse_camera, make_sphere, make_transformable},
    prelude::{delta_time, *},
};

use components::*;

#[main]
pub fn main() {
    Entity::new()
        .with_merge(make_perspective_infinite_reverse_camera())
        .with(aspect_ratio_from_window(), EntityId::resources())
        .with_default(main_scene())
        .with(translation(), Vec3::splat(15.0))
        .with(lookat_target(), vec3(0., 0., 0.))
        .spawn();

    // make sun
    let sun = make_transformable()
        .with_merge(make_sphere())
        .with(sphere_radius(), 1.0)
        .spawn();

    // make planets
    for radius in 1..=5 {
        let radius = radius as f32 * 2.0;
        let speed = TAU / radius;
        let new_color = random::<Vec3>() * 0.5 + 0.5;
        let planet = make_transformable()
            .with_merge(make_sphere())
            .with_default(planet())
            .with(sphere_radius(), 0.4)
            .with_default(local_to_parent())
            .with(orbit_speed(), speed)
            .with(orbit_radius(), radius)
            .with(orbit_progress(), 0.0)
            .with(color(), new_color.extend(1.0))
            .spawn();

        entity::add_child(sun, planet);
    }

    query((orbit_speed(), orbit_progress())).each_frame(move |entities| {
        for (e, (speed, progress)) in entities {
            let new_progress = progress + speed * delta_time();
            entity::set_component(e, orbit_progress(), new_progress);
        }
    });

    query((orbit_radius(), orbit_progress())).each_frame(move |entities| {
        for (e, (radius, progress)) in entities {
            let unit = Vec2::from_angle(progress);
            let offset = unit * radius;
            entity::add_component(e, translation(), offset.extend(0.0));
        }
    });

    let planets = query((color(), translation())).requires(planet()).build();
    run_async(async move {
        loop {
            sleep(0.1).await;

            for (_e, (planet_color, position)) in planets.evaluate() {
                make_transformable()
                    .with_merge(make_sphere())
                    .with(translation(), position)
                    .with(sphere_radius(), 0.1)
                    .with(color(), planet_color)
                    .with(lifespan(), 5.0)
                    .spawn();
            }
        }
    });

    query(lifespan()).each_frame(move |entities| {
        for (e, old_lifespan) in entities {
            let new_lifespan = old_lifespan - delta_time();

            if new_lifespan < 0.0 {
                entity::despawn_recursive(e);
            } else {
                let new_scale = new_lifespan / 5.0;
                entity::set_component(e, lifespan(), new_lifespan);
                entity::set_component(e, scale(), Vec3::splat(new_scale));
            }
        }
    });
}
