use grammar::*;
use grammar::OperatorType::*;

/// The `Generator` is a wrapper around an owned `String` that's used to
/// stringify the AST. There is a bunch of useful methods here to manage
/// things like indentation and automatically producing minified code.
struct Generator {
    pub minify: bool,
    code: String,
    dent: u16,
}

impl Generator {
    pub fn new(minify: bool) -> Self {
        Generator {
            minify: minify,
            code: String::new(),
            dent: 0,
        }
    }

    pub fn new_line(&mut self) {
        if !self.minify {
            self.code.push('\n');
            for _ in 0..self.dent {
                self.code.push_str("    ");
            }
        }
    }

    pub fn write(&mut self, slice: &str) {
        self.code.push_str(slice);
    }

    pub fn write_min(&mut self, slice: &str, minslice: &str) {
        if self.minify {
            self.write(minslice);
        } else {
            self.write(slice);
        }
    }

    pub fn write_char(&mut self, ch: char) {
        self.code.push(ch);
    }

    pub fn write_list<T: Code>(&mut self, items: &Vec<T>) {
        let mut first = true;
        for item in items {
            if first {
                first = false;
            } else {
                self.write_min(", ", ",");
            }
            item.to_code(self);
        }
    }

    pub fn write_block<T: Code>(&mut self, items: &Vec<T>) {
        self.indent();
        for item in items {
            self.new_line();
            item.to_code(self);
        }
        self.dedent();
        self.new_line();
    }

    pub fn write_declaration_or_expression(&mut self, statement: &Statement) {
        match *statement {
            Statement::VariableDeclaration {
                ref kind,
                ref declarators,
            } => {
                kind.to_code(self);
                self.write_char(' ');
                self.write_list(declarators);
            },

            Statement::Expression {
                ref value,
            } => {
                value.to_code(self);
            },

            _ => panic!("Invalid AST structure!"),
        }
    }

    pub fn indent(&mut self) {
        self.dent += 1;
    }

    pub fn dedent(&mut self) {
        self.dent -= 1;
    }

    pub fn consume(self) -> String {
        self.code
    }
}

/// The `Code` trait provides an interface to pieces of grammar, that allows
/// to efficiently write characters and string slices to the code `Generator`.
trait Code {
    fn to_code(&self, gen: &mut Generator);
}

impl Code for String {
    fn to_code(&self, gen: &mut Generator) {
        gen.write(self);
    }
}

impl<T: Code> Code for Option<T> {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Some(ref value) => value.to_code(gen),
            None            => {}
        }
    }
}

impl Code for OperatorType {
    fn to_code(&self, gen: &mut Generator) {
        gen.write(match *self {
            FatArrow         => "=>",
            Accessor         => ".",
            New              => "new",
            Increment        => "++",
            Decrement        => "--",
            LogicalNot       => "!",
            BitwiseNot       => "~",
            Typeof           => "typeof",
            Void             => "void",
            Delete           => "delete",
            Multiplication   => "*",
            Division         => "/",
            Remainder        => "%",
            Exponent         => "**",
            Addition         => "+",
            Substraction     => "-",
            BitShiftLeft     => "<<",
            BitShiftRight    => ">>",
            UBitShiftRight   => ">>>",
            Lesser           => "<",
            LesserEquals     => "<=",
            Greater          => ">",
            GreaterEquals    => ">=",
            Instanceof       => "instanceof",
            In               => "in",
            StrictEquality   => "===",
            StrictInequality => "!==",
            Equality         => "==",
            Inequality       => "!=",
            BitwiseAnd       => "&",
            BitwiseXor       => "^",
            BitwiseOr        => "|",
            LogicalAnd       => "&&",
            LogicalOr        => "||",
            Conditional      => "?",
            Assign           => "=",
            AddAssign        => "+=",
            SubstractAssign  => "-=",
            ExponentAssign   => "**=",
            MultiplyAssign   => "*=",
            DivideAssign     => "/=",
            RemainderAssign  => "%=",
            BSLAssign        => "<<=",
            BSRAssign        => ">>=",
            UBSRAssign       => ">>>=",
            BitAndAssign     => "&=",
            BitXorAssign     => "^=",
            BitOrAssign      => "|=",
            Spread           => "...",
        });
    }
}

impl Code for LiteralValue {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            LiteralUndefined          => gen.write_min("undefined", "void 0"),
            LiteralNull               => gen.write("null"),
            LiteralTrue               => gen.write_min("true", "!0",),
            LiteralFalse              => gen.write_min("false", "!1"),
            LiteralInteger(ref num)   => gen.write(&num.to_string()),
            LiteralFloat(ref num)     => gen.write(&num.to_string()),
            LiteralString(ref string) => gen.write(&format!("{:?}", string))
        }
    }
}

