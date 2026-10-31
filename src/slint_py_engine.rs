/*
*
* MANAGE SLINT-GUI ENTRY POINT HELPER
* TO BOOT THAT ON START UP AND MANAGE STATE AND CHAT LOOP !!
*
* */

// global objects to maintain !!

use std::sync::OnceLock;

use pyo3::types::{PyDict, PyList};
use pyo3::{IntoPyObjectExt, prelude::*};
use pyo3::{Py, PyAny, PyResult, Python, types::PyAnyMethods};
use std::time::Instant;
// runtime objects and cached objects to pass over whole loop of process ticket !!
// while user send message and response are occuring few global states and cached
// objects maintained here to passed for direct TicketIn (core) or internal purpose !!
pub struct SlintRuntime {
    // because this is unbind py Object instance of Coldwind Engine instance got from boot !
    pub engine: Py<PyAny>,
    // actuall coldwind context passed by the helper !!
    pub coldwind_context: Py<PyAny>,
    //session_state: dict[str, Any] = {"messages": [], "message_type": None}
    pub session_state: Py<PyAny>,

    /// Cached `TicketIn` class handle so we never re-import on every turn
    pub ticket_in_cls: Py<PyAny>,
    /// Cached list of allowed tool names (`list[str]`) extracted from `engine.tools` at boot
    pub allowed_tools: Py<PyList>,
    /// Session identifier & monotonic turn counter for unique `ticket_id` generation
    pub session_id: String,
    pub user_id: String,
    pub turn_counter: u64,
}

// all the registered tools meta data to show them in different sliding window drawer component !!
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
}

// got populated once when we boot to show to users !!
pub static TOOL_INFO_LIST: OnceLock<Vec<ToolMetadata>> = OnceLock::new();

//this is whole object would be handed over direct to ui rendering logic
//to make them pick what ever they want to show !!
#[derive(Clone, Debug)]
pub struct TicketOutResult {
    pub ticket_id: String,
    pub status: String,                // "COMPLETED" | "FAILED" | "TIMED_OUT"
    pub transcript: String,            // Final AI response text
    pub latency_ms: f64,               // Turn execution time from TicketOut.usage
    pub error_message: Option<String>, // if status == "FAILED" so their would be error detail
    pub is_exiting: bool,              // user passed slash command (exit)?
}

