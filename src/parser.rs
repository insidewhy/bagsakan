use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct InterfaceInfo {
    pub name: String,
    pub properties: Vec<PropertyInfo>,
    pub file_path: String,
}

pub struct PropertyInfo {
    pub name: String,
    pub type_annotation: String,
    pub optional: bool,
}

pub struct ValidatorFunction {
    pub name: String,
    pub interface_name: String,
}

pub struct TypeScriptParser {
    pub interfaces: HashMap<String, InterfaceInfo>,
    pub validator_functions: Vec<ValidatorFunction>,
    validator_pattern: Regex,
}

impl TypeScriptParser {
    pub fn new(pattern: &str) -> Self {
        Self {
            interfaces: HashMap::new(),
            validator_functions: Vec::new(),
            validator_pattern: Regex::new(pattern).unwrap(),
        }
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let source_text = fs::read_to_string(path)?;
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap_or(SourceType::default());

        let parser = Parser::new(&allocator, &source_text, source_type);
        let result = parser.parse();

        if !result.errors.is_empty() {
            eprintln!("Parse errors in {:?}: {:?}", path, result.errors);
            return Ok(());
        }

        let file_path_str = path.to_string_lossy().to_string();
        self.process_program(&result.program, &file_path_str);

        Ok(())
    }

    fn process_program(&mut self, program: &Program, file_path: &str) {
        for stmt in &program.body {
            self.process_statement(stmt, file_path);
        }
    }

    fn process_statement(&mut self, stmt: &Statement, file_path: &str) {
        match stmt {
            Statement::ExpressionStatement(expr_stmt) => {
                self.process_expression(&expr_stmt.expression);
            }
            Statement::ReturnStatement(ret_stmt) => {
                if let Some(arg) = &ret_stmt.argument {
                    self.process_expression(arg);
                }
            }
            Statement::IfStatement(if_stmt) => {
                self.process_expression(&if_stmt.test);
                self.process_statement(&if_stmt.consequent, file_path);
                if let Some(alt) = &if_stmt.alternate {
                    self.process_statement(alt, file_path);
                }
            }
            Statement::BlockStatement(block) => {
                for stmt in &block.body {
                    self.process_statement(stmt, file_path);
                }
            }
            _ => {
                if let Some(decl) = stmt.as_declaration() {
                    self.process_declaration(decl, file_path);
                } else if let Some(module_decl) = stmt.as_module_declaration() {
                    if let ModuleDeclaration::ExportNamedDeclaration(export) = module_decl {
                        if let Some(decl) = &export.declaration {
                            self.process_declaration(decl, file_path);
                        }
                    }
                }
            }
        }
    }

    fn process_declaration(&mut self, decl: &Declaration, file_path: &str) {
        match decl {
            Declaration::TSInterfaceDeclaration(interface) => {
                self.process_interface(interface, file_path);
            }
            Declaration::FunctionDeclaration(func) => {
                // Check function body for validator calls
                if let Some(body) = &func.body {
                    self.process_function_body(body);
                }
            }
            Declaration::ClassDeclaration(class) => {
                self.process_class(class);
            }
            Declaration::VariableDeclaration(var_decl) => {
                self.process_variable_declaration(var_decl);
            }
            _ => {}
        }
    }

    fn process_interface(&mut self, interface: &TSInterfaceDeclaration, file_path: &str) {
        let interface_name = interface.id.name.as_str().to_string();
        let mut properties = Vec::new();

        let body = &interface.body;
        for member in &body.body {
            if let TSSignature::TSPropertySignature(prop) = member {
                let prop_name = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => id.name.as_str().to_string(),
                    PropertyKey::Identifier(id) => id.name.as_str().to_string(),
                    _ => continue,
                };

                let type_str = if let Some(type_ann) = &prop.type_annotation {
                    get_type_string(&type_ann.type_annotation)
                } else {
                    "any".to_string()
                };

                properties.push(PropertyInfo {
                    name: prop_name,
                    type_annotation: type_str,
                    optional: prop.optional,
                });
            }
        }

