use ruff_python_ast::{Expr, Identifier, Parameter, ParameterWithDefault, Parameters};

/// A trimmed-down version of the Ruff Generator,
/// but only for generating the parameters of a function.
pub struct ParameterGenerator<'a> {
    buffer: String,
    render_expr: &'a dyn Fn(&Expr) -> String,
}

impl<'a> ParameterGenerator<'a> {
    /// `render_expr` turns an annotation or default expression into source text.
    #[must_use]
    pub fn new(render_expr: &'a dyn Fn(&Expr) -> String) -> Self {
        Self {
            buffer: String::new(),
            render_expr,
        }
    }

    fn p(&mut self, s: &str) {
        self.buffer.push_str(s);
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
        if let Some(annotation) = &parameter.annotation {
            self.p(": ");
            // Forward references are shown without their quotes
            let text = match annotation.as_ref() {
                Expr::StringLiteral(literal) => literal.value.to_str().to_owned(),
                other => (self.render_expr)(other),
            };
            self.p(&text);
        }
    }

    fn unparse_parameter_with_default(&mut self, parameter_with_default: &ParameterWithDefault) {
        self.unparse_parameter(&parameter_with_default.parameter);
        if let Some(default) = &parameter_with_default.default {
            // PEP 8: spaces around `=` only when the parameter is annotated
            let annotated = parameter_with_default.parameter.annotation.is_some();
            self.p(if annotated { " = " } else { "=" });
            let text = (self.render_expr)(default);
            self.p(&text);
        }
    }

    #[must_use]
    pub fn generate(self) -> String {
        self.buffer
    }
}
