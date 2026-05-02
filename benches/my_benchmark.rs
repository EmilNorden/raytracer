use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use raytracer::content::gltf::loader::GltfLoader;
use raytracer::content::scene_loader::SceneLoader;
use raytracer::frame::Frame;
use raytracer::integrator::integrator::{Integrator, IntegratorImpl};
use raytracer::integrator::pathtracing::PathTracingIntegrator;
use raytracer::options::RenderOptions;

fn bench_render(c: &mut Criterion) {
    let mut group = c.benchmark_group("render");
    group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(25))
        .warm_up_time(std::time::Duration::from_secs(2));

    let scenes = [
        ("alley", "benches/alley/alley.gltf", 20u32),
        ("materials", "benches/refr/refr.gltf", 20u32),
    ];

    for (name, path, samples) in scenes {
        let options = RenderOptions {
            scene_file: path.to_string(),
            output_folder: "".to_string(),
            resolution: raytracer::options::Resolution { width: 1024, height: 768 },
            samples,
            frame_rate: 0,
            debug: false,
            max_bounces: 4,
            video: false,
        };

        let (scene, _animation_controller) =
            GltfLoader::load_scene(&options.scene_file, &options).unwrap();
        let integrator = IntegratorImpl::Pathtracing(PathTracingIntegrator::new());

        group.bench_with_input(BenchmarkId::new("path_tracer", name), &options, |b, opts| {
            b.iter_batched(
                || Frame::new(opts.resolution.width, opts.resolution.height),
                |mut frame| {
                    integrator.integrate(&scene, &mut frame, opts.samples);
                },
                BatchSize::SmallInput,
            )
        });
    }
    /*group
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(25))
        .warm_up_time(std::time::Duration::from_secs(2));

    let options = RenderOptions {
        scene_file: "benches/alley/alley.gltf".to_string(),
        output_folder: "".to_string(),
        resolution: raytracer::options::Resolution { width: 1024, height: 768 },
        samples: 20,
        frame_rate: 0,
        debug: false,
        max_bounces: 4,
        video: false,
    };

    let (scene, _animation_controller) = GltfLoader::load_scene(&options.scene_file, &options).unwrap();
    let integrator = IntegratorImpl::Pathtracing(PathTracingIntegrator::new());

    group.bench_function("path_tracer", |b| {
        b.iter_batched(
            || Frame::new(1024, 768),
            |mut frame| {
                black_box(integrator.integrate(&scene, &mut frame, options.samples));
            },
            BatchSize::SmallInput,
        )
    });*/

    group.finish();
}


criterion_group!(benches, bench_render);
criterion_main!(benches);
