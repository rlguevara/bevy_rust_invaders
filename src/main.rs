#![allow(unused)]

use std::slice::Windows;
use bevy::{window::PrimaryWindow, math::Vec3Swizzles, sprite::collide_aabb::collide, ecs::query::{self, Has}, utils::HashSet};
use components::{FromPlayer, SpriteSize, Laser, FromEnemy, Enemy, ExplosionToSpawn, Explosion, ExplosionTimer, Player};
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use crate::components::{Velocity, Movable};

use bevy::prelude::*;

mod player;
mod enemy;
mod components;

const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_SIZE: (f32, f32) = (144.,75.);
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9.,54.);
const PLAYER_RESPAWN_DELAY: f64 = 2.;

const SPRITE_SCALE: f32 = 0.5;

const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 100.;

const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);
const ENEMY_MAX: u32 = 2;
const FORMATION_MEMBERS_MAX: u32 = 2;

const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
const EXPLOSION_LENGTH: usize = 16;

//Resources
#[derive(Resource)]
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}

#[derive(Resource)]
struct GameTextures{
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
    explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
struct EnemyCount(u32);

#[derive(Resource)]
struct PlayerState {
    on: bool,
    last_shot: f64,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64){
        self.on = false;
        self.last_shot = time;
    }

    fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
    }
}



fn main() {
    App::new()
    .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Bevy Rust Invaders".to_string(),
            resolution: (598.,676.).into(),
            ..Default::default()
        }),
        ..Default::default()
    }))
    .add_plugins(PlayerPlugin)
    .add_plugins(EnemyPlugin)
    .add_systems(Startup, setup_system)
    .add_systems(Update, movable_system)
    .add_systems(Update, player_laser_hit_enemy_system)
    .add_systems(Update, enemy_laser_hit_player_system)
    .add_systems(Update, explosion_to_spawn_system)
    .add_systems(Update, explosion_animation_system)
    .run();
}

//Systems
fn setup_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>, //texture_atlases is a resource for grids
    query: Query<&Window, With<PrimaryWindow>>) {
    //add camera
    commands.spawn(Camera2dBundle::default());

    //capture window size
    let Ok(primary) = query.get_single() else {
        return;
    };
    let (win_w, win_h) = (primary.width(), primary.height());

    //position window
    // primary.set_position(IVec2::new(2780, 4900));

     //window size resource
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    //create explosion texture atlas
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(texture_atlas);

    //Game Textures resource
    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));

}

fn movable_system (
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        transform.translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn{
            //despawn when out of screen
            const MARGIN: f32 = 200.;
            if transform.translation.x < -win_size.w/2. - MARGIN ||
            transform.translation.x > win_size.w/2. + MARGIN ||
            transform.translation.y < -win_size.h/2. - MARGIN ||
            transform.translation.y > win_size.h/2. + MARGIN {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    //iterate through the lasers
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        let laser_scale = Vec2::from(laser_tf.scale.xy());

        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        //iterate through the enemies
        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter(){
            let enemey_scale = Vec2::from(enemy_tf.scale.xy());
            
            if despawned_entities.contains(&enemy_entity) || despawned_entities.contains(&laser_entity){
                continue;
            }

            //check for collision
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemey_scale,
            );

            //perform collision
            if let Some(_) = collision {
                //remove laser
                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);

                //remove enemy
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;

                //spawn explosion
                commands.spawn(ExplosionToSpawn(enemy_tf.translation));
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
){
    for (entity, explosion_to_spawn) in query.iter() {
        //spawn explosion
        commands.spawn(SpriteSheetBundle {
            texture_atlas: game_textures.explosion.clone(),
            transform: Transform {
                translation: explosion_to_spawn.0,
                scale: Vec3::splat(0.5),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Explosion)
        .insert(ExplosionTimer::default());

        //despawn entity
        commands.entity(entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(Entity, &mut TextureAtlasSprite, &mut ExplosionTimer), With<Explosion>>,
){
    for (entity, mut sprite, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; //move to next sprite cell
            if sprite.index >= EXPLOSION_LENGTH {
                commands.entity(entity).despawn();
            } 
        }
    }
}

fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
){
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = Vec2::from(laser_tf.scale.xy());
            let player_scale = Vec2::from(player_tf.scale.xy());

            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );

            if let Some(_) = collision {
                commands.entity(laser_entity).despawn();
                commands.entity(player_entity).despawn();
                player_state.shot(time.elapsed_seconds_f64());
                commands.spawn(ExplosionToSpawn(player_tf.translation));
                break;
            }
        }
    }
}