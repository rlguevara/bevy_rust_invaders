use std::time::Duration;

use bevy::time::common_conditions::on_timer;
use bevy::{prelude::*, transform};
use crate::{GameTextures, WinSize, PLAYER_SPRITE, PLAYER_SIZE, SPRITE_SCALE, TIME_STEP, BASE_SPEED, PlayerState, PLAYER_RESPAWN_DELAY};
use crate::components::{Velocity, Player, Movable, SpriteSize, FromPlayer, Laser};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(PlayerState::default())
        .add_systems(Update, player_spawn_system.run_if(on_timer(Duration::from_secs_f32(0.5))))
        .add_systems(Update, player_keyboard_event_system)
        .add_systems(Update, player_fire_system);
    }
}

fn player_spawn_system (
    mut commands: Commands, 
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
){
    let now = time.elapsed_seconds_f64();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1. || now > last_shot + PLAYER_RESPAWN_DELAY) {
        //add player
        let bottom = -win_size.h/2.;
        commands.spawn(SpriteBundle {
            texture: game_textures.player.clone(),
            transform: Transform {
                translation: Vec3::new(0., bottom + PLAYER_SIZE.1/2. * SPRITE_SCALE + 5., 10.),
                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player)
        .insert(Velocity {x: 0., y: 0.})
        .insert(Movable {auto_despawn: false})
        .insert(SpriteSize::from(PLAYER_SIZE));

        player_state.spawned();
    }
}

fn player_keyboard_event_system (
    kb: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
){
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.x = 0.;
        velocity.y = 0.;
        if kb.pressed(KeyCode::Left) {
            velocity.x -= 1.;
        }
        if kb.pressed(KeyCode::Right) {
            velocity.x += 1.;
        }
        if kb.pressed(KeyCode::Up) {
            velocity.y += 1.;
        }
        if kb.pressed(KeyCode::Down) {
            velocity.y -= 1.;
        }
    }
}

fn player_fire_system (
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Player>>,
){
    if let Ok(player_tf) = query.get_single() {
        if kb.just_pressed(KeyCode::Space) {
            let x_offset = PLAYER_SIZE.0/2. * SPRITE_SCALE -5.;
            
            let mut spawn_laser = |x_offset: f32| {
                commands.spawn(SpriteBundle {
                    texture: game_textures.player_laser.clone(),
                    transform: Transform {
                        translation: Vec3::new(player_tf.translation.x + x_offset, player_tf.translation.y +15., 10.),
                        scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Velocity {x: 0., y: 1.})
                .insert(Movable {auto_despawn: true})
                .insert(FromPlayer)
                .insert(SpriteSize::from(PLAYER_SIZE))
                .insert(Laser);
            };

            spawn_laser(x_offset);
            spawn_laser(-x_offset);
        }
    }
}
