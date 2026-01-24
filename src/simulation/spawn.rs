use bevy::prelude::*;
use rand::Rng;

pub struct Spawner {
    particle_spawn_density: u32,
    initial_vel: Vec3,
    jitter_strength: f32,
    spawn_region: SpawnRegion,
}

impl Spawner {
    pub fn new(particle_spawn_density: u32, initial_vel: Vec3, jitter_strength: f32, spawn_region: SpawnRegion) -> Self {
        Self {
            particle_spawn_density,
            initial_vel,
            jitter_strength,
            spawn_region,
        }
    }

    pub fn spawn(&self) -> SpawnData {
        let mut points = Vec::new();
        let mut velocities = Vec::new();

        let region = &self.spawn_region;
        let particles_per_axis = region.calc_part_count_per_axis(self.particle_spawn_density);
        let (p, v) = self.spawn_cube(particles_per_axis, region.centre, Vec3::ONE * region.size);
        points.extend(p);
        velocities.extend(v);

        SpawnData { points, velocities }
    }

    pub fn spawn_cube(&self, particles_per_axis: u32, centre: Vec3, size: Vec3) -> (Vec<Vec3>, Vec<Vec3>) {
        let mut rng = rand::rng();

        let num_points = particles_per_axis * particles_per_axis * particles_per_axis;
        let mut points = Vec::with_capacity(num_points as usize);
        let mut velocities = Vec::with_capacity(num_points as usize);

        for x in 0..particles_per_axis {
            for y in 0..particles_per_axis {
                for z in 0..particles_per_axis {
                    let tx = x as f32 / particles_per_axis as f32;
                    let ty = y as f32 / particles_per_axis as f32;
                    let tz = z as f32 / particles_per_axis as f32;

                    let px  = (tx - 0.5) * size.x + centre.x;
                    let py  = (ty - 0.5) * size.y + centre.y;
                    let pz  = (tz - 0.5) * size.z + centre.z;

                    let jitter: Vec3 = random_inside_unit_sphere(&mut rng) * self.jitter_strength;
                    let point = Vec3::new(px, py, pz) + jitter;

                    points.push(point);
                    velocities.push(self.initial_vel);
                }
            }
        }

        return (points, velocities)
    }

}

pub struct SpawnRegion {
    centre: Vec3,
    size: f32
}

impl SpawnRegion {
    pub fn new(centre: Vec3, size: f32) -> Self {
        Self { centre, size }
    }

    pub fn volume(&self) -> f32 {
        self.size * self.size * self.size
    }

    pub fn calc_part_count_per_axis(&self, particle_density: u32) -> u32 {
        let target_particle_count = (self.volume() * particle_density as f32) as u32;
        let particles_per_axis = (target_particle_count as f32).cbrt().ceil() as u32;
        return particles_per_axis;
    }
}

pub struct SpawnData {
    pub points: Vec<Vec3>,
    pub velocities: Vec<Vec3>,
}


fn random_inside_unit_sphere(rng: &mut impl Rng) -> Vec3 {
    let dir = Vec3::new(
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
        rng.random_range(-1.0..1.0),
    )
    .normalize_or_zero();

    let radius = rng.random::<f32>().cbrt(); // wichtig für gleichmäßige Verteilung
    dir * radius
}

// Verwendung