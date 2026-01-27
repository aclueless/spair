use syn::{Block, Expr, Stmt};

#[derive(PartialEq, Debug)]
pub enum Stage {
    HtmlString(String),
    Creation,
    Update,
}

#[derive(PartialEq)]
pub enum StagePicker {
    DefaultToCreation,
    DefaultToUpdate,
    CheckWithUpdateVariables(Vec<String>),
    CheckWithCreationVariables(Vec<String>),
}

impl StagePicker {
    pub fn stage_of(&self, expr: &Expr) -> Stage {
        match self {
            StagePicker::DefaultToCreation => Stage::Creation,
            StagePicker::DefaultToUpdate => Stage::Update,
            StagePicker::CheckWithUpdateVariables(variable_names) => {
                if expr_has_ref_to(expr, variable_names) {
                    Stage::Update
                } else {
                    Stage::Creation
                }
            }
            StagePicker::CheckWithCreationVariables(variable_names) => {
                if expr_has_ref_to(expr, variable_names) {
                    Stage::Creation
                } else {
                    Stage::Update
                }
            }
        }
    }
}

pub(crate) fn expr_has_ref_to(expr: &Expr, variable_names: &[String]) -> bool {
    match expr {
        Expr::Array(expr_array) => {
            for expr in expr_array.elems.iter() {
                if expr_has_ref_to(expr, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Assign(_expr_assign) => false,
        Expr::Async(_expr_async) => false,
        Expr::Await(_expr_await) => false,
        Expr::Binary(expr_binary) => {
            expr_has_ref_to(&expr_binary.left, variable_names)
                || expr_has_ref_to(&expr_binary.right, variable_names)
        }
        Expr::Block(expr_block) => block_has_ref_to(&expr_block.block, variable_names),
        Expr::Break(_expr_break) => false,
        Expr::Call(expr_call) => {
            for arg in expr_call.args.iter() {
                if expr_has_ref_to(arg, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Cast(expr_cast) => expr_has_ref_to(&expr_cast.expr, variable_names),
        Expr::Closure(expr_closure) => expr_has_ref_to(&expr_closure.body, variable_names),
        Expr::Const(_expr_const) => false,
        Expr::Continue(_expr_continue) => false,
        Expr::Field(expr_field) => expr_has_ref_to(&expr_field.base, variable_names),
        Expr::ForLoop(expr_for_loop) => expr_has_ref_to(&expr_for_loop.expr, variable_names),
        Expr::Group(_expr_group) => false,
        Expr::If(expr_if) => {
            expr_has_ref_to(&expr_if.cond, variable_names)
                || block_has_ref_to(&expr_if.then_branch, variable_names)
                || expr_if
                    .else_branch
                    .as_ref()
                    .map(|else_branch| expr_has_ref_to(&else_branch.1, variable_names))
                    .unwrap_or_default()
        }
        Expr::Index(expr_index) => {
            expr_has_ref_to(&expr_index.expr, variable_names)
                || expr_has_ref_to(&expr_index.index, variable_names)
        }
        Expr::Infer(_expr_infer) => false,
        Expr::Let(expr_let) => expr_has_ref_to(&expr_let.expr, variable_names),
        Expr::Lit(_expr_lit) => false,
        Expr::Loop(expr_loop) => block_has_ref_to(&expr_loop.body, variable_names),
        Expr::Macro(_expr_macro) => true,
        Expr::Match(expr_match) => expr_has_ref_to(&expr_match.expr, variable_names),
        Expr::MethodCall(expr_method_call) => {
            if expr_has_ref_to(&expr_method_call.receiver, variable_names) {
                return true;
            }
            for a in expr_method_call.args.iter() {
                if expr_has_ref_to(a, variable_names) {
                    return false;
                }
            }
            false
        }
        Expr::Paren(expr_paren) => expr_has_ref_to(&expr_paren.expr, variable_names),
        Expr::Path(expr_path) => {
            if expr_path.qself.is_some() {
                return false;
            }
            if expr_path.path.segments.len() != 1 {
                return false;
            }
            for vname in variable_names.iter() {
                if expr_path.path.is_ident(vname) {
                    return true;
                }
            }
            false
        }
        Expr::Range(expr_range) => {
            if let Some(expr) = expr_range.start.as_ref()
                && expr_has_ref_to(expr, variable_names)
            {
                return true;
            }
            if let Some(expr) = expr_range.end.as_ref()
                && expr_has_ref_to(expr, variable_names)
            {
                return true;
            }
            false
        }
        Expr::RawAddr(_expr_raw_addr) => false,
        Expr::Reference(expr_reference) => expr_has_ref_to(&expr_reference.expr, variable_names),
        Expr::Repeat(expr_repeat) => expr_has_ref_to(&expr_repeat.expr, variable_names),
        Expr::Return(expr_return) => expr_return
            .expr
            .as_ref()
            .map(|v| expr_has_ref_to(v, variable_names))
            .unwrap_or(false),
        Expr::Struct(expr_struct) => {
            for f in expr_struct.fields.iter() {
                if expr_has_ref_to(&f.expr, variable_names) {
                    return true;
                }
            }
            if let Some(expr) = expr_struct.rest.as_ref() {
                expr_has_ref_to(expr, variable_names)
            } else {
                false
            }
        }
        Expr::Try(expr_try) => expr_has_ref_to(&expr_try.expr, variable_names),
        Expr::TryBlock(expr_try_block) => block_has_ref_to(&expr_try_block.block, variable_names),
        Expr::Tuple(expr_tuple) => {
            for expr in expr_tuple.elems.iter() {
                if expr_has_ref_to(expr, variable_names) {
                    return true;
                }
            }
            false
        }
        Expr::Unary(expr_unary) => expr_has_ref_to(&expr_unary.expr, variable_names),
        Expr::Unsafe(expr_unsafe) => block_has_ref_to(&expr_unsafe.block, variable_names),
        Expr::Verbatim(_token_stream) => false,
        Expr::While(expr_while) => {
            expr_has_ref_to(&expr_while.cond, variable_names)
                || block_has_ref_to(&expr_while.body, variable_names)
        }
        Expr::Yield(_expr_yield) => false,
        _ => false,
    }
}

fn block_has_ref_to(block: &Block, update_stage_variables: &[String]) -> bool {
    for stmt in block.stmts.iter() {
        if let Stmt::Expr(expr, _) = stmt
            && expr_has_ref_to(expr, update_stage_variables)
        {
            return true;
        }
    }
    false
}
