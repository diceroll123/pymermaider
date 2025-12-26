use ruff_python_ast::{Identifier, Parameter, ParameterWithDefault, Parameters};

/// A trimmed-down version of the Ruff Generator,
/// but only for generating the parameters of a function.
pub struct ParameterGenerator {
    buffer: String,
}

impl Default for ParameterGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ParameterGenerator {
    pub const fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    fn p(&mut self, s: &str) {
        self.buffer += s;
    }

    fn p_id(&mut self, s: &Identifier) {
        self.p(s.as_str());
    }

    fn p_if(&mut self, cond: bool, s: &str) {
        if cond {
            self.p(s);
        }
    }

    fn p_delim(&mut self, first: &mut bool, s: &str) {
        self.p_if(!core::mem::take(first), s);
    }

    pub fn unparse_parameters(&mut self, parameters: &Parameters) {
        let mut first = true;
        for (i, parameter_with_default) in parameters
            .posonlyargs
            .iter()
            .chain(&parameters.args)
            .enumerate()
        {
            self.p_delim(&mut first, ", ");
            self.unparse_parameter_with_default(parameter_with_default);
            self.p_if(i + 1 == parameters.posonlyargs.len(), ", /");
        }
        if parameters.vararg.is_some() || !parameters.kwonlyargs.is_empty() {
            self.p_delim(&mut first, ", ");
            self.p("*");
        }
        if let Some(vararg) = &parameters.vararg {
            self.unparse_parameter(vararg);
        }
        for kwarg in &parameters.kwonlyargs {
            self.p_delim(&mut first, ", ");
            self.unparse_parameter_with_default(kwarg);
        }
        if let Some(kwarg) = &parameters.kwarg {
            self.p_delim(&mut first, ", ");
            self.p("**");
            self.unparse_parameter(kwarg);
        }
    }

    fn unparse_parameter(&mut self, parameter: &Parameter) {
        self.p_id(&parameter.name);
    }

    fn unparse_parameter_with_default(&mut self, parameter_with_default: &ParameterWithDefault) {
        self.unparse_parameter(&parameter_with_default.parameter);
    }

    pub fn generate(self) -> String {
        self.buffer
    }
}
