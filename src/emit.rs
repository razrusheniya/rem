use crate::*;

pub const ABI: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

impl Define {
    pub fn emit(defines: Vec<Self>) -> Result<String, String> {
        let mut extrn = String::new();
        let mut output = String::new();
        let ctx = &mut Context::default();

        for Define(name, args, body) in &defines {
            let mut addr = 8;
            let mut prologue = String::new();
            for (idx, _arg) in args.iter().enumerate() {
                if let Some(reg) = ABI.get(idx) {
                    prologue += &format!("\tmov qword [rbp-{addr}], {reg}\n");
                } else {
                    prologue += &format!(
                        "\tmov rax, [rbp+{}]\n\tmov qword [rbp-{addr}], rax\n",
                        (idx - 4) * 8
                    );
                }
                addr += 8;
            }

            ctx.local = Function::default();
            ctx.local.var = args.clone();
            let body = body.emit(ctx)?;

            output += &format!(
                "{name}:\n\tpush rbp\n\tmov rbp, rsp\n\tsub rsp, {}\n{prologue}{body}\tleave\n\tret\n\n",
                {
                    let bytes = ctx.local.var.len() * 8;
                    if bytes % 16 == 0 { bytes } else { bytes + 8 }
                }
            );
        }

        for Define(name, _, _) in &defines {
            ctx.global.func.swap_remove(name);
        }
        for symbol in &ctx.global.func {
            extrn += &format!("\textern {symbol}\n");
        }

        Ok(format!(
            "section .data\n{}\nsection .text\n\tglobal main\n{extrn}\n{output}\n",
            ctx.global.data,
        ))
    }
}

