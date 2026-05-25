use anyhow::Result;
use pine_cli::syntax;

use crate::output::{ResolvedFormat, print_json};

pub(crate) fn run(code: &str, format: ResolvedFormat) -> Result<()> {
    let mut lexer = syntax::Lexer::new(code);
    let tokens = lexer
        .tokenize()
        .map_err(|e| anyhow::anyhow!("lex error: {e}"))?;
    let mut parser = syntax::Parser::new(tokens);
    let statements = parser
        .parse()
        .map_err(|e| anyhow::anyhow!("parse error: {e}"))?;
    let program = syntax::Program::new(statements);

    match format {
        ResolvedFormat::Json => {
            print_json(&program)?;
        }
        ResolvedFormat::Text => {
            let mut out = String::new();
            render_program(&program, &mut out);
            print!("{out}");
        }
    }
    Ok(())
}

// ---------- AST pretty-printer ----------
//
// Indented-text rendering of the parsed AST. Each node prints one line
// with its kind + a brief identifier (variable name, operator, literal
// value, etc.); children indent by two spaces. ASCII-only.

fn render_program(p: &syntax::Program, out: &mut String) {
    out.push_str("Program\n");
    for stmt in &p.statements {
        render_stmt(stmt, 1, out);
    }
}

fn render_stmt(stmt: &syntax::ast::Stmt, depth: usize, out: &mut String) {
    use syntax::ast::Stmt;
    let pad = indent(depth);
    match stmt {
        Stmt::VarDecl {
            name,
            type_qualifier,
            type_annotation,
            initializer,
            is_varip,
        } => {
            let kw = if *is_varip { "varip" } else { "var" };
            let q = type_qualifier
                .as_ref()
                .map(|q| format!("{q:?} "))
                .unwrap_or_default();
            let ty = type_annotation
                .as_ref()
                .map(|t| format!(": {t}"))
                .unwrap_or_default();
            out.push_str(&format!("{pad}VarDecl {kw} {q}{name}{ty}\n"));
            if let Some(init) = initializer {
                render_expr(init, depth + 1, out);
            }
        }
        Stmt::Assignment { target, value } => {
            out.push_str(&format!("{pad}Assignment\n"));
            out.push_str(&format!("{}target:\n", indent(depth + 1)));
            render_expr(target, depth + 2, out);
            out.push_str(&format!("{}value:\n", indent(depth + 1)));
            render_expr(value, depth + 2, out);
        }
        Stmt::TupleAssignment { names, value } => {
            out.push_str(&format!("{pad}TupleAssignment [{}]\n", names.join(", ")));
            render_expr(value, depth + 1, out);
        }
        Stmt::Expression(e) => {
            out.push_str(&format!("{pad}Expression\n"));
            render_expr(e, depth + 1, out);
        }
        Stmt::If {
            condition,
            then_branch,
            else_if_branches,
            else_branch,
        } => {
            out.push_str(&format!("{pad}If\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            for s in then_branch {
                render_stmt(s, depth + 2, out);
            }
            for (i, (cond, body)) in else_if_branches.iter().enumerate() {
                out.push_str(&format!("{}else-if[{i}] cond:\n", indent(depth + 1)));
                render_expr(cond, depth + 2, out);
                out.push_str(&format!("{}else-if[{i}] body:\n", indent(depth + 1)));
                for s in body {
                    render_stmt(s, depth + 2, out);
                }
            }
            if let Some(eb) = else_branch {
                out.push_str(&format!("{}else:\n", indent(depth + 1)));
                for s in eb {
                    render_stmt(s, depth + 2, out);
                }
            }
        }
        Stmt::For {
            var_name,
            from,
            to,
            body,
        } => {
            out.push_str(&format!("{pad}For {var_name}\n"));
            out.push_str(&format!("{}from:\n", indent(depth + 1)));
            render_expr(from, depth + 2, out);
            out.push_str(&format!("{}to:\n", indent(depth + 1)));
            render_expr(to, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::ForIn {
            index_var,
            item_var,
            collection,
            body,
        } => {
            let pat = match index_var {
                Some(idx) => format!("[{idx}, {item_var}]"),
                None => item_var.clone(),
            };
            out.push_str(&format!("{pad}ForIn {pat}\n"));
            out.push_str(&format!("{}collection:\n", indent(depth + 1)));
            render_expr(collection, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::While { condition, body } => {
            out.push_str(&format!("{pad}While\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}body:\n", indent(depth + 1)));
            for s in body {
                render_stmt(s, depth + 2, out);
            }
        }
        Stmt::Break => out.push_str(&format!("{pad}Break\n")),
        Stmt::Continue => out.push_str(&format!("{pad}Continue\n")),
        Stmt::TypeDecl {
            name,
            fields,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            out.push_str(&format!("{pad}TypeDecl {e}{name}\n"));
            for f in fields {
                out.push_str(&format!(
                    "{}field {} : {}\n",
                    indent(depth + 1),
                    f.name,
                    f.type_annotation
                ));
                if let Some(dv) = &f.default_value {
                    render_expr(dv, depth + 2, out);
                }
            }
        }
        Stmt::MethodDecl {
            name,
            params,
            body,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}MethodDecl {e}{name}({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Stmt::EnumDecl {
            name,
            fields,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            out.push_str(&format!("{pad}EnumDecl {e}{name}\n"));
            for f in fields {
                let t = f
                    .title
                    .as_ref()
                    .map(|t| format!(" = {t:?}"))
                    .unwrap_or_default();
                out.push_str(&format!("{}variant {}{t}\n", indent(depth + 1), f.name));
            }
        }
        Stmt::FunctionDecl {
            name,
            params,
            body,
            export,
        } => {
            let e = if *export { "export " } else { "" };
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}FunctionDecl {e}{name}({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Stmt::Export { item } => {
            use syntax::ast::ExportItem;
            let label = match item {
                ExportItem::Type(n) => format!("type {n}"),
                ExportItem::Function(n) => format!("function {n}"),
            };
            out.push_str(&format!("{pad}Export {label}\n"));
        }
        Stmt::Import { path, alias } => {
            out.push_str(&format!("{pad}Import {path} as {alias}\n"));
        }
    }
}

fn render_expr(expr: &syntax::ast::Expr, depth: usize, out: &mut String) {
    use syntax::ast::Expr;
    let pad = indent(depth);
    match expr {
        Expr::Literal(lit) => {
            out.push_str(&format!("{pad}Literal {}\n", render_literal(lit)));
        }
        Expr::Variable(name) => {
            out.push_str(&format!("{pad}Variable {name}\n"));
        }
        Expr::Binary { left, op, right } => {
            out.push_str(&format!("{pad}Binary {op:?}\n"));
            render_expr(left, depth + 1, out);
            render_expr(right, depth + 1, out);
        }
        Expr::Unary { op, expr: inner } => {
            out.push_str(&format!("{pad}Unary {op:?}\n"));
            render_expr(inner, depth + 1, out);
        }
        Expr::Call {
            callee,
            type_args,
            args,
        } => {
            let targs = if type_args.is_empty() {
                String::new()
            } else {
                format!("<{}>", type_args.join(", "))
            };
            out.push_str(&format!("{pad}Call{targs}\n"));
            out.push_str(&format!("{}callee:\n", indent(depth + 1)));
            render_expr(callee, depth + 2, out);
            for (i, arg) in args.iter().enumerate() {
                use syntax::ast::Argument;
                match arg {
                    Argument::Positional(e) => {
                        out.push_str(&format!("{}arg[{i}]:\n", indent(depth + 1)));
                        render_expr(e, depth + 2, out);
                    }
                    Argument::Named { name, value } => {
                        out.push_str(&format!("{}arg[{i}] {name}=:\n", indent(depth + 1)));
                        render_expr(value, depth + 2, out);
                    }
                }
            }
        }
        Expr::Index { expr: inner, index } => {
            out.push_str(&format!("{pad}Index\n"));
            render_expr(inner, depth + 1, out);
            out.push_str(&format!("{}[\n", indent(depth + 1)));
            render_expr(index, depth + 2, out);
            out.push_str(&format!("{}]\n", indent(depth + 1)));
        }
        Expr::MemberAccess { object, member } => {
            out.push_str(&format!("{pad}MemberAccess .{member}\n"));
            render_expr(object, depth + 1, out);
        }
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            out.push_str(&format!("{pad}Ternary\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            render_expr(then_expr, depth + 2, out);
            out.push_str(&format!("{}else:\n", indent(depth + 1)));
            render_expr(else_expr, depth + 2, out);
        }
        Expr::Function { params, body } => {
            let plist = params
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{pad}Function ({plist})\n"));
            for s in body {
                render_stmt(s, depth + 1, out);
            }
        }
        Expr::Array(items) => {
            out.push_str(&format!("{pad}Array [{}]\n", items.len()));
            for item in items {
                render_expr(item, depth + 1, out);
            }
        }
        Expr::Switch { value, cases } => {
            out.push_str(&format!("{pad}Switch\n"));
            out.push_str(&format!("{}value:\n", indent(depth + 1)));
            render_expr(value, depth + 2, out);
            for (i, (pat, result)) in cases.iter().enumerate() {
                out.push_str(&format!("{}case[{i}] pattern:\n", indent(depth + 1)));
                render_expr(pat, depth + 2, out);
                out.push_str(&format!("{}case[{i}] result:\n", indent(depth + 1)));
                render_expr(result, depth + 2, out);
            }
        }
        Expr::IfExpr {
            condition,
            then_expr,
            else_if_branches,
            else_expr,
        } => {
            out.push_str(&format!("{pad}IfExpr\n"));
            out.push_str(&format!("{}cond:\n", indent(depth + 1)));
            render_expr(condition, depth + 2, out);
            out.push_str(&format!("{}then:\n", indent(depth + 1)));
            render_expr(then_expr, depth + 2, out);
            for (i, (c, e)) in else_if_branches.iter().enumerate() {
                out.push_str(&format!("{}else-if[{i}] cond:\n", indent(depth + 1)));
                render_expr(c, depth + 2, out);
                out.push_str(&format!("{}else-if[{i}] expr:\n", indent(depth + 1)));
                render_expr(e, depth + 2, out);
            }
            if let Some(e) = else_expr {
                out.push_str(&format!("{}else:\n", indent(depth + 1)));
                render_expr(e, depth + 2, out);
            }
        }
    }
}

fn render_literal(lit: &syntax::ast::Literal) -> String {
    use syntax::ast::Literal;
    match lit {
        Literal::Number(n) => format!("Number {n}"),
        Literal::String(s) => format!("String {s:?}"),
        Literal::Bool(b) => format!("Bool {b}"),
        Literal::Na => "Na".to_string(),
        Literal::HexColor(c) => format!("HexColor {c}"),
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_pretty_print_smoke() {
        let mut lexer = syntax::Lexer::new("x = 5 + 3\n");
        let tokens = lexer.tokenize().expect("lex");
        let mut parser = syntax::Parser::new(tokens);
        let stmts = parser.parse().expect("parse");
        let program = syntax::Program::new(stmts);
        let mut out = String::new();
        render_program(&program, &mut out);
        // Expect a tree-shaped header + the binary expression decomposed
        // into operator + operands.
        assert!(out.starts_with("Program\n"));
        assert!(out.contains("VarDecl"));
        assert!(out.contains("Binary Add"));
        assert!(out.contains("Number 5"));
        assert!(out.contains("Number 3"));
    }
}