fn is_identifier(label: &String) -> bool {
    let mut chars = label.chars();

    // All identifiers have to have at least one char, so unwrap is safe here.
    let first = chars.next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    for ch in chars {
        if !ch.is_alphanumeric() && ch != '_' && ch != '$' {
            return false;
        }
    }

    true
}

impl Code for ObjectMember {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            ObjectMember::Shorthand {
                ref key
            } => gen.write(key),

            ObjectMember::Literal {
                ref key,
                ref value,
            } => {
                if is_identifier(key) {
                    gen.write(key);
                } else {
                    gen.write(&format!("{:?}", key));
                }
                gen.write_min(": ", ":");
                value.to_code(gen);
            },

            ObjectMember::Computed {
                ref key,
                ref value,
            } => {
                gen.write_char('[');
                key.to_code(gen);
                gen.write_min("]: ", "]:");
                value.to_code(gen);
            },

            ObjectMember::Method {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write(name);
                gen.write_char('(');
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
            },

            ObjectMember::ComputedMethod {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write_char('[');
                name.to_code(gen);
                gen.write("](");
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
            },
        }
    }
}

impl Code for MemberKey {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            MemberKey::Literal(ref string) => {
                gen.write_char('.');
                gen.write(string);
            },
            MemberKey::Computed(ref expr)  => {
                gen.write_char('[');
                expr.to_code(gen);
                gen.write_char(']');
            },
        }
    }
}

impl Code for Parameter {
    fn to_code(&self, gen: &mut Generator) {
        let Parameter { ref name } = *self;
        gen.write(name);
    }
}

impl Code for Expression {
    fn to_code(&self, gen: &mut Generator) {
        match *self {

            Expression::This => gen.write("this"),

            Expression::Identifier(ref ident) => gen.write(ident),

            Expression::Literal(ref literal)  => literal.to_code(gen),

            Expression::Array(ref items) => {
                gen.write_char('[');
                gen.write_list(items);
                gen.write_char(']');
            },

            Expression::Sequence(ref items) => {
                gen.write_char('(');
                gen.write_list(items);
                gen.write_char(')');
            },

            Expression::Object(ref members) => {
                gen.write_char('{');
                gen.indent();
                let mut first = true;
                for member in members {
                    if first {
                        first = false;
                    } else {
                        gen.write_char(',');
                    }
                    gen.new_line();
                    member.to_code(gen);
                }
                gen.dedent();
                gen.new_line();
                gen.write_char('}');
            },

            Expression::Member {
                ref object,
                ref property,
            } => {
                object.to_code(gen);
                property.to_code(gen);
            },

            Expression::Call {
                ref callee,
                ref arguments,
            } => {
                callee.to_code(gen);
                gen.write_char('(');
                gen.write_list(arguments);
                gen.write_char(')');
            },

            Expression::Binary {
                ref left,
                ref operator,
                ref right,
            } => {
                if left.binding_power() < self.binding_power() {
                    gen.write_char('(');
                    left.to_code(gen);
                    gen.write_char(')');
                } else {
                    left.to_code(gen);
                }
                gen.write_min(" ", "");
                operator.to_code(gen);
                gen.write_min(" ", "");
                right.to_code(gen);
            },

            Expression::Prefix {
                ref operator,
                ref operand,
            } => {
                operator.to_code(gen);
                operand.to_code(gen);
            },

            Expression::Postfix {
                ref operator,
                ref operand,
            } => {
                operand.to_code(gen);
                operator.to_code(gen);
            },

            Expression::Conditional {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                test.to_code(gen);
                gen.write_min(" ? ", "?");
                consequent.to_code(gen);
                gen.write_min(" : ", ":");
                alternate.to_code(gen);
            },

            Expression::ArrowFunction {
                ref params,
                ref body,
            } => {
                if params.len() == 1 {
                    params[0].to_code(gen);
                } else {
                    gen.write_char('(');
                    gen.write_list(params);
                    gen.write_char(')');
                }
                gen.write_min(" => ", "=>");
                match **body {
                    Statement::Expression {
                        ref value,
                    } => value.to_code(gen),
                    _ => body.to_code(gen),
                }
            },

            Expression::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.write("function");
                if let Some(ref name) = *name {
                    gen.write_char(' ');
                    gen.write(name);
                } else {
                    gen.write_min(" ", "");
                }
                gen.write_char('(');
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
            },

            // _ => gen.write_char('💀'),
        }
    }
}

impl Code for VariableDeclarationKind {
    fn to_code(&self, gen: &mut Generator) {
        gen.write(match *self {
            VariableDeclarationKind::Var   => "var",
            VariableDeclarationKind::Let   => "let",
            VariableDeclarationKind::Const => "const",
        })
    }
}

