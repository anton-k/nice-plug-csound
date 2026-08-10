use nice_plug::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

/// State relating to the editor itself (not necessarly the GUI). Put any
/// state that should persist between editor opens here.
pub struct CsoundEditorState {
    pub params: Arc<ParamList>,
}

pub trait CsoundParams {
    fn get(&self, name: &str) -> Option<&FloatParam>;
}

pub struct ParamList {
    pub params: HashMap<String, FloatParam>,
}

impl CsoundParams for ParamList {
    fn get(&self, name: &str) -> Option<&FloatParam> {
        self.params.get(name)
    }
}

impl ParamList {
    pub fn get_param(&self, name: &str) -> Option<&FloatParam> {
        self.params.get(name)
    }
}

impl ParamList {
    pub fn new(_names: &[String]) -> Self {
        let names = _names.to_vec();
        let values: Vec<FloatParam> = names
            .iter()
            .map(|name| FloatParam::new(name, 0.5, FloatRange::Linear { min: 0.0, max: 1.0 }))
            .collect();

        let params: HashMap<String, FloatParam> = names.into_iter().zip(values).collect();

        Self { params }
    }
}

unsafe impl Params for ParamList {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        self.params
            .iter()
            .map(|(name, ptr)| {
                (
                    name.clone(),
                    ParamPtr::FloatParam(ptr as *const FloatParam),
                    "".to_string(),
                )
            })
            .collect()
    }
}
