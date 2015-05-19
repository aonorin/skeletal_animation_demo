extern crate camera_controllers;
extern crate collada;
extern crate dev_menu;
extern crate env_logger;
extern crate gfx;
extern crate gfx_debug_draw;
extern crate gfx_device_gl;
extern crate gfx_gl as gl;
extern crate piston;
extern crate piston_window;
extern crate sdl2;
extern crate sdl2_window;
extern crate shader_version;
extern crate skeletal_animation;
extern crate vecmath;

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use gfx_debug_draw::DebugRenderer;

use piston::window::{
    WindowSettings,
};

use piston::event::*;

use vecmath::{mat4_id};

use sdl2_window::Sdl2Window;

use camera_controllers::{
    OrbitZoomCamera,
    OrbitZoomCameraSettings,
    CameraPerspective,
    model_view_projection
};

mod demo;
use demo::Settings;

fn main() {

    env_logger::init().unwrap();

    let (win_width, win_height) = (640, 480);
    let window = Rc::new(RefCell::new(Sdl2Window::new(
        shader_version::OpenGL::_3_2,
        WindowSettings::new(
            "Skeletal Animation Demo".to_string(),
            piston::window::Size { width: 640, height: 480 }
        ).exit_on_esc(true)
    )));

    let piston_window = piston_window::PistonWindow::new(window, piston_window::empty_app());

    let factory = piston_window.device.borrow_mut().spawn_factory();
    let mut debug_renderer = DebugRenderer::new(factory, 64).ok().unwrap();

    let model = mat4_id();
    let mut projection = CameraPerspective {
        fov: 90.0f32,
        near_clip: 0.1,
        far_clip: 1000.0,
        aspect_ratio: (win_width as f32) / (win_height as f32)
    }.projection();

    let mut orbit_zoom_camera: OrbitZoomCamera<f32> = OrbitZoomCamera::new(
        [0.0, 0.0, 0.0],
        OrbitZoomCameraSettings::default()
    );

    // Start event loop

    let mut settings = Settings {

        use_dlb: true,
        draw_skeleton: true,
        draw_labels: false,
        draw_mesh: true,
        playback_speed: 1.0,

        params: HashMap::new(),
    };

    let mut menu = dev_menu::Menu::<Settings>::new();

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle DLB/LBS Skinning",
        Box::new( |ref mut settings| {
            settings.use_dlb = !settings.use_dlb;
        })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Skeleton",
        Box::new( |ref mut settings| { settings.draw_skeleton = !settings.draw_skeleton; })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Joint Labels",
        Box::new( |ref mut settings| { settings.draw_labels = !settings.draw_labels; })
    ));

    menu.add_item(dev_menu::MenuItem::action_item(
        "Toggle Mesh",
        Box::new( |ref mut settings| { settings.draw_mesh = !settings.draw_mesh; })
    ));

    menu.add_item(dev_menu::MenuItem::slider_item(
        "Playback Speed = ",
        [-5.0, 5.0],
        0.01,
        Box::new( |ref settings| { settings.playback_speed }),
        Box::new( |ref mut settings, value| { settings.playback_speed = value }),
    ));

    let mut lbs_demo = {
        let factory = piston_window.device.borrow_mut().spawn_factory();
        demo::lbs_demo(factory)
    };

    let mut dlb_demo = {
        let factory = piston_window.device.borrow_mut().spawn_factory();
        demo::dlb_demo(factory)
    };

    for (param, &value) in dlb_demo.controller.get_parameters().iter() {
        settings.params.insert(param.clone(), value);

        // Apparently need to make our own string copies to move into each closure..
        let param_copy_1 = param.clone();
        let param_copy_2 = param.clone();

        menu.add_item(dev_menu::MenuItem::slider_item(
            &format!("Param[{}] = ", param)[..],
            [0.0, 1.0],
            0.01,
            Box::new( move |ref settings| {
                settings.params[&param_copy_1[..]]
            }),
            Box::new( move |ref mut settings, value| {
                settings.params.insert(param_copy_2.clone(), value);
            }),
        ));
    }

    // set head look controller params to nice initial values..
    settings.params.insert("head-look-level".to_string(), 1.0);
    settings.params.insert("head-look-sideways-level".to_string(), 1.0);
    settings.params.insert("head-down-to-up".to_string(), 0.5);
    settings.params.insert("head-left-to-right".to_string(), 0.5);

    for e in piston_window {

        orbit_zoom_camera.event(&e);
        menu.event(&e, &mut settings);

        e.resize(|width, height| {
            // Update projection matrix
            projection = CameraPerspective {
                fov: 90.0f32,
                near_clip: 0.1,
                far_clip: 1000.0,
                aspect_ratio: (width as f32) / (height as f32)
            }.projection();
        });

        e.update(|args| {
            dlb_demo.update(&settings, args.dt);
            lbs_demo.update(&settings, args.dt);
        });

        e.draw_3d(|stream| {

            use gfx::traits::Stream;

            let args = e.render_args().unwrap();

            stream.clear(gfx::ClearData {
                color: [0.3, 0.3, 0.3, 1.0],
                depth: 1.0,
                stencil: 0,
            });

            let camera_view = orbit_zoom_camera.camera(args.ext_dt).orthogonal();

            let camera_projection = model_view_projection(
                model,
                camera_view,
                projection
            );

            // Draw axes
            debug_renderer.draw_line([0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 1.0, 0.0, 1.0]);
            debug_renderer.draw_line([0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [0.0, 0.0, 1.0, 1.0]);

            debug_renderer.draw_text_at_position(
                "X",
                [6.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0],
            );

            debug_renderer.draw_text_at_position(
                "Y",
                [0.0, 6.0, 0.0],
                [0.0, 1.0, 0.0, 1.0],
            );

            debug_renderer.draw_text_at_position(
                "Z",
                [0.0, 0.0, 6.0],
                [0.0, 0.0, 1.0, 1.0],
            );

            dlb_demo.render(&settings, &mut debug_renderer, stream, camera_view, camera_projection, args.ext_dt, settings.use_dlb);

            lbs_demo.render(&settings, &mut debug_renderer, stream, camera_view, camera_projection, args.ext_dt, !settings.use_dlb);

            menu.draw(&settings, &mut debug_renderer);

            if let Err(e) = debug_renderer.render(stream, camera_projection) {
                println!("{:?}", e);
            }
        });
    }
}