impl Code for ClassMember {
    fn to_code(&self, gen: &mut Generator) {
        match *self {

            ClassMember::Constructor {
                ref params,
                ref body,
            } => {
                gen.write("constructor(");
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
            },

            ClassMember::Method {
                is_static,
                ref name,
                ref params,
                ref body,
            } => {
                if is_static {
                    gen.write("static ");
                }
                gen.write(name);
                gen.write_char('(');
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
            },

            ClassMember::Property {
                is_static,
                ref name,
                ref value,
            } => {
                if is_static {
                    gen.write("static ");
                }
                gen.write(name);
                gen.write_min(" = ", "=");
                value.to_code(gen);
                gen.write_char(';');
            }
        }
    }
}

impl Code for VariableDeclarator {
    fn to_code(&self, gen: &mut Generator) {
        gen.write(&self.name);
        if self.value.is_some() {
            gen.write_min(" = ", "=");
            self.value.to_code(gen);
        }
    }
}

impl Code for Statement {
    fn to_code(&self, gen: &mut Generator) {
        match *self {
            Statement::Labeled {
                ref label,
                ref body,
            } => {
                gen.write(label);
                gen.write_min(": ", ":");
                body.to_code(gen);
            },

            Statement::Block {
                ref body,
            } => {
                gen.write_char('{');
                gen.write_block(body);
                gen.write_char('}');
            },

            Statement::Expression {
                ref value,
            } => {
                value.to_code(gen);
                gen.write_char(';');
            },

            Statement::Return {
                ref value,
            } => {
                gen.write("return");
                if let Some(ref value) = *value {
                    gen.write_char(' ');
                    value.to_code(gen);
                }
                gen.write_char(';');
            },

            Statement::Break {
                ref label,
            } => {
                gen.write("break");
                if let Some(ref label) = *label {
                    gen.write_char(' ');
                    gen.write(label);
                }
                gen.write_char(';');
            },

            Statement::VariableDeclaration {
                ref kind,
                ref declarators,
            } => {
                kind.to_code(gen);
                gen.write_char(' ');
                gen.write_list(declarators);
                gen.write_char(';');
            },

            Statement::Function {
                ref name,
                ref params,
                ref body,
            } => {
                gen.new_line();
                gen.write("function ");
                gen.write(name);
                gen.write_char('(');
                gen.write_list(params);
                gen.write_min(") {", "){");
                gen.write_block(body);
                gen.write_char('}');
                gen.new_line();
            },

            Statement::If {
                ref test,
                ref consequent,
                ref alternate,
            } => {
                gen.write_min("if (", "if(");
                test.to_code(gen);
                gen.write_min(") ", ")");
                consequent.to_code(gen);

                if let Some(ref alternate) = *alternate {
                    gen.write(" else ");
                    alternate.to_code(gen);
                };
            },

            Statement::While {
                ref test,
                ref body,
            } => {
                gen.write_min("while (", "while(");
                test.to_code(gen);
                gen.write_min(") ", ")");
                body.to_code(gen);
            },

            Statement::For {
                ref init,
                ref test,
                ref update,
                ref body,
            } => {
                gen.write_min("for (", "for(");
                if let Some(ref init) = *init {
                    gen.write_declaration_or_expression(init);
                }
                gen.write_min("; ", ";");
                test.to_code(gen);
                gen.write_min("; ", ";");
                update.to_code(gen);
                gen.write_min(") ", ")");
                body.to_code(gen);
            },

            Statement::ForIn {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min("for (", "for(");
                gen.write_declaration_or_expression(left);
                gen.write(" in ");
                right.to_code(gen);
                gen.write_min(") ", ")");
                body.to_code(gen);
            },

            Statement::ForOf {
                ref left,
                ref right,
                ref body,
            } => {
                gen.write_min("for (", "for(");
                gen.write_declaration_or_expression(left);
                gen.write(" of ");
                right.to_code(gen);
                gen.write_min(") ", ")");
                body.to_code(gen);
            },

            Statement::Class {
                ref name,
                ref extends,
                ref body,
            } => {
                gen.new_line();
                gen.write("class ");
                gen.write(name);
                if let &Some(ref super_class) = extends {
                    gen.write(" extends ");
                    gen.write(super_class);
                }
                gen.write_min(" {", "{");
                gen.write_block(body);
                gen.write_char('}');
                gen.new_line();
            },
        }
    }
}

pub fn generate_code(program: Program, minify: bool) -> String {
    let mut gen = Generator::new(minify);

    for statement in program.body {
        statement.to_code(&mut gen);
        gen.new_line();
    }

    gen.consume()
}
