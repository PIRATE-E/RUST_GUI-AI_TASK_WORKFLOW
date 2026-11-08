slint::include_modules!();
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ai_workflow_gui::load_python::gc_connect;
use ai_workflow_gui::log_entry_handler::{attach_display_into_handler, attach_handle_into_handler};
use ai_workflow_gui::slint_py_engine::SlintRuntime;
use ai_workflow_gui::{LogEntry, TicketUiState, init_sync_send_all};
use slint::{ComponentHandle, Model, ModelRc, SharedString};
use slint::{ToSharedString, VecModel};

use time_format::{now, strftime_local};

use pyo3::prelude::*;
use std::sync::mpsc::{self};
use uuid::Uuid;
// state we have to maintain !! between main gui thread
// and worker thread which holds slint runtime python objects
// to make worker thread to submit ticket in we have to hand over
// state worker thread than proceed upon that stage !! whether tjcket in or destroy !!
#[allow(unused)]
enum WorkerState {
    CurrentUserMsg {
        prompt: String,
        ticket_id: String,
        turn_counter: u32,
    },
    Destroy,
}

// this is DrawerLogItem implementation in rust which is requiired to show the logs we fetched
// to the drawer !! because slint except only primitives we have to go with serialize the
// LogEntry into Whole different object and then store that object into the buffer to show on
// when user clicked on (>) button !!

// DrawerLogItem conversion using Slint codegen struct
impl From<&LogEntry> for DrawerLogItem {
    fn from(log: &LogEntry) -> Self {
        Self {
            timestamp: log.timestamp.as_str().into(),
            category: log.log_type.as_str().into(),
            level: log.log_level.as_str().into(),
            message: log.message.as_str().into(),
            meta_preview: match &log.metadata {
                Some(m) => slint::format!("{m}"),
                None => SharedString::default(),
            },
        }
    }
}

