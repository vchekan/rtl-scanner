use druid::{AppLauncher, WindowDesc, Widget, Data, Lens, PlatformError, WidgetExt, ExtEventSink, Selector, Target, AppDelegate, Handled, Env, Command, DelegateCtx, Event, UpdateCtx, LifeCycle, EventCtx, PaintCtx, BoxConstraints, LifeCycleCtx, Size, LayoutCtx, LensExt};
use druid::widget::{Label, Flex, ProgressBar, Tabs, TabsTransition, RadioGroup, LabelText, CrossAxisAlignment, Spinner, Stepper, TextBox};
use std::thread;
use std::time::Duration;
use crate::Device;
use log::debug;
use druid::text::RichText;
use druid::lens::Map;

pub fn main() -> Result<(), PlatformError> {
    let mut launcher = AppLauncher::with_window(
        WindowDesc::new(build_ui).title("RTL Scanner"))
        .delegate(EventDelegate {});

    let handler = launcher.get_external_handle();
    device_discovery_loop(handler);

    launcher.launch(State::default())?;
    Ok(())
}

const LIST_DEVICES: Selector<Vec<Device>> = Selector::new("rtl_scanner.list_devices");

#[derive(Clone, Data, Lens)]
struct State {
    ready_status: ReadyStatus,
    devices: DeviceList,
    device_idx: usize,
    progress: f64,
    scan_from: f64,
    scan_to: f64,
    dwell: u32,
}

impl Default for State {
    fn default() -> Self { State {
        ready_status: ReadyStatus::Initializing,
        devices: DeviceList::default(),
        device_idx: 0,
        progress: 0.0,
        scan_from: 100.0,
        scan_to: 3000.0,
        dwell: 16,
    }}
}

#[derive(Debug, Clone, PartialEq, Data)]
enum ReadyStatus {
    Initializing,
    Ready,
    Running,
}

impl Default for ReadyStatus {
    fn default() -> Self { ReadyStatus::Initializing }
}


fn build_ui() -> impl Widget<State> {
    main_panel()
        //.debug_invalidation()
        //.debug_paint_layout().debug_widget_id()
}

fn main_panel() -> impl Widget<State> {
    Flex::column()
        .with_flex_child(tabs_panel(), 1.0)
        .with_child(status_bar())
}

fn tabs_panel() -> impl Widget<State> {
    Tabs::new()
        .with_tab("Scan", scan_tab())
        .with_tab("Settings", settings_tab())
        .with_transition(TabsTransition::Instant)
}

fn scan_tab() -> impl Widget<State> {
    Label::new("Scan tab")
}

fn settings_tab() -> impl Widget<State> {
    let drg = DynRadioGroup{ radio_buttons: None};
    let drg = drg.lens(State::devices);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Flex::row()
            //
            // Scan range
            //
            .with_child(Label::new("Scan range"))
            .with_spacer(10.0)
            .with_child(TextBox::new()
                .lens(State::scan_from.map(|&freq| format!("{:.1}", freq), |freq: &mut f64, s: String| *freq = s.parse().unwrap_or(100.0)))
            ).with_child(Stepper::new().with_range(100.0, 3000.0).with_step(0.1).lens(State::scan_from))
            .with_spacer(20.0)
            .with_child(TextBox::new()
                .lens(State::scan_to.map(|&freq| format!("{:.1}", freq), |freq: &mut f64, s: String| *freq = s.parse().unwrap_or(100.0))))
            .with_child(Stepper::new().with_range(100.0, 3000.0).with_step(0.1).lens(State::scan_to))
        )
        //
        // Dwell
        //
        .with_spacer(10.0)
        .with_child(Flex::row()
            .with_child(Label::new("Dwell (ms)"))
                .with_spacer(10.0)
                .with_child(TextBox::new()
                    .lens(State::dwell.map(|&d| format!("{}", d), |f, s: String| *f = s.parse().unwrap_or(16))))
        )
        //
        // Devices
        //
        .with_spacer(10.0)
        .with_child(drg)
        .with_child(device_details().lens(State::devices))
}

fn status_bar() -> impl Widget<State> {
    Flex::row()
        .with_child(
            Label::dynamic(|state: &ReadyStatus, _env| format!("{:?}", state))
                .lens(State::ready_status)
        )
        .with_flex_child(ProgressBar::new().lens(State::progress), 1.0)
}