        self.interfaces.insert(
            interface_name.clone(),
            InterfaceInfo {
                name: interface_name,
                properties,
                file_path: file_path.to_string(),
            },
        );
    }

    fn process_function_body(&mut self, body: &FunctionBody) {
        for stmt in &body.statements {
            self.process_statement(stmt, "");
        }
    }

    fn check_call_expression(&mut self, call: &CallExpression) {
        // Check if the callee is an identifier that matches our pattern
        match &call.callee {
            Expression::Identifier(id) => {
                let func_name = id.name.as_str();
                if let Some(captures) = self.validator_pattern.captures(func_name) {
                    if let Some(interface_name) = captures.get(1) {
                        // Found a validator function call
                        self.validator_functions.push(ValidatorFunction {
                            name: func_name.to_string(),
                            interface_name: interface_name.as_str().to_string(),
                        });
                    }
                }
            }
            _ => {
                // Debug: log other types of callees we might be missing
                if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                    match &call.callee {
                        Expression::StaticMemberExpression(member) => {
                            eprintln!("DEBUG: Static member callee: {}", member.property.name);
                        }
                        Expression::ComputedMemberExpression(_) => {
                            eprintln!("DEBUG: Computed member expression callee");
                        }
                        _ => {
                            eprintln!("DEBUG: Other callee type in call expression");
                        }
                    }
                }
            }
        }

        // Process arguments for nested calls
        for arg in &call.arguments {
            match arg {
                Argument::SpreadElement(spread) => {
                    self.process_expression(&spread.argument);
                }
                _ => {
                    if let Some(expr) = arg.as_expression() {
                        self.process_expression(expr);
                    }
                }
            }
        }
    }

    fn process_class(&mut self, class: &Class) {
        let body = &class.body;
        for member in &body.body {
            match member {
                ClassElement::MethodDefinition(method) => {
                    // Check method body for validator calls
                    if let Some(body) = &method.value.body {
                        self.process_function_body(body);
                    }
                }
                ClassElement::PropertyDefinition(prop) => {
                    if let Some(value) = &prop.value {
                        self.process_expression(value);
                    }
                }
                _ => {}
            }
        }
    }

    fn process_variable_declaration(&mut self, var_decl: &VariableDeclaration) {
        for decl in &var_decl.declarations {
            if let VariableDeclarator {
                init: Some(init), ..
            } = &decl
            {
                self.process_expression(init);
            }
        }
    }

    fn process_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::FunctionExpression(func) => {
                // Check function body for validator calls
                if let Some(body) = &func.body {
                    self.process_function_body(body);
                }
            }
            Expression::ArrowFunctionExpression(arrow) => {
                // Check arrow function body for validator calls
                self.process_function_body(&arrow.body);
            }
            Expression::CallExpression(call) => {
                self.check_call_expression(call);
            }
            Expression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop {
                        self.process_expression(&obj_prop.value);
                    }
                }
            }
            Expression::UnaryExpression(unary) => {
                // Handle expressions like !validateUser(data)
                self.process_expression(&unary.argument);
            }
            Expression::BinaryExpression(binary) => {
                // Handle expressions like validateUser(data) && other
                self.process_expression(&binary.left);
                self.process_expression(&binary.right);
            }
            Expression::LogicalExpression(logical) => {
                // Handle expressions like validateUser(data) || validateProduct(data)
                self.process_expression(&logical.left);
                self.process_expression(&logical.right);
            }
            _ => {}
        }
    }
}

fn get_type_string(ts_type: &TSType) -> String {
    match ts_type {
        TSType::TSStringKeyword(_) => "string".to_string(),
        TSType::TSNumberKeyword(_) => "number".to_string(),
        TSType::TSBooleanKeyword(_) => "boolean".to_string(),
        TSType::TSAnyKeyword(_) => "any".to_string(),
        TSType::TSVoidKeyword(_) => "void".to_string(),
        TSType::TSNullKeyword(_) => "null".to_string(),
        TSType::TSUndefinedKeyword(_) => "undefined".to_string(),
        TSType::TSArrayType(arr) => format!("{}[]", get_type_string(&arr.element_type)),
        TSType::TSUnionType(union) => {
            let types: Vec<String> = union.types.iter().map(|t| get_type_string(t)).collect();
            types.join(" | ")
        }
        TSType::TSLiteralType(lit) => match &lit.literal {
            TSLiteral::StringLiteral(s) => format!("'{}'", s.value.as_str()),
            TSLiteral::NumericLiteral(n) => n.value.to_string(),
            TSLiteral::BooleanLiteral(b) => b.value.to_string(),
            _ => "unknown".to_string(),
        },
        TSType::TSTypeReference(type_ref) => {
            if let TSTypeName::IdentifierReference(id) = &type_ref.type_name {
                id.name.as_str().to_string()
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}