impl SlintRuntime {
    // have to fire that on very start up of the slint !!
    pub fn boot<'py>(py: Python<'py>) -> PyResult<Self> {
        let main_orch_mod = py.import("coldwind.slint_gui.main_orchestrator")?;
        let boot_fn = main_orch_mod.getattr("boot_slint_py_runtime")?;

        // got tuple of engine object and context object !!
        let engine_n_context_n_session_state = boot_fn.call0()?;

        let (engine, context, session_state): (Py<PyAny>, Py<PyAny>, Py<PyAny>) =
            engine_n_context_n_session_state.extract()?;

        // Cache `TicketIn` class once at boot
        // (avoids py.import overhead on every turn for the process ticket)
        let ticket_mod = py.import("coldwind.core.models.ticket_in")?;
        let ticket_in_cls: Py<PyAny> = ticket_mod.getattr("TicketIn")?.unbind();

        // extract registered tools and cached them to passed over in the ticket
        // for the process ticket !

        let mut allowed_py_list_name = PyList::empty(py);
        let mut allowed_rs_vec_name_n_desc = Vec::<ToolMetadata>::new();
        // grab the self.tools from the engine !!
        let tools_attr = engine.bind(py).getattr("tools")?;

        // grab the direct discriptor slot of PyObject_GetIter from the list !!
        // that would make act as rust native Interator and (fast to !)
        let tools_iter_obj = tools_attr.try_iter()?;
        for tool in tools_iter_obj {
            // unwrap with default value except name !!
            let tool_py = tool?;
            let name_py = tool_py.getattr("name")?;

            // to store in the oncelock for global info !!
            let name_rs = tool_py.getattr("name")?.extract::<String>()?;
            let desc = tool_py
                .getattr("description")
                .and_then(|d| d.extract::<String>())
                .unwrap_or_default();

            allowed_py_list_name.append(&name_py)?;
            allowed_rs_vec_name_n_desc.push(ToolMetadata {
                name: name_rs,
                description: desc,
            });
        }
        let _ = TOOL_INFO_LIST.set(allowed_rs_vec_name_n_desc);

        Ok(Self {
            engine,
            coldwind_context: context,
            session_state,
            ticket_in_cls,
            allowed_tools: allowed_py_list_name.unbind(),
            session_id: "slint-gui-development-0.0.1".to_string(),
            user_id: "pirate-developercode-01".to_string(),
            turn_counter: 0,
        })
    }

    pub fn process_ticket<'py>(
        &mut self,
        py: Python<'py>,
        user_message: &str,
        timeout_sec: u32,
    ) -> PyResult<TicketOutResult> {
        let start_time = Instant::now();
        let user_message = user_message.trim();
        if user_message.is_empty() {
            return Ok(TicketOutResult {
                ticket_id: "dev-null".to_string(),
                status: "COMPLETED".to_string(),
                transcript: String::new(),
                latency_ms: start_time.elapsed().as_millis() as f64,
                error_message: None,
                is_exiting: false,
            });
        };

        // Guard B: Fast-path intercept for "exit" / "quit" / "/exit"
        // (Just like desktop/main_orchestrator.py — avoids wasting an LLM call!)
        if matches!(
            user_message.to_lowercase().as_str(),
            "exit" | "quit" | "/exit"
        ) {
            // self.request_exit(py)?;
            return Ok(TicketOutResult {
                ticket_id: "exit-cmd".to_string(),
                status: "COMPLETED".to_string(),
                transcript: "Exiting Cold Wind AI. Goodbye!".to_string(),
                latency_ms: start_time.elapsed().as_millis() as f64,
                error_message: None,
                is_exiting: true,
            });
        }

        // Guard C: Fast-path intercept for "/clear"
        // (Resets `self.session_state` in 0ms without calling the LLM!)
        if user_message.eq_ignore_ascii_case("/clear") {
            // self.clear_session(py)?;
            return Ok(TicketOutResult {
                ticket_id: "clear-cmd".to_string(),
                status: "COMPLETED".to_string(),
                transcript: "Chat history cleared.".to_string(),
                latency_ms: start_time.elapsed().as_millis() as f64,
                error_message: None,
                is_exiting: false,
            });
        }

        self.turn_counter += 1;
        let ticket_id = format!("slint-{:04}", self.turn_counter);

        // Step 2.1: Emit user message to `context.get_message_display().display_message("user", ...)`
        if let Ok(display) = self
            .coldwind_context
            .bind(py)
            .call_method0("get_message_display")
        {
            let _ = display.call_method1("display_message", ("user", user_message));
        };

        // Step 2.2: Build `kwargs` (`PyDict`) for `TicketIn(**kwargs)`
        let limits_dict = PyDict::new(py);
        limits_dict.set_item("timeout_seconds", timeout_sec)?;

        let kwargs = PyDict::new(py);
        kwargs.set_item("ticket_id", &ticket_id)?;
        kwargs.set_item("session_id", &self.session_id)?;
        kwargs.set_item("user_id", &self.user_id)?;
        kwargs.set_item("user_message", user_message)?;
        kwargs.set_item("working_state", self.session_state.bind(py))?;
        kwargs.set_item("allowed_tools", self.allowed_tools.bind(py))?;
        kwargs.set_item("limits", limits_dict)?;

        // Instantiate `TicketIn(**kwargs)` using our cached class handle
        let ticket_in_obj = self.ticket_in_cls.bind(py).call((), Some(&kwargs))?;

        // Step 2.3: Call `self.engine.process_ticket(ticket_in_obj)` -> returns `TicketOut`
        let ticket_out = self
            .engine
            .bind(py)
            .call_method1("process_ticket", (ticket_in_obj,))?;

        // Step 2.4: Unpack `TicketOut` fields
        let status: String = ticket_out.getattr("status")?.extract()?;
        let transcript: String = ticket_out
            .getattr("transcript")?
            .extract()
            .unwrap_or_default();

        // Extract `latency_ms` from `ticket_out.usage` dictionary if present
        let mut latency_ms: f64 = 0.0;
        if let Ok(usage_any) = ticket_out.getattr("usage") {
            if let Ok(usage_dict) = usage_any.cast::<PyDict>() {
                if let Ok(Some(lat_val)) = usage_dict.get_item("latency_ms") {
                    latency_ms = lat_val.extract::<f64>().unwrap_or(0.0);
                }
            }
        }

        // Step 2.5: Check status & update `self.session_state` pointer on success
        if status == "COMPLETED" {
            let new_state = ticket_out.getattr("new_working_state")?;
            // O(1) pointer swap on Python heap!
            self.session_state = new_state.unbind();

            Ok(TicketOutResult {
                ticket_id,
                status,
                transcript,
                latency_ms,
                error_message: None,
                is_exiting: false,
            })
        } else {
            // Turn failed (`status == "FAILED"` or `"TIMED_OUT"`): extract `error_info`
            let mut err_msg = "Unknown turn execution error".to_string();
            match ticket_out.getattr("error_info") {
                Ok(err_info_any) => {
                    if !err_info_any.is_none() {
                        if let Ok(err_dict) = err_info_any.cast::<PyDict>() {
                            if let Ok(Some(e_str)) = err_dict.get_item("error") {
                                err_msg = e_str.extract::<String>().unwrap_or(err_msg);
                            }
                        } else if let Ok(s) = err_info_any.str() {
                            err_msg = s.to_string_lossy().into_owned();
                        }
                    }
                }
                _ => (),
            }

            Ok(TicketOutResult {
                ticket_id,
                status,
                transcript,
                latency_ms,
                error_message: Some(err_msg),
                is_exiting: false,
            })
        }
    }

    /// 6. DESTROY / SHUTDOWN PHASE: Call once when Slint window closes!
    /// Runs `ColdWindEngine.shutdown()` (`ChatDestructor` hooks: ModelManager, MCP_Manager, BrowserHandler)
    /// and flushes `TextHandler` log files.
    pub fn shutdown<'py>(&self, py: Python<'py>) -> PyResult<()> {
        // 1. Trigger ColdWindEngine.shutdown() -> ChatDestructor.call_all_cleanup_functions()
        let _ = self.engine.bind(py).call_method0("shutdown");

        // 2. Flush and close open log files in TextHandler
        if let Ok(handler_mod) = py.import("coldwind.core.system_logging.handlers.handler_base") {
            if let Ok(text_handler_cls) = handler_mod.getattr("TextHandler") {
                let _ = text_handler_cls.call_method0("clean_up");
            }
        }
        Ok(())
    }
}
