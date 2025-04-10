// Copyright (c) Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

//! This module implements an expression linter that checks for boolean identities
//! that occur in conditionals:
//! 1. `x && true`, which can be replaced with just `x`.
//! 2. `x && false`, which can be replaced with just `false`.
//! 3. `x || true`, which can be replaced with just `true`.
//! 4. `x || false`, which can be replaced with just `x`.
//! 5. `x <==> true`, which can be replaced with just `x`.
//! 6. `x <==> false`, which can be replaced with just `!x`.
//! 7. `x == true`, which can be replaced with just `x`.
//! 8. `x == false`, which can be replaced with just `!x`.
//! 9. `x != true`, which can be replaced with just `!x`.
//! 10. `x != false`, which can be replaced with just `x`.
//! 11. `x ==> true`, which can be replaced with just `true`.
//! 12. `x ==> false`, which can be replaced with just `!x`.
//! 13. `true ==> x`, which can be replaced with just `x`.
//! 14. `false ==> x`, which can be replaced with just `true`.
//! 15. `!true`, which can be replaced with just `false`.
//! 16. `!false`, which can be replaced with just `true`.
//!
//! Note also that rules 1 through 10 have both LHS and RHS version

use move_compiler_v2::external_checks::ExpChecker;
use move_model::{
    ast::{ExpData, Operation, Value},
    model::GlobalEnv,
};

#[derive(Default)]
pub struct BoolIdentProp;

impl ExpChecker for BoolIdentProp {
    fn get_name(&self) -> String {
        "bool_ident_prop".to_string()
    }

    fn visit_expr_pre(&mut self, env: &GlobalEnv, expr: &ExpData) {
        use ExpData::{Call, Value as ExpValue};
        use Operation::*;
        use Value::Bool;

        let get_msg = |cmp: &Operation, b: bool, side| {
            match (cmp, b, side) {
                (And, true, s) | (Or, false, s) => {
                    Some(format!(
                        "The {} occurrence of `{}` is redundant and should be removed.",
                        if s == 0 { "left" } else { "right" },
                        b
                    ))
                },
                (And, false, _) | (Or, true, _) | (Implies, true, 1) => {
                    Some(format!(
                        "This expression can be simplified as `{}`.",
                        b
                    ))
                },
                (Implies, false, 0) => {
                    Some("This expression can be simplified as `true`.".to_string())
                },
                (Eq, _, _) | (Neq, _, _) | (Iff, _, _) | (Implies, true, 0) | (Implies, false, 1) => {
                    let relation = match (cmp, side) {
                        (Eq, _) => "is equal to",
                        (Neq, _) => "is not equal to",
                        (Iff, _) => "is equivalent to",
                        (Implies, 0) => "is implied by",
                        (Implies, 1) => "implies",
                        _ => unreachable!(),
                    };
                    Some(format!(
                        "Directly use the {}boolean expression instead of checking if it {} `{}`.",
                        if b && !matches!(*cmp, Neq) { "" } else { "negation of " },
                        relation,
                        b
                    ))
                }
                _ => panic!("Unexpected operation encountered: {:?}", cmp),
            }
        };
        let Call(_, cmp, args) = expr else {
            return;
        };
        let msg = match cmp {
            And | Or | Eq | Neq | Implies | Iff =>{
                match (args[0].as_ref(), args[1].as_ref()) {
                    // When one of the arguments is a boolean literal (true or false)
                    (ExpValue(_, Bool(b)), _) => {
                        get_msg(cmp, *b, 0)
                    }
                    (_, ExpValue(_, Bool(b))) => {
                        get_msg(cmp, *b, 1)
                    }
                    _ => None,
                }
            },
            Not => match args[0].as_ref() {
                ExpValue(_, Bool(b)) => {
                    Some(format!(
                        "This expression can be simplified as `{}`.",
                        !b
                    ))},
                _ => None
            }
            _ => None
        };


        if let Some(msg) = msg {
            self.report(
                env,
                &env.get_node_loc(expr.node_id()),
                &msg
            );
        }
    }
}