fn main() {
    let app = AppWindow::new().unwrap();

    // our app main window state !! currently at booting to make the
    // python worker to spawn the python runtime !!
    app.set_app_state(AppState::Booting);
    let appwekref = app.as_weak();
    // we have to grab the listeners and the properties before the app got run !!

    // this is rc model but required for both purpose to push the chat message or grab the chat
    // bubble and compare is it generating ??
    let message_stack = Rc::new(VecModel::<ChatMessage>::default());

    // because only 1 message at a time ! maintain idempotency
    let (state_sender, state_receiver) = mpsc::sync_channel::<WorkerState>(1);

    let boot_weak_ref = appwekref.clone();
    let telementry_weak_ref = appwekref.clone();
    let drawer_expand_weak_ref = appwekref.clone();

    // set up log receiver for thread 3

    let log_receiver = init_sync_send_all();

    // set up turn counter for recorded how many tickets got sended over this session !!
    // currently it not considered that how we would manage it if we expanded to multi session
    // chats!

    let mut turn_counter = 0u32;

    // drawer logs model for rust and slint to share the filled logs when user clicked on
    // expand if the ticket is currently in processing (is generating = true)
    // because unlike message_stack was purely push or modified
    // from slint or invoke_from_event_loop (which is thread 0)
    // drawer_logs_buffer of rust have to share between slint thread 0 (which is cloning those logs
    // into actual log bubble on ui)
    // and the thread 3 (which is pushing those logs) we do have to define
    // 2 rust and slint models for rust this is arc mutex vector because it have to travel between
    // thread and allow sync instead of coming back to thead 0 and modified
    // this approach makes thread 0 just to render ui instead thread 3 to push those logs while the
    // drawer is closed

    // rust model for thread 3 (belongs to thread 3)
    let drawer_log_items_buffer = Arc::new(Mutex::new(Vec::<DrawerLogItem>::new()));
    // rust model cloned for thread 1 so it can be cloned and transformed into the
    // drawer_logs_model (slint model)
    let drawer_log_items_ready = drawer_log_items_buffer.clone();
    // rust model cloned for thread 0 when user clicked on send message so it can be cleared the buffer
    // to process fresh logs and store them into brand new buffer !!
    let drawer_log_items_clear = drawer_log_items_buffer.clone();

    // slint model which (is binded to the viewport ModelRc(via set_active_ticket_logs )) gonna filled on when user clicked on expand drawer !!
    let drawer_logs_model = Rc::new(VecModel::<DrawerLogItem>::default());

    // cloned drawer_logs_model for make it clear !! when user send next message
    let drawer_logs_model_clear = drawer_logs_model.clone();

    // now the worker thread 2 spawned ~!

    std::thread::spawn(move || {
        // python woker thread -- init python -- recieve user message -- process ticket return
        // ticker out result !!
        //

        // boot up runtime !!

        let boot_run_res = Python::attach(|py| -> PyResult<SlintRuntime> {
            gc_connect(py)?;
            attach_handle_into_handler(py)?;
            attach_display_into_handler(py)?;

            SlintRuntime::boot(py)
        });

        // artificial sleep to observe !!
        std::thread::sleep(Duration::from_secs(2));

        match boot_run_res {
            Ok(mut slint_runtime_obj) => {
                // success we got the global runtime objects after core boot

                let ui_weak = boot_weak_ref.clone();
                slint::invoke_from_event_loop(move || {
                    // make the splash screen go away with success state !!
                    if let Some(app_ui) = ui_weak.upgrade() {
                        app_ui.set_app_state(AppState::Ready);
                    }
                })
                .unwrap();

                // now the chat loop goes from here !!

                while let Ok(worker_state) = state_receiver.recv() {
                    let ui_weak = boot_weak_ref.clone();

                    match worker_state {
                        WorkerState::CurrentUserMsg {
                            prompt,
                            ticket_id,
                            turn_counter,
                        } => {
                            // user message recieved
                            Python::attach(|py| {
                                // send the ticket to the core and grab the ticket out !
                                match slint_runtime_obj.process_ticket(
                                    py,
                                    &prompt,
                                    &ticket_id,
                                    turn_counter,
                                    60,
                                ) {
                                    Ok(ticket_out) => {
                                        slint::invoke_from_event_loop(move || {
                                            if let Some(app_ui) = ui_weak.upgrade() {
                                                // in place mutate the dummy AI ChatMessage to message_stack to the final AI ChatMessage
                                                // grab the VecModel Rc

                                                // this is not vec model rc but we have only need in
                                                // place modify the ai bubble message which we can
                                                // done from abstracted trait object to !!
                                                let chat_model_rc = app_ui.get_messages();
                                                let last_ai_row_idx = chat_model_rc.row_count() - 1;
                                                let mut last_ai_msg = chat_model_rc
                                                    .row_data(last_ai_row_idx)
                                                    .expect("in-place modification failed of last AI-ChatMessage");

                                                // last message overwrite details
                                                last_ai_msg.is_generating = false;
                                                last_ai_msg.ticket_id = ticket_out.ticket_id.to_shared_string();
                                                last_ai_msg.text = ticket_out.transcript.to_shared_string();
                                                last_ai_msg.task_progress = ticket_out.status.to_shared_string();
                                                last_ai_msg.live_summary = ticket_out.status.to_shared_string();
                                                last_ai_msg.timestamp = format!("{:.0}ms", ticket_out.latency_ms).to_shared_string();
                                                last_ai_msg.rpm_count = 0; // Reset active RPM on completion
                                                last_ai_msg.api_count = ticket_out.total_requests as i32;

                                                chat_model_rc.set_row_data(last_ai_row_idx, last_ai_msg.clone());

                                                // Transition open drawer from Amber to Emerald on completion!
                                                if app_ui.get_drawer_open() {
                                                    app_ui.set_active_ticket(last_ai_msg);
                                                }
                                            }
                                        })
                                        .unwrap();
                                    }
                                    Err(_err) => {
                                        // Push error ChatMessage to message_stack
                                    }
                                }
                            });
                        }
                        WorkerState::Destroy => {
                            // destroy the app !!
                        }
                    }
                }
            }
            Err(err) => {
                // failed to boot and grab runtime objects !!

                let ui_weak = boot_weak_ref.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(app_ui) = ui_weak.upgrade() {
                        app_ui.set_app_state(AppState::Failed);
                        app_ui.set_boot_error(err.to_string().into());
                    }
                })
                .unwrap();
            }
        }
    });

    // now thread 3 spawned !!
    // which handles the draining of the logs and parse and draw the ai message summary
    // live telementry components on the last ai message bubble if it is generating !!

    std::thread::spawn(move || {
        // WHAT: Telemetry accumulator initialized with zero-arg constructor.
        // WHY: Autonomous state machine. Automatically populates ticket_id and resets metrics when TICKET • START arrives.
        // ANDROID/PYTHON BRIDGE: Equivalent to initializing a Telemetry ViewModel or DesktopStatusHandler().
        let mut state = TicketUiState::new();
        while let Ok(log) = log_receiver.recv() {
            // 1. Ingest into private accumulator
            state.parse_log(&log);

            // 2 now we have to append that log into the in memory log buffer so the logs could be
            // showed in the drawer !! if drawer was closed !!
            let drawer_log_item = DrawerLogItem::from(&log);
            // push into the rust buffer of drawer log items !!
            drawer_log_items_buffer
                .lock()
                .unwrap()
                .push(drawer_log_item.clone());

            // 3. Extract snapshot values to update the AI bubble
            let task_progress = state.task_progress();
            let rpm_count = state.rpm_count;
            let total_requests = state.total_requests;
            let fail_count = state.failed_count;
            let headline = state.current_headline.clone();

            // 4. Dispatch to Thread 0 without blocking
            let ui_weak = telementry_weak_ref.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(app_ui) = ui_weak.upgrade() {
                    let chat_model = app_ui.get_messages();
                    let count = chat_model.row_count();
                    if count > 0 {
                        let last_idx = count - 1;
                        if let Some(mut last_ai_chat_msg) = chat_model.row_data(last_idx) {
                            // Only mutate active in-flight generating bubbles!
                            // WHAT: Updates telemetry metrics ONLY. Never touches or overwrites bubble ticket_id!
                            // WHY: Thread 0 already assigned canonical ticket_id. Eliminates split-brain identity bugs.
                            // ANDROID/PYTHON BRIDGE: Equivalent to updating ViewHolder progress bar without changing item ID.
                            if last_ai_chat_msg.is_generating {
                                last_ai_chat_msg.task_progress = task_progress.into();
                                last_ai_chat_msg.rpm_count = rpm_count;
                                last_ai_chat_msg.api_count = total_requests;
                                last_ai_chat_msg.fail_count = fail_count;
                                last_ai_chat_msg.live_summary = headline.into();
                                // 1. Update the AI bubble in the chat list if generating
                                chat_model.set_row_data(last_idx, last_ai_chat_msg.clone());

                                // 2. LIVE DRAWER SYNC: If drawer is open on this active turn, update it at 60 FPS!
                                if app_ui.get_drawer_open()
                                    && app_ui.get_active_ticket().is_generating
                                {
                                    app_ui.set_active_ticket(last_ai_chat_msg);

                                    // if the drawer is open we have to show the logs live !!
                                    // get the rc model to mutate directly and push the logs into it
                                    // while live of the drawer is opened ~!
                                    // this is dyn trait object ListView we have to grab the binded
                                    // rc model instead
                                    let drawer_log_dyn_model_live = app_ui.get_active_ticket_logs();
                                    // grab the rc model here !
                                    if let Some(rc_model_drawer_log_item) =
                                        drawer_log_dyn_model_live
                                            .as_any()
                                            .downcast_ref::<VecModel<DrawerLogItem>>()
                                    {
                                        rc_model_drawer_log_item.push(drawer_log_item);
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .unwrap();
        }
    });

    // phase 2 of sending user message to worker thread !
    // grab the already filled message stack from the slint !!
    // as proto type to load chats from disk
    {
        let count = app.get_messages().row_count();
        for i in 0..count {
            if let Some(data) = app.get_messages().row_data(i) {
                message_stack.push(data);
            }
        }
    }

    // bind the modelrc reference to the slint set messages so now !! the messages would auto track
    // the model because it got bind to its reference !!
    // doing this after reading from the disk !! so all auto loaded rows in the model would be
    // showed on very start up of the app !!

    app.set_messages(ModelRc::from(message_stack.clone()));

    // send button on user text box !!
    //send message to the worker thread
    //

    app.on_send_message(move || {
        if let Some(app) = appwekref.upgrade() {
            // clear the logs of from buffer of drawer_log_item because we have to
            // now populates new logs of new ticket !!
            drawer_log_items_clear.lock().unwrap().clear();
            drawer_logs_model_clear.set_vec(Vec::default());

            let text = app.get_current_input();

            let time = now().unwrap();
            let time =
                strftime_local("%m-%d-%Y || %H-%M-%S || %p", time).unwrap_or("00err".to_string());

            turn_counter += 1; // user message got send it means new ticket should be issued
            let ticket_id = format!("development-{}", Uuid::new_v4());

            // user message send to worker thread
            _ = state_sender.send(WorkerState::CurrentUserMsg {
                prompt: String::from(text.clone()),
                ticket_id: ticket_id.clone(),
                turn_counter,
            });

            if !text.is_empty() {
                // render the user message bubble !!
                message_stack.push(ChatMessage {
                    is_user: true,
                    sender: "user".to_shared_string(),
                    text,
                    timestamp: time.to_shared_string(),
                    ..Default::default()
                });
                // clean the text box input area !!
                app.set_current_input("".to_shared_string());
                // render the AI placeholder message bubble !!

                // Inside app.on_send_message (after pushing UserBubble):
                message_stack.push(ChatMessage {
                    ticket_id: ticket_id.clone().into(),
                    is_user: false,
                    sender: "ColdWind AI".into(),
                    text: "".into(), // Ignored while is_generating == true
                    timestamp: "Running...".into(),
                    is_generating: true,
                    task_progress: "Task [0/0]".into(),
                    rpm_count: 0,
                    api_count: 0,
                    fail_count: 0,
                    live_summary: "Queued • Initializing turn...".into(),
                });
            }
        };
    });

    // bind the drawer_logs_model refrence to slint to the active ui of that slint property able to
    // show the data behind that reference of rc model !
    app.set_active_ticket_logs(ModelRc::from(drawer_logs_model.clone()));

    // expand button clicked to open drawer

    #[allow(unused)]
    app.on_inspect_ticket(move |ticket_id_str| {
        if let Some(app_ui) = drawer_expand_weak_ref.upgrade() {
            // grab which bubble to actually expand on user click on expand button
            let expand_ai_bubble = app_ui.get_active_ticket();

            if expand_ai_bubble.is_generating {
                // grab the current buffer copy its vector into thread 3's stack
                let current_ready_logs = drawer_log_items_ready.lock().unwrap().clone();
                drawer_logs_model.set_vec(current_ready_logs);
            } else {
                // fetch from disk !
                // instead first clear the viewport of the drawer so it can showned empty for while
                // which is valid instead of showing logs of whole another ticket !!
                drawer_logs_model.set_vec(Vec::default());
            }
        }
    });

    app.run().unwrap();
}