fn device_details() -> impl Widget<DeviceList> {
    Flex::column()
        .with_child(Label::new("Device properties"))
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(Label::new(|data: &Device, _env: &_| format!("Name: {}", data.name)))
        .with_child(Label::new(|data: &Device, _env: &_| format!("Product: {}", data.product)))
        .with_child(Label::new(|data: &Device, _env: &_| format!("Manufacturer: {}", data.manufacturer)))
        .with_child(Label::new(|data: &Device, _env: &_| format!("Serial: {}", data.serial)))
        //.lens(lens!(Device))
        .lens(SelectedDeviceLens)
}

fn device_discovery_loop(handler: ExtEventSink) {
    // TODO: fail if thread is failed?
    thread::Builder::new().name("device discovery".to_string()).spawn(move || {
        let mut last_count = 0;
        loop {
            let devices = crate::list_devices();
            if devices.len() != last_count {
                last_count = devices.len();
                // TODO: cam move payload, instead of sending ref and then clone
                // handler.submit_command(LIST_DEVICES.with(SingleUse::new(devices)))
                handler.submit_command(LIST_DEVICES, devices, Target::Auto)
                    .expect("device_discovery_loop: failed to send command to the main thread");
            }
            thread::sleep(Duration::from_secs(1));
        }
    });
}

struct EventDelegate;
impl AppDelegate<State> for EventDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut State,
        _env: &Env,
    ) -> Handled {
        //if cmd.is(LIST_DEVICES) {
          //  data.devices = cmd.get_unchecked(LIST_DEVICES).clone();
        if let Some(devices) = cmd.get(LIST_DEVICES) {
            // TODO: preserve selected by value
            data.devices = DeviceList{devices: devices.clone(), selected: 0};
            return Handled::Yes
        }
        Handled::No
    }
}

#[derive(Debug, Default, Clone, Data, Lens)]
struct DeviceList {
    #[data(same_fn="device_vec_eq")]
    pub devices: Vec<Device>,
    selected: usize,
}

struct SelectedDeviceLens;
impl Lens<DeviceList, Device> for SelectedDeviceLens {
    fn with<V, F: FnOnce(&Device) -> V>(&self, data: &DeviceList, f: F) -> V {
        if !data.devices.is_empty() {
            let device = data.devices.get(data.selected).unwrap();
            f(device)
        } else {
            f(&Device {name: "-".to_string(), manufacturer: "-".to_string(), product: "-".to_string(), serial: "-".to_string()})
        }
    }

    fn with_mut<V, F: FnOnce(&mut Device) -> V>(&self, data: &mut DeviceList, f: F) -> V {
        if !data.devices.is_empty() {
            let device = data.devices.get_mut(data.selected).unwrap();
            f(device)
        } else {
            f(&mut Device {name: "-".to_string(), manufacturer: "-".to_string(), product: "-".to_string(), serial: "-".to_string()})
        }
    }
}

fn device_vec_eq(a: &Vec<Device>, b: &Vec<Device>) -> bool {
    // It is unlikely that device is removed and another one is inserted instantly.
    // So we can detect device changes by detecting change in device count.
    a.len() == b.len()
}

struct DynRadioGroup {
    radio_buttons: Option<Box<dyn Widget<DeviceList>>>,
}

impl Widget<DeviceList> for DynRadioGroup {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut DeviceList, env: &Env) {
        if let Some(child) = &mut self.radio_buttons {
            child.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &DeviceList, env: &Env) {
        if let Some(child) = &mut self.radio_buttons {
            child.lifecycle(ctx, event, data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &DeviceList, data: &DeviceList, env: &Env) {
        if old_data.devices.len() != data.devices.len() {
            let group: Vec<(String, usize)> = data.devices.iter().enumerate()
                .map(|(i, device)| (device.name.clone(), i))
                .collect();

            let group = RadioGroup::new(group)
                .lens(DeviceList::selected);


            self.radio_buttons = Some(Box::new(group));
            ctx.children_changed();
            ctx.request_layout();
        } else if old_data.selected != data.selected {
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &DeviceList, env: &Env) -> Size {
        if let Some(child) = &mut self.radio_buttons {
            child.layout(ctx, bc, data, env)
        } else {
            Size::new(0.0, 0.0)
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &DeviceList, env: &Env) {
        if let Some(child) = &mut self.radio_buttons {
            child.paint(ctx, data, env);
        }
    }
}
