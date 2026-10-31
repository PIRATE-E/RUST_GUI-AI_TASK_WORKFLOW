slint::include_modules!();
use std::rc::Rc;
use std::time::Duration;

use ai_workflow_gui::init_sync_send_all;
use ai_workflow_gui::load_python::gc_connect;
use ai_workflow_gui::log_entry_handler::{attach_display_into_handler, attach_handle_into_handler};
use ai_workflow_gui::slint_py_engine::SlintRuntime;
use slint::{ComponentHandle, Model, ModelRc};
use slint::{ToSharedString, VecModel};

use time_format::{now, strftime_local};

use pyo3::prelude::*;
use std::sync::mpsc::{self, Receiver, SyncSender};
// state we have to maintain !! between main gui thread
// and worker thread which holds slint runtime python objects
// to make worker thread to submit ticket in we have to hand over
// state worker thread than proceed upon that stage !! whether tjcket in or destroy !!
enum WorkerState {
    CurrentUserMsg(String),
    Destroy,
}

fn main() {
    let app = AppWindow::new().unwrap();

    // our app main window state !! currently at booting to make the
    // python worker to spawn the python runtime !!
    app.set_app_state(AppState::Booting);
    let appwekref = app.as_weak();
    // we have to grab the listeners and the properties before the app got run !!

    let message_stack = Rc::new(VecModel::<ChatMessage>::default());

    // because only 1 message at a time ! maintain idempotency
    let (state_sender, state_receiver) = mpsc::sync_channel::<WorkerState>(1);

    let boot_weak_ref = appwekref.clone();

    // now the worker thread spawned ~!

    std::thread::spawn(move || {
        // python woker thread -- init python -- recieve user message -- process ticket return
        // ticker out result !!
        //

        let log_receiver = init_sync_send_all();

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
                        WorkerState::CurrentUserMsg(prompt) => {
                            // user message recieved
                            Python::attach(|py| {
                                match slint_runtime_obj.process_ticket(py, &prompt, 60) {
                                    Ok(ticket_out) => {
                                        let reply = ticket_out.transcript.as_str();
                                        slint::invoke_from_event_loop(move || {
                                            if let Some(app_ui) = ui_weak.upgrade() {
                                                // Push AI ChatMessage to message_stack
                                                let chat_meesage: ChatMessage = ChatMessage {
                                                    is_user: false,
                                                    sender: "AI".into(),
                                                    text: ticket_out.transcript.into(),
                                                    timestamp: ticket_out
                                                        .latency_ms
                                                        .to_string()
                                                        .into(),
                                                };
                                                if let Some(vec_model) = app_ui
                                                    .get_messages()
                                                    .as_any()
                                                    .downcast_ref::<VecModel<ChatMessage>>()
                                                {
                                                    vec_model.push(chat_meesage);
                                                }
                                            }
                                        })
                                        .unwrap();
                                    }
                                    Err(err) => {
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

    // phase 2 of sending user message to worker threa !
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
    //
    //send message to the worker thread
    //

    app.on_send_message(move || {
        if let Some(app) = appwekref.upgrade() {
            let text = app.get_current_input();

            let time = now().unwrap();
            let time =
                strftime_local("%m-%d-%Y || %H-%M-%S || %p", time).unwrap_or("00err".to_string());

            // user message send to worker thread
            _ = state_sender.send(WorkerState::CurrentUserMsg(String::from(text.clone())));

            if !text.is_empty() {
                message_stack.push(ChatMessage {
                    is_user: true,
                    sender: "user".to_shared_string(),
                    text,
                    timestamp: time.to_shared_string(),
                });
                app.set_current_input("".to_shared_string());
                app.set_messages(ModelRc::from(message_stack.clone()));
            }
        };
    });

    app.run().unwrap();
}
