use std::f32::consts::PI;
use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy::transform;
use bevy::{prelude::*, transform::commands, render::texture};
use rand::{thread_rng, Rng};
use crate::{GameTextures, WinSize, ENEMY_SPRITE, ENEMY_SIZE, SPRITE_SCALE, TIME_STEP, BASE_SPEED, EnemyCount, ENEMY_MAX};
use crate::components::{Enemy, SpriteSize, Laser, FromEnemy, Movable, Velocity};

use self::formation::{FormationMaker, Formation};

mod formation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, enemy_spawn_system.run_if(on_timer(Duration::from_secs(1))))
        .add_systems(Update, enemy_fire_system.run_if(enemy_fire_criteria))
        .add_systems(Update, enemy_movement_system)
        .insert_resource(FormationMaker::default());

    }
}

fn enemy_spawn_system(mut commands: Commands, 
    mut enemy_count: ResMut<EnemyCount>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
    mut formation_maker: ResMut<FormationMaker>,
){
    if enemy_count.0 < ENEMY_MAX {
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;
        // let mut rng = thread_rng();
        // let w_span = win_size.w / 2. -100.;
        // let h_span = win_size.h / 2. -100.;
        // let x = rng.gen_range(-w_span..w_span);
        // let y = rng.gen_range(-h_span..h_span);

        commands.spawn(SpriteBundle {
            texture: game_textures.enemy.clone(),
            transform: Transform {
                translation: Vec3::new(x, y, 10.),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Enemy)
        .insert(SpriteSize::from(ENEMY_SIZE))
        .insert(formation);

        enemy_count.0 += 1;
    }
}

fn enemy_fire_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    enemy_query: Query<&Transform, With<Enemy>>,
){
    for &tf in enemy_query.iter(){
        commands.spawn(SpriteBundle {
            texture: game_textures.enemy_laser.clone(),
            transform: Transform {
                translation: tf.translation,
                rotation: Quat::from_rotation_z(std::f32::consts::PI),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SpriteSize::from(ENEMY_SIZE))
        .insert(Laser)
        .insert(FromEnemy)
        .insert(Movable
            {auto_despawn: true,}
        )
        .insert(Velocity
            {x: 0., y: -1.,}
        );
    }

}

fn enemy_fire_criteria() -> bool {
    thread_rng().gen_bool(1. / 60.)
}

fn enemy_movement_system(
    mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>,
    time: Res<Time>,
){
    let now = time.elapsed_seconds_f64();

    for (mut tf, mut formation) in query.iter_mut(){
        //current position
        let (x_org, y_org) = (tf.translation.x, tf.translation.y);

        //max distance
        let max_distance =  TIME_STEP * BASE_SPEED;

        //fixtures
        let dir: f32 = if formation.start.0 < 0. {1.} else {-1.};
        let (x_pivot, y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;

        //calculate new position
        let angle = formation.angle + dir * formation.speed * TIME_STEP / (x_radius.min(y_radius) * PI / 2.);

        //compute target
        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        //compute distance
        let x_diff = x_dst - x_org;
        let y_diff = y_dst - y_org;
        let distance = (x_diff.powi(2) + y_diff.powi(2)).sqrt();
        let distance_ratio = if distance !=0. {max_distance / distance} else {0.};

        //compute final x/y
        let x = x_org + x_diff * distance_ratio;
        let x = if x_diff > 0. {x.max(x_dst)} else {x.min(x_dst)};
        let y = y_org + y_diff * distance_ratio;
        let y = if y_diff > 0. {y.max(y_dst)} else {y.min(y_dst)};

        if distance < max_distance * formation.speed / 20. {
            formation.angle = angle;
        }

        (tf.translation.x, tf.translation.y) = (x, y);

        // tf.translation.x += TIME_STEP * BASE_SPEED / 4.;
        // tf.translation.y += TIME_STEP * BASE_SPEED /4.;
    }
}