use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Stack, Button, GestureClick, EventControllerScroll, EventControllerScrollFlags};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc; // Internal channel only
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use pulsectl::controllers::{SinkController, DeviceControl}; // Trait for getting volume
use libpulse_binding as pulse;
use pulse::mainloop::standard::Mainloop;
use pulse::context::{Context, FlagSet as ContextFlagSet};
use pulse::context::subscribe::InterestMaskSet;
use pulse::volume::Volume;

pub fn build(scroll_speed: f64) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 0);
    container.add_css_class("volume-module");

    // UI Components
    let stack = Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::None);

    let prefix_page = Label::new(Some("VOL"));
    prefix_page.set_xalign(0.5);
    stack.add_child(&prefix_page);

    let controls_page = GtkBox::new(Orientation::Horizontal, 0);
    controls_page.add_css_class("controls");
    controls_page.set_halign(gtk4::Align::Center);
    let btn_down = Button::with_label("<");
    let btn_up = Button::with_label(">");
    controls_page.append(&btn_down);
    controls_page.append(&btn_up);
    stack.add_child(&controls_page);
    
    let stack_page = stack.page(&prefix_page);
    stack_page.set_name("label");
    let stack_page_controls = stack.page(&controls_page);
    stack_page_controls.set_name("controls");
    
    stack.set_visible_child_name("label");

    let val_label = Label::new(Some("[---]"));

    container.append(&stack);
    container.append(&val_label);

    // --- State Management ---
    let timeout_source = Rc::new(RefCell::new(None::<gtk4::glib::SourceId>));
    
    let stack_weak = stack.downgrade();
    let timeout_source_clone = timeout_source.clone();
    
    let reset_timer = Rc::new(move || {
        let mut source = timeout_source_clone.borrow_mut();
        if let Some(id) = source.take() {
            id.remove();
        }
        
        let stack_weak = stack_weak.clone();
        let timeout_source_inner = timeout_source_clone.clone();
        
        let new_id = gtk4::glib::timeout_add_seconds_local(3, move || {
            if let Some(stack) = stack_weak.upgrade() {
                stack.set_visible_child_name("label");
            }
            *timeout_source_inner.borrow_mut() = None;
            gtk4::glib::ControlFlow::Break
        });
        
        *source = Some(new_id);
    });

    let local_vol = Rc::new(std::cell::Cell::new(0.0f64));

    let val_label_clone = val_label.clone();
    let local_vol_clone_ui = local_vol.clone();
    let update_ui = Rc::new(move |vol: Option<f64>| {
        if let Some(v) = vol {
            local_vol_clone_ui.set(v);
            let v_display = (v * 100.0).round() as u32;
            val_label_clone.set_text(&format!("[{:03}]", v_display));
        } else {
            val_label_clone.set_text("[ERR]");
        }
    });

    let (set_vol_tx, set_vol_rx) = mpsc::channel::<f64>();

    if let Ok(mut c) = SinkController::create() {
        if let Ok(dev) = c.get_default_device() {
            update_ui(Some(dev.volume.avg().0 as f64 / Volume::NORMAL.0 as f64));
        }
    }

    let adjust_vol = {
        let local_vol = local_vol.clone();
        let update_ui = update_ui.clone();
        let set_vol_tx = set_vol_tx.clone();
        move |delta: f64| {
            let new_vol = (local_vol.get() + delta).clamp(0.0, 1.0);
            update_ui(Some(new_vol));
            let _ = set_vol_tx.send(new_vol);
        }
    };

    // --- Interaction Handlers ---

    let click_gesture = GestureClick::new();
    let stack_clone_for_click = stack.clone();
    let reset_timer_click = reset_timer.clone();
    
    click_gesture.connect_pressed(move |_, _, _, _| {
        if stack_clone_for_click.visible_child_name().as_deref() == Some("label") {
            stack_clone_for_click.set_visible_child_name("controls");
            reset_timer_click();
        }
    });
    prefix_page.add_controller(click_gesture);
    
    let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
    let adjust_vol_scroll = adjust_vol.clone();
    
    scroll_controller.connect_scroll(move |_, _, dy| {
        let speed_fraction = scroll_speed / 100.0;
        let step = -dy * speed_fraction;
        adjust_vol_scroll(step);
        gtk4::glib::Propagation::Stop
    });
    container.add_controller(scroll_controller);

    let adjust_vol_btn_up = adjust_vol.clone();
    let reset_timer_btn_up = reset_timer.clone();
    
    btn_up.connect_clicked(move |_| {
        adjust_vol_btn_up(0.05);
        reset_timer_btn_up();
    });

    let adjust_vol_btn_down = adjust_vol.clone();
    let reset_timer_btn_down = reset_timer.clone();
    
    btn_down.connect_clicked(move |_| {
        adjust_vol_btn_down(-0.05);
        reset_timer_btn_down();
    });
    
    // --- Message Channel for Background Thread ---
    let (sender, receiver) = async_channel::unbounded();
    
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_thread = stop_flag.clone();

    std::thread::spawn(move || {
        let mut controller = match SinkController::create() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to create PA controller in thread: {}", e);
                return;
            }
        };

        if let Ok(default_device) = controller.get_default_device() {
             let v = default_device.volume.avg().0 as f64 / Volume::NORMAL.0 as f64;
             let _ = sender.send_blocking(Some(v));
        }

        let mut mainloop = match Mainloop::new() {
            Some(m) => m,
            None => { eprintln!("Failed to create mainloop"); return; }
        };
        
        let mut context = match Context::new(&mainloop, "Anomale Volume Listener") {
            Some(c) => c,
            None => { eprintln!("Failed to create context"); return; }
        };

        if context.connect(None, ContextFlagSet::NOFLAGS, None).is_err() {
            eprintln!("Failed to connect context");
            return;
        }

        loop {
            match mainloop.iterate(false) {
                pulse::mainloop::standard::IterateResult::Success(_) => {},
                _ => { eprintln!("Mainloop iterate failed"); return; }
            }
            match context.get_state() {
                pulse::context::State::Ready => break,
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    eprintln!("Context connection failed");
                    return;
                },
                _ => {}
            }
        }

        let (tx, rx) = mpsc::channel();
        
        context.set_subscribe_callback(Some(Box::new(move |_, _, _| {
            let _ = tx.send(());
        })));
        
        context.subscribe(InterestMaskSet::SINK | InterestMaskSet::SERVER, |success| {
            if !success { eprintln!("Subscribe failed"); }
        });

        loop {
            if stop_flag_thread.load(Ordering::Relaxed) {
                break;
            }
            
            let mut pending_set: Option<f64> = None;
            while let Ok(new_vol) = set_vol_rx.try_recv() {
                pending_set = Some(new_vol);
            }
            
            if let Some(target_vol) = pending_set {
                if let Ok(mut device) = controller.get_default_device() {
                    let new_vol_pa = Volume((target_vol * Volume::NORMAL.0 as f64) as u32);
                    let channels = device.volume.len();
                    device.volume.set(channels, new_vol_pa);
                    let _ = controller.set_device_volume_by_index(device.index, &device.volume);
                }
            }

            match mainloop.iterate(false) {
                pulse::mainloop::standard::IterateResult::Success(_) => {},
                _ => break,
            }
            
            if let Ok(_) = rx.try_recv() {
                 while let Ok(_) = rx.try_recv() {}
                 
                 if let Ok(default_device) = controller.get_default_device() {
                     let v = default_device.volume.avg().0 as f64 / Volume::NORMAL.0 as f64;
                     let _ = sender.send_blocking(Some(v));
                 }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
    });

    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Ok(vol) = receiver.recv().await {
            update_ui(vol);
        }
    });

    container.connect_destroy(move |_| {
        stop_flag.store(true, Ordering::Relaxed);
    });

    container
}
