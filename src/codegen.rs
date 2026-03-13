//! Code generator - Emits x86-64 assembly (Linux System V ABI).

use crate::ast::*;
use std::collections::HashMap;

pub struct CodeGen {
    output: String,
    /// Stack offset for local variables (negative from rbp)
    var_offset: i32,
    /// Map of variable name -> stack offset
    vars: HashMap<String, i32>,
    /// Unique label counter
    label_counter: u32,
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            var_offset: 0,
            vars: HashMap::new(),
            label_counter: 0,
        }
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        self.label_counter += 1;
        format!("{}.{}", prefix, self.label_counter)
    }

    pub fn compile(&mut self, program: &Program) -> String {
        self.emit(".intel_syntax noprefix");
        self.emit(".globl main");
        self.emit("");

        for func in &program.functions {
            self.compile_function(func);
        }

        std::mem::take(&mut self.output)
    }

    fn compile_function(&mut self, func: &Function) {
        self.var_offset = 0;
        self.vars.clear();

        self.emit(&format!("{}:", func.name));
        self.emit("  push rbp");
        self.emit("  mov rbp, rsp");

        // Allocate space for locals (we'll track as we go)
        // Params: rdi, rsi, rdx, rcx, r8, r9 (first 6)
        let param_regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (i, param) in func.params.iter().enumerate() {
            if i < 6 {
                self.emit(&format!("  mov [rbp-{}], {}", 8 * (i + 1), param_regs[i]));
                self.vars.insert(param.clone(), 8 * (i + 1) as i32);
            }
        }

        let param_stack = func.params.len().saturating_sub(6) * 8;
        if param_stack > 0 {
            self.var_offset = 16 + param_stack as i32; // rbp + ret addr
            for (i, param) in func.params.iter().skip(6).enumerate() {
                let off = 16 + (i as i32 + 1) * 8;
                self.vars.insert(param.clone(), off);
            }
        } else {
            self.var_offset = 8;
        }

        for stmt in &func.body {
            self.compile_statement(stmt);
        }

        // Implicit return 0 when falling off end
        self.emit("  xor eax, eax");
        self.emit("  mov rsp, rbp");
        self.emit("  pop rbp");
        self.emit("  ret");
        self.emit("");
    }

    fn compile_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Decl { name } => {
                self.vars.insert(name.clone(), self.var_offset);
                self.var_offset += 8;
                self.emit(&format!("  sub rsp, 8  # {} (uninitialized)", name));
            }
            Statement::DeclInit { name, init } => {
                self.vars.insert(name.clone(), self.var_offset);
                self.var_offset += 8;
                self.emit(&format!("  sub rsp, 8  # {}", name));
                self.compile_expr(init);
                self.emit(&format!("  mov [rbp-{}], eax", self.vars[name]));
            }
            Statement::Expr(expr) => {
                self.compile_expr(expr);
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e);
                } else {
                    self.emit("  xor eax, eax");
                }
                self.emit("  mov rsp, rbp");
                self.emit("  pop rbp");
                self.emit("  ret");
            }
            Statement::If {
                cond,
                then_body,
                else_body,
            } => {
                let else_label = self.fresh_label("else");
                let end_label = self.fresh_label("endif");
                self.compile_expr(cond);
                self.emit("  cmp eax, 0");
                if else_body.is_some() {
                    self.emit(&format!("  je {}", else_label));
                } else {
                    self.emit(&format!("  je {}", end_label));
                }
                for s in then_body {
                    self.compile_statement(s);
                }
                if else_body.is_some() {
                    self.emit(&format!("  jmp {}", end_label));
                    self.emit(&format!("{}:", else_label));
                    for s in else_body.as_ref().unwrap() {
                        self.compile_statement(s);
                    }
                }
                self.emit(&format!("{}:", end_label));
            }
            Statement::While { cond, body } => {
                let cond_label = self.fresh_label("while_cond");
                let body_label = self.fresh_label("while_body");
                self.emit(&format!("  jmp {}", cond_label));
                self.emit(&format!("{}:", body_label));
                for s in body {
                    self.compile_statement(s);
                }
                self.emit(&format!("{}:", cond_label));
                self.compile_expr(cond);
                self.emit("  cmp eax, 0");
                self.emit(&format!("  jne {}", body_label));
            }
            Statement::For {
                init,
                cond,
                step,
                body,
            } => {
                if let Some(i) = init {
                    self.compile_statement(i);
                }
                let cond_label = self.fresh_label("for_cond");
                let body_label = self.fresh_label("for_body");
                self.emit(&format!("  jmp {}", cond_label));
                self.emit(&format!("{}:", body_label));
                for s in body {
                    self.compile_statement(s);
                }
                if let Some(s) = step {
                    self.compile_expr(s);
                }
                self.emit(&format!("{}:", cond_label));
                if let Some(c) = cond {
                    self.compile_expr(c);
                    self.emit("  cmp eax, 0");
                    self.emit(&format!("  jne {}", body_label));
                } else {
                    self.emit(&format!("  jmp {}", body_label));
                }
            }
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                self.emit(&format!("  mov eax, {}", n));
            }
            Expr::Ident(name) => {
                let off = self.vars.get(name).expect("Unknown variable");
                self.emit(&format!("  mov eax, [rbp-{}]", off));
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(right);
                self.emit("  push rax");
                self.compile_expr(left);
                self.emit("  pop rcx");
                match op {
                    BinOp::Add => self.emit("  add eax, ecx"),
                    BinOp::Sub => {
                        self.emit("  sub eax, ecx");
                    }
                    BinOp::Mul => self.emit("  imul eax, ecx"),
                    BinOp::Div => {
                        self.emit("  cdq");
                        self.emit("  idiv ecx");
                    }
                    BinOp::Mod => {
                        self.emit("  cdq");
                        self.emit("  idiv ecx");
                        self.emit("  mov eax, edx");
                    }
                    BinOp::Eq => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  sete al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::Ne => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  setne al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::Lt => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  setl al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::Le => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  setle al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::Gt => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  setg al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::Ge => {
                        self.emit("  cmp eax, ecx");
                        self.emit("  setge al");
                        self.emit("  movzx eax, al");
                    }
                    BinOp::AndAnd => {
                        let skip = self.fresh_label("and_skip");
                        self.emit("  cmp eax, 0");
                        self.emit(&format!("  je {}", skip));
                        self.emit("  mov eax, ecx");
                        self.emit(&format!("{}:", skip));
                    }
                    BinOp::OrOr => {
                        let skip = self.fresh_label("or_skip");
                        self.emit("  cmp eax, 0");
                        self.emit(&format!("  jne {}", skip));
                        self.emit("  mov eax, ecx");
                        self.emit(&format!("{}:", skip));
                    }
                }
            }
            Expr::UnaryOp { op, operand } => {
                self.compile_expr(operand);
                match op {
                    UnaryOp::Neg => self.emit("  neg eax"),
                    UnaryOp::Not => {
                        self.emit("  cmp eax, 0");
                        self.emit("  sete al");
                        self.emit("  movzx eax, al");
                    }
                }
            }
            Expr::Assign { name, value } => {
                self.compile_expr(value);
                let off = self.vars.get(name).expect("Unknown variable");
                self.emit(&format!("  mov [rbp-{}], eax", off));
            }
            Expr::Call { name, args } => {
                // System V: rdi, rsi, rdx, rcx, r8, r9
                let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for (i, arg) in args.iter().take(6).enumerate() {
                    self.compile_expr(arg);
                    self.emit(&format!("  mov {}, eax", regs[i]));
                }
                for arg in args.iter().skip(6) {
                    self.compile_expr(arg);
                    self.emit("  push rax");
                }
                self.emit(&format!("  call {}", name));
                if args.len() > 6 {
                    self.emit(&format!("  add rsp, {}", (args.len() - 6) * 8));
                }
            }
        }
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}
