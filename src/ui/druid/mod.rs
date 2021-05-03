use druid::{AppLauncher, WindowDesc, Widget, Data, Lens, PlatformError, WidgetExt, ExtEventSink, Selector, Target, AppDelegate, Handled, Env, Command, DelegateCtx, WindowId, Event};
use druid::widget::{Label, Flex, Checkbox, List, ProgressBar, CrossAxisAlignment, SizedBox, ViewSwitcher, Tabs, TabsTransition, RadioGroup};
use std::borrow::BorrowMut;
use druid_widget_nursery::Dropdown;
use std::thread;
use std::time::Duration;
use crate::Device;
use std::sync::Arc;

pub fn main() -> Result<(), PlatformError> {
    let mut launcher = AppLauncher::with_window(
        WindowDesc::new(build_ui)
            .title("RTL Scanner"))
        .delegate(EventDelegate {});

    let handler = launcher.get_external_handle();
    device_discovery_loop(handler);

    launcher.launch(State::default())?;
    Ok(())
}

const LIST_DEVICES: Selector<Arc<Vec<Device>>> = Selector::new("rtl_scanner.list_devices");

#[derive(Clone, Default, Data, Lens)]
struct State {
    ready_status: ReadyStatus,
    devices: Arc<Vec<Device>>,
    device_idx: usize,
    progress: f64,
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
        // .debug_paint_layout()
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
    let devices = RadioGroup::new(vec![]);
    Flex::column()
        .with_child(Label::new("Settings tab"))
        .with_child(RadioGroup::new(
            
        ))
        //.with_child(Checkbox::)

}

fn status_bar() -> impl Widget<State> {
    Flex::row()
        // .cross_axis_alignment(CrossAxisAlignment::End)
        .with_child(
            Label::dynamic(|state: &ReadyStatus, _env| format!("{:?}", state))
                .lens(State::ready_status)
        )
        .with_flex_child(ProgressBar::new().lens(State::progress), 1.0)
        // .fix_height(100.0)
}

fn device_discovery_loop(handler: ExtEventSink) {
    // TODO: fail if thread is failed?
    thread::Builder::new().name("device discovery".to_string()).spawn(move || {
        let devices = Arc::new(crate::list_devices());
        handler.submit_command(LIST_DEVICES, devices, Target::Auto).expect("device_discovery_loop: failed to send command to the main thread");
        thread::sleep(Duration::from_secs(5))
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
        if cmd.is(LIST_DEVICES) {
            data.devices = cmd.get_unchecked(LIST_DEVICES).clone();
            return Handled::Yes
        }
        Handled::No
    }
}