use crate::*;
use crate::python::*;
use crate::parsing::python::*;
use crate::parsing::key_action::*;
use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyTypeError};
use std::array::IntoIter;

#[pyclass]
pub struct WriterInstanceHandle {
    exit_tx: oneshot::Sender<()>,
    join_handle: std::thread::JoinHandle<()>,
    ev_tx: mpsc::Sender<InputEvent>,
    message_tx: mpsc::UnboundedSender<ControlMessage>,
}

#[pymethods]
impl WriterInstanceHandle {
    pub fn send(&mut self, val: String) {
        let actions = parse_key_sequence_py(val.as_str()).unwrap();

        for action in actions.to_key_actions() {
            futures::executor::block_on(
                self.ev_tx.send(action.to_input_ev())
            ).unwrap();
            futures::executor::block_on(
                self.ev_tx.send(SYN_REPORT.clone())
            ).unwrap();
        }
    }
    pub fn send_modifier(&mut self, val: String) -> PyResult<()> {
        let actions = parse_key_sequence_py(val.as_str())
            .unwrap()
            .to_key_actions();

        if actions.len() != 1 {
            return Err(PyValueError::new_err(format!("expected a single key action, got {}", actions.len())));
        }

        let action = actions.get(0).unwrap();

        if [*KEY_LEFT_CTRL, *KEY_RIGHT_CTRL, *KEY_LEFT_ALT, *KEY_RIGHT_ALT, *KEY_LEFT_SHIFT, *KEY_RIGHT_SHIFT, *KEY_LEFT_META, *KEY_RIGHT_META]
            .contains(&action.key) {
            self.message_tx.send(ControlMessage::UpdateModifiers(*action));
        } else {
            return Err(PyValueError::new_err("key action needs to be a modifier event"));
        }

        futures::executor::block_on(self.ev_tx.send(action.to_input_ev())).unwrap();
        futures::executor::block_on(self.ev_tx.send(SYN_REPORT.clone())).unwrap();
        Ok(())
    }

    pub fn map(&mut self, py: Python, from: String, to: PyObject) -> PyResult<()> {
        if let Ok(to) = to.extract::<String>(py) {
            self._map_internal(from, to);
            return Ok(());
        }

        let is_callable = to.cast_as::<PyAny>(py)
            .map_or(false, |obj| obj.is_callable());

        if is_callable {
            self._map_callback(from, to);
            return Ok(());
        }

        return Err(PyTypeError::new_err("unknown type"));
    }

    fn _map_callback(&mut self, from: String, to: PyObject) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();

        match from {
            ParsedKeyAction::KeyAction(from) => {
                self.message_tx.send(ControlMessage::AddMapping(from, RuntimeAction::PythonCallback(to)));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(1), RuntimeAction::PythonCallback(to)));
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(0), RuntimeAction::NOP));
                self.message_tx.send(ControlMessage::AddMapping(from.to_key_action(2), RuntimeAction::NOP));
            }
        }

        Ok(())
    }

    fn _map_internal(&mut self, from: String, to: String) -> PyResult<()> {
        let from = parse_key_action_with_mods_py(&from).unwrap();
        let mut to = parse_key_sequence_py(&to).unwrap();

        match from {
            ParsedKeyAction::KeyAction(from) => {
                if to.len() == 1 {
                    let to = to.remove(0);
                    // action to click
                    if let ParsedKeyAction::KeyClickAction(to) = to {
                        let mapping = map_action_to_click(&from, &to);
                        self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                    // action to action
                    if let ParsedKeyAction::KeyAction(to) = to {
                        let mapping = map_action_to_action(&from, &to);
                        self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
                        return Ok(());
                    }
                }

                // action to seq
                let mapping = map_action_to_seq(from, to);
                self.message_tx.send(ControlMessage::AddMapping(mapping.0, mapping.1));
            }
            ParsedKeyAction::KeyClickAction(from) => {
                if to.len() == 1 {
                    match to.remove(0) {
                        // click to click
                        ParsedKeyAction::KeyClickAction(to) => {
                            let mappings = map_click_to_click(&from, &to);
                            IntoIter::new(mappings).for_each(|(from, to)| {
                                self.message_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                        // click to action
                        ParsedKeyAction::KeyAction(to) => {
                            let mappings = map_click_to_action(&from, &to);
                            IntoIter::new(mappings).for_each(|(from, to)| {
                                self.message_tx.send(ControlMessage::AddMapping(from, to));
                            });
                            return Ok(());
                        }
                    };
                }

                // click to seq
                let mappings = map_click_to_seq(from, to);
                IntoIter::new(mappings).for_each(|(from, to)| {
                    self.message_tx.send(ControlMessage::AddMapping(from, to));
                });
            }
        }

        Ok(())
    }
}
