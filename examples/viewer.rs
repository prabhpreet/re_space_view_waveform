use re_space_view_waveform::types::archetypes::WaveformPoint;
use re_types::datatypes::{AnnotationInfo, ClassDescription, ClassDescriptionMapElem, Rgba32};
use re_viewer::external::{re_log, re_memory};

#[global_allocator]
static GLOBAL: re_memory::AccountingAllocator<mimalloc::MiMalloc> =
    re_memory::AccountingAllocator::new(mimalloc::MiMalloc);

pub struct CustomSink {
    channel: re_smart_channel::Sender<re_sdk::log::LogMsg>,
}

impl CustomSink {
    pub fn new(channel: re_smart_channel::Sender<re_sdk::log::LogMsg>) -> Self {
        Self { channel }
    }
}

impl re_sdk::sink::LogSink for CustomSink {
    fn send(&self, msg: re_sdk::log::LogMsg) {
        self.channel.send(msg).unwrap();
    }

    fn flush_blocking(&self) {
        self.channel.flush_blocking().unwrap();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    re_log::setup_logging();
    re_crash_handler::install_crash_handlers(re_viewer::build_info());

    let (rec, rx) = re_smart_channel::smart_channel(
        re_smart_channel::SmartMessageSource::Sdk,
        re_smart_channel::SmartChannelSource::Sdk,
    );

    let startup_options = re_viewer::StartupOptions {
        hide_welcome_screen: true,
        ..Default::default()
    };

    let app_env = re_viewer::AppEnvironment::Custom("waveform example".to_string());

    let handle = std::thread::spawn(move || {
        let application_id = "waveform_example".to_string();
        let sink = CustomSink::new(rec);
        let rec = re_sdk::RecordingStream::new(
            re_sdk::new_store_info(application_id),
            re_sdk::log::ChunkBatcherConfig::DEFAULT,
            Box::new(sink),
        )
        .unwrap();

        //Set annotation context classes 0 and 1 for "ON" and "OFF" respectively
        let d_path = ["D/d1", "D/d2"];
        //Plot 2 digital with a random initial value
        let d_init = [rand::random::<bool>(); 2];
        let d_normal = [false; 2];

        for ((&d, &d_init), &d_normal) in d_path.iter().zip(d_init.iter()).zip(d_normal.iter()) {
            rec.log_static(
                d,
                &re_types::archetypes::AnnotationContext {
                    context: re_types::components::AnnotationContext(vec![
                        ClassDescriptionMapElem {
                            class_id: re_types::datatypes::ClassId(0),
                            class_description: ClassDescription {
                                info: AnnotationInfo {
                                    id: 0,
                                    label: Some("OFF".into()),
                                    color: Some(Rgba32::from_rgb(100, 100, 255)),
                                },
                                ..Default::default()
                            },
                        },
                        ClassDescriptionMapElem {
                            class_id: re_types::datatypes::ClassId(1),
                            class_description: ClassDescription {
                                info: AnnotationInfo {
                                    id: 1,
                                    label: Some("ON".into()),
                                    color: Some(Rgba32::from_rgb(10, 10, 255)),
                                },
                                ..Default::default()
                            },
                        },
                    ]),
                },
            )
            .unwrap();

            let d_init = if d_init { 1 } else { 0 };
            let d_normal = if d_normal { 1 } else { 0 };

            rec.log_static(d, &WaveformPoint::new_discrete_state_init(d_init))
                .unwrap();
            rec.log_static(d, &WaveformPoint::new_discrete_state_normal(d_normal))
                .unwrap();
        }

        let e_paths = ["E/e1", "E/e2"];
        for &e in e_paths.iter() {
            rec.log_static(
                e,
                &re_types::archetypes::AnnotationContext {
                    context: re_types::components::AnnotationContext(vec![
                        ClassDescriptionMapElem {
                            class_id: re_types::datatypes::ClassId(2),
                            class_description: ClassDescription {
                                info: AnnotationInfo {
                                    id: 2,
                                    label: Some("T1".into()),
                                    color: Some(Rgba32::from_rgb(255, 0, 0)),
                                },
                                ..Default::default()
                            },
                        },
                        ClassDescriptionMapElem {
                            class_id: re_types::datatypes::ClassId(3),
                            class_description: ClassDescription {
                                info: AnnotationInfo {
                                    id: 3,
                                    label: Some("T2".into()),
                                    color: Some(Rgba32::from_rgb(140, 240, 0)),
                                },
                                ..Default::default()
                            },
                        },
                    ]),
                },
            )
            .unwrap();
        }

        let mut d_state = d_init;

        let y_paths = ["A/y1", "A/y2", "B/y3", "C/y4", "C/y5"];

        let y_cl = vec![
            |t: f64| -> f64 {
                (2.0 * std::f64::consts::PI * 0.5 * t).sin() + 0.1 * rand::random::<f64>()
            },
            |t: f64| -> f64 {
                (3.0 * std::f64::consts::PI * 0.5 * t).cos() + 0.3 * rand::random::<f64>()
            },
            |t: f64| -> f64 {
                (4.0 * std::f64::consts::PI * 0.5 * t + (std::f64::consts::PI / 3.0)).sin()
            },
            |t: f64| -> f64 {
                (5.0 * std::f64::consts::PI * 0.5 * t + (std::f64::consts::PI * 2.0 / 3.0)).sin()
                    + 0.1 * rand::random::<f64>()
            },
            |t: f64| -> f64 {
                (6.0 * std::f64::consts::PI * 0.5 * t).cos() + 0.6 * rand::random::<f64>()
            },
        ];

        //Plot sine waveform with analogs for 10 seconds at 1kHz sample rate with random noise, toggle digital states every 1 second, toggle 2 events every 3 seconds
        for i in 0..10_000 {
            let t = i as f64 / 1_000.0;

            for (&y, &cl) in y_paths.iter().zip(y_cl.iter()) {
                rec.log(y, &WaveformPoint::new_scalar(cl(t))).unwrap();
            }

            //Toggle digital states every 1 second
            if i % 1_000 == 0 {
                d_state = [!d_state[0], !d_state[1]];

                for (&d, &d_state) in d_path.iter().zip(d_state.iter()) {
                    rec.log(
                        d,
                        &WaveformPoint::new_discrete_state(if d_state { 1 } else { 0 }),
                    )
                    .unwrap();
                }
            }

            //Randomly toggle either event every 3 seconds
            if i % 3_000 == 0 {
                if rand::random::<bool>() {
                    rec.log("E/e1", &WaveformPoint::new_event(2)).unwrap();
                } else {
                    rec.log("E/e2", &WaveformPoint::new_event(3)).unwrap();
                }
            }
        }
    });

    re_viewer::run_native_app(
        Box::new(move |cc| {
            let mut app = re_viewer::App::new(
                re_viewer::build_info(),
                &app_env,
                startup_options,
                cc.egui_ctx.clone(),
                cc.storage,
            );
            app.add_receiver(rx);

            app.add_space_view_class::<re_space_view_waveform::WaveformSpaceView>()
                .unwrap();

            Box::new(app)
        }),
        None,
    )?;

    handle.join().unwrap();
    Ok(())
}