impl Expr {
    fn emit(&self, ctx: &mut Context) -> Result<String, String> {
        macro_rules! op {
            ($asm: literal, $lhs: expr, $rhs: expr) => {
                format!(
                    "{}\tpush rax\n{}\tmov r10, rax\n\tpop rax\n\t{} rax, r10\n",
                    $lhs.emit(ctx)?,
                    $rhs.emit(ctx)?,
                    $asm,
                )
            };
        }
        macro_rules! cmp {
            ($op: literal, $lhs: expr , $rhs: expr) => {
                format!(
                    "{}\tset{} al\n\tmovzx rax, al\n",
                    op!("cmp", $lhs, $rhs),
                    $op
                )
            };
        }
        macro_rules! read {
            ($size: expr) => {
                match $size {
                    Size::Byte => format!("\tmovzx rax, al\n"),
                    Size::Word => format!("\tmovzx rax, ax\n"),
                    Size::Long => format!("\tmovzx rax, eax\n"),
                    Size::Normal => String::new(),
                }
            };
        }
        macro_rules! write {
            ($size: expr) => {
                match $size {
                    Size::Byte => format!("\tmovzx r10b, r10\n"),
                    Size::Word => format!("\tmovzx r10w, r10\n"),
                    Size::Long => format!("\tmovzx r10d, r10\n"),
                    Size::Normal => String::new(),
                }
            };
        }
        macro_rules! label {
            () => {{
                let id = ctx.global.idx;
                ctx.global.idx += 1;
                id.to_string()
            }};
        }

        match self {
            Expr::If(cond, then, els) => {
                let id = label!();
                if let Some(els) = els {
                    Ok(format!(
                        "if.{id}:\n{}\tcmp rax, 0\n\tje else.{id}\n{}\tjmp end_if.{id}\nelse.{id}:\n{}end_if.{id}:\n",
                        cond.emit(ctx)?,
                        then.emit(ctx)?,
                        els.emit(ctx)?,
                    ))
                } else {
                    Ok(format!(
                        "if.{id}:\n{}\tcmp rax, 0\n\tje end_if.{id}\n{}end_if.{id}:\n",
                        cond.emit(ctx)?,
                        then.emit(ctx)?,
                    ))
                }
            }
            Expr::While(cond, body) => {
                let id = {
                    let id = label!();
                    ctx.local.jmp.push(id.clone());
                    id
                };
                let output = format!(
                    "while.{id}:\n{}\tcmp rax, 0\n\tje end_while.{id}\n{}\tjmp while.{id}\nend_while.{id}:\n",
                    cond.emit(ctx)?,
                    body.emit(ctx)?,
                );
                ctx.local.jmp.pop();
                Ok(output)
            }
            Expr::Break(expr) => {
                if let Some(jmp) = ctx.local.jmp.last().cloned() {
                    Ok(format!("{}\tjmp end_while.{jmp}\n", expr.emit(ctx)?))
                } else {
                    Err(format!("while loop not found"))
                }
            }
            Expr::Return(expr) => Ok(format!("{}\tleave\n\tret\n", expr.emit(ctx)?)),
            Expr::Block(lines) => Ok(lines
                .iter()
                .map(|line| line.emit(ctx))
                .collect::<Result<Vec<String>, String>>()?
                .concat()),
            Expr::Call(callee, args) => {
                let mut pusher = String::new();
                let mut argset = String::new();
                for (idx, arg) in args.iter().rev().enumerate() {
                    pusher += &format!("{}\tpush rax\n", arg.emit(ctx)?);
                    if let Some(reg) = ABI.get(idx) {
                        argset += &format!("\tpop {reg}\n");
                    }
                }

                let pre = pusher + &argset + &callee.emit(ctx)?;
                Ok(format!("{pre}\tmov r10, rax\n\txor rax, rax\n\tcall r10\n"))
            }
            Expr::Variable(name) => {
                if let Some(i) = ctx.local.var.get_index_of(name) {
                    Ok(format!("\tmov rax, [rbp-{}]\n", (i + 1) * 8))
                } else {
                    ctx.global.func.insert(name.clone());
                    Ok(format!("\tlea rax, [{name}]\n"))
                }
            }
            Expr::Pointer(var) => {
                if let Some(i) = ctx.local.var.get_index_of(var) {
                    Ok(format!("\tlea rax, [rbp-{}]\n", (i + 1) * 8))
                } else {
                    Err(format!("undefined variable: {var}"))
                }
            }
            Expr::Derefer(expr, size) => Ok(format!(
                "{}\tmovzx {size}, [rax]\n{}",
                expr.emit(ctx)?,
                read!(size)
            )),
            Expr::Let(name, value) => match &**name {
                Expr::Variable(name) => {
                    let idx = ctx.local.var.get_index_of(name).unwrap_or({
                        ctx.local.var.insert(name.clone());
                        ctx.local.var.len() - 1
                    });
                    Ok(format!(
                        "{}\tmov [rbp-{}], rax\n",
                        value.emit(ctx)?,
                        (idx + 1) * 8,
                    ))
                }
                Expr::Derefer(ptr, size) => Ok(format!(
                    "{}\tpush rax\n{}\tpop r10\n\t{}\n\tmov {size:?} [rax], r10\n",
                    value.emit(ctx)?,
                    ptr.emit(ctx)?,
                    write!(size),
                )),
                _ => Err(format!("invalid assign　to: {name:?}")),
            },
            Expr::Integer(value) => Ok(format!("\tmov rax, {value}\n")),
            Expr::String(value) => {
                let value = value
                    .replace("\\n", "\", 10, \"")
                    .replace("\\\"", "\", 34, \"");

                let name = format!("str.{}", label!());
                let code = format!("\t{name} db {value}, 0\n");
                ctx.global.data += &code;

                Ok(format!("\tmov rax, {name}\n"))
            }
            Expr::Add(lhs, rhs) => Ok(op!("add", lhs, rhs)),
            Expr::Sub(lhs, rhs) => Ok(op!("sub", lhs, rhs)),
            Expr::Mul(lhs, rhs) => Ok(op!("imul", lhs, rhs)),
            Expr::Eql(lhs, rhs) => Ok(cmp!("e", lhs, rhs)),
            Expr::NotEq(lhs, rhs) => Ok(cmp!("ne", lhs, rhs)),
            Expr::Gt(lhs, rhs) => Ok(cmp!("g", lhs, rhs)),
            Expr::Lt(lhs, rhs) => Ok(cmp!("l", lhs, rhs)),
            Expr::GtEq(lhs, rhs) => Ok(cmp!("ge", lhs, rhs)),
            Expr::LtEq(lhs, rhs) => Ok(cmp!("le", lhs, rhs)),
            Expr::And(lhs, rhs) => Ok(op!("and", lhs, rhs)),
            Expr::Or(lhs, rhs) => Ok(op!("or", lhs, rhs)),
            Expr::Xor(lhs, rhs) => Ok(op!("xor", lhs, rhs)),
            Expr::Div(lhs, rhs) => Ok(format!(
                "{}\tpush rax\n{}\tmov rsi, rax\n\tpop rax\n\tcqo\n\tidiv rsi\n",
                lhs.emit(ctx)?,
                rhs.emit(ctx)?,
            )),
            Expr::Mod(lhs, rhs) => {
                Ok(Expr::Div(lhs.clone(), rhs.clone()).emit(ctx)? + "\tmov rax, rdx\n")
            }
        }
    }
}
