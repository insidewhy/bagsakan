use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_resolver::{ResolveOptions, Resolver};
use oxc_span::SourceType;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

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

pub struct EnumInfo {
    pub members: Vec<EnumMember>,
}

pub struct EnumMember {
    pub value: EnumValue,
}

pub enum EnumValue {
    String(String),
    Number(f64),
    Computed,
}

pub struct TypeScriptParser {
    pub interfaces: HashMap<String, InterfaceInfo>,
    pub enums: HashMap<String, EnumInfo>,
    pub validator_functions: Vec<ValidatorFunction>,
    validator_pattern: Regex,
    parsed_files: HashSet<PathBuf>,
    source_files: HashSet<PathBuf>,
    current_file_is_source: bool,
    resolver: Resolver,
    follow_external_imports: bool,
    exclude_packages: Vec<String>,
}

impl TypeScriptParser {
    pub fn new(
        pattern: &str,
        follow_external_imports: bool,
        exclude_packages: Vec<String>,
        export_conditions: Vec<String>,
    ) -> Self {
        let mut resolve_options = ResolveOptions::default();

        // Configure for TypeScript resolution
        resolve_options.extensions = vec![
            ".ts".to_string(),
            ".tsx".to_string(),
            ".d.ts".to_string(),
            ".js".to_string(),
            ".jsx".to_string(),
            ".json".to_string(),
        ];

        // Enable TypeScript mode for proper .js -> .ts resolution
        resolve_options.extension_alias = vec![
            (
                ".js".to_string(),
                vec![".ts".to_string(), ".tsx".to_string(), ".js".to_string()],
            ),
            (
                ".jsx".to_string(),
                vec![".tsx".to_string(), ".jsx".to_string()],
            ),
            (
                ".mjs".to_string(),
                vec![".mts".to_string(), ".mjs".to_string()],
            ),
            (
                ".cjs".to_string(),
                vec![".cts".to_string(), ".cjs".to_string()],
            ),
        ];

        // Enable exports field support
        resolve_options.exports_fields = vec![vec!["exports".to_string()]];

        // Set main fields for module resolution
        resolve_options.main_fields = vec![
            "types".to_string(),
            "typings".to_string(),
            "module".to_string(),
            "main".to_string(),
        ];

        // Enable resolving index files
        resolve_options.main_files = vec!["index".to_string()];

        // Prefer relative imports to resolve as-is
        resolve_options.prefer_relative = true;

        // Set export conditions (e.g., "dev", "production", "import", "require")

        if !export_conditions.is_empty() {
            // Add custom conditions first, then default ones
            let mut conditions = export_conditions;
            conditions.push("types".to_string());
            conditions.push("import".to_string());
            conditions.push("node".to_string());
            conditions.push("default".to_string());
            resolve_options.condition_names = conditions;
        } else {
            // Default conditions for TypeScript/Node.js
            resolve_options.condition_names = vec![
                "types".to_string(),
                "import".to_string(),
                "node".to_string(),
                "default".to_string(),
            ];
        }

        Self {
            interfaces: HashMap::new(),
            enums: HashMap::new(),
            validator_functions: Vec::new(),
            validator_pattern: Regex::new(pattern).unwrap(),
            parsed_files: HashSet::new(),
            source_files: HashSet::new(),
            current_file_is_source: false,
            resolver: Resolver::new(resolve_options),
            follow_external_imports,
            exclude_packages,
        }
    }

    pub fn mark_as_source_file(&mut self, path: &Path) {
        if let Ok(canonical_path) = path.canonicalize() {
            self.source_files.insert(canonical_path);
        }
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Get canonical path to avoid parsing the same file twice
        let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Skip if already parsed
        if self.parsed_files.contains(&canonical_path) {
            return Ok(());
        }

        self.parsed_files.insert(canonical_path.clone());

        // Check if this is a source file
        self.current_file_is_source = self.source_files.contains(&canonical_path);

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

        // Collect imports before processing the program
        let imports = self.collect_imports(&result.program);

        if std::env::var("BAGSAKAN_DEBUG").is_ok() && !imports.is_empty() {
            eprintln!("Found imports in {:?}: {:?}", path, imports);
        }

        self.process_program(&result.program, &file_path_str);

        // Parse imported files
        for import_path in imports {
            match self.resolve_import(path, &import_path) {
                Ok(resolved_path) => {
                    if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                        eprintln!("Resolved '{}' to {:?}", import_path, resolved_path);
                    }
                    let _ = self.parse_file(&resolved_path);
                }
                Err(e) => {
                    let error_msg = e.to_string();

                    // Provide helpful error messages
                    if error_msg.contains("node_modules without .d.ts") {
                        eprintln!("Warning: No TypeScript definitions found for '{}'. Consider installing @types package.", import_path);
                    } else if error_msg.contains("Package") && error_msg.contains("excluded") {
                        // Silently skip excluded packages
                    } else if error_msg.contains("External imports are disabled") {
                        // Silently skip when external imports are disabled
                    } else if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                        eprintln!(
                            "Failed to resolve import '{}' from {:?}: {}",
                            import_path, path, e
                        );

                        // Provide suggestions
                        if !import_path.starts_with(".") {
                            eprintln!("  Hint: Make sure the package is installed in node_modules");
                            if !import_path.starts_with("@types/") {
                                eprintln!(
                                    "  Hint: Try installing @types/{} if it's a JavaScript package",
                                    import_path.split('/').next().unwrap_or(&import_path)
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn collect_imports(&self, program: &Program) -> Vec<String> {
        let mut imports = Vec::new();

        for stmt in &program.body {
            if let Some(module_decl) = stmt.as_module_declaration() {
                match module_decl {
                    ModuleDeclaration::ImportDeclaration(import) => {
                        imports.push(import.source.value.as_str().to_string());
                    }
                    ModuleDeclaration::ExportNamedDeclaration(export) => {
                        if let Some(source) = &export.source {
                            imports.push(source.value.as_str().to_string());
                        }
                    }
                    ModuleDeclaration::ExportAllDeclaration(export) => {
                        imports.push(export.source.value.as_str().to_string());
                    }
                    _ => {}
                }
            }
        }

        imports
    }

    pub fn resolve_import(
        &self,
        current_file: &Path,
        import_path: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let current_dir = current_file.parent().ok_or("No parent directory")?;
        // oxc_resolver needs absolute paths to work correctly
        let current_dir = current_dir
            .canonicalize()
            .unwrap_or_else(|_| current_dir.to_path_buf());

        // Check if we should follow external imports
        if !import_path.starts_with(".") && !import_path.starts_with("/") {
            if !self.follow_external_imports {
                return Err("External imports are disabled".into());
            }

            // Check if package is excluded
            let package_name = if import_path.contains('/') {
                import_path.split('/').next().unwrap_or("")
            } else {
                import_path
            };

            if self.exclude_packages.iter().any(|excluded| {
                package_name == excluded || import_path.starts_with(&format!("{}/", excluded))
            }) {
                return Err(format!("Package '{}' is excluded", package_name).into());
            }
        }

        // Use oxc_resolver to resolve the import
        if std::env::var("BAGSAKAN_DEBUG").is_ok() {
            eprintln!(
                "Attempting to resolve '{}' from {:?}",
                import_path, current_dir
            );
        }

        match self.resolver.resolve(current_dir, import_path) {
            Ok(resolution) => {
                let path = resolution.into_path_buf();

                if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                    eprintln!("  oxc_resolver found: {:?}", path);
                }

                Ok(path)
            }
            Err(e) => {
                if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                    eprintln!("  oxc_resolver error: {:?}", e);
                }
                Err(format!("Failed to resolve '{}': {:?}", import_path, e).into())
            }
        }
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
                    match module_decl {
                        ModuleDeclaration::ExportNamedDeclaration(export) => {
                            if let Some(decl) = &export.declaration {
                                self.process_declaration(decl, file_path);
                            }

                            // Debug: log export specifiers
                            if std::env::var("BAGSAKAN_DEBUG").is_ok()
                                && !export.specifiers.is_empty()
                            {
                                for _spec in &export.specifiers {
                                    eprintln!("Export specifier found in {}", file_path);
                                }
                            }
                        }
                        ModuleDeclaration::ExportDefaultDeclaration(_export) => {
                            // Handle default exports if needed
                            if std::env::var("BAGSAKAN_DEBUG").is_ok() {
                                eprintln!("Default export found in {}", file_path);
                            }
                        }
                        _ => {}
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
            Declaration::TSEnumDeclaration(enum_decl) => {
                self.process_enum(enum_decl);
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

    fn process_enum(&mut self, enum_decl: &TSEnumDeclaration) {
        let enum_name = enum_decl.id.name.as_str().to_string();
        let mut members = Vec::new();
        let mut next_numeric_value = 0.0;

        for member in &enum_decl.body.members {
            let value = if let Some(init) = &member.initializer {
                match init {
                    Expression::StringLiteral(lit) => {
                        EnumValue::String(lit.value.as_str().to_string())
                    }
                    Expression::NumericLiteral(lit) => {
                        next_numeric_value = lit.value + 1.0;
                        EnumValue::Number(lit.value)
                    }
                    _ => EnumValue::Computed,
                }
            } else {
                // For numeric enums without initializers, use auto-increment
                let current_value = next_numeric_value;
                next_numeric_value += 1.0;
                EnumValue::Number(current_value)
            };

            members.push(EnumMember { value });
        }

        self.enums.insert(enum_name, EnumInfo { members });
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
                        // Only collect validator functions from source files
                        if self.current_file_is_source {
                            self.validator_functions.push(ValidatorFunction {
                                name: func_name.to_string(),
                                interface_name: interface_name.as_str().to_string(),
                            });
                        }
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
                let base_type = id.name.as_str();

                // Handle generic types with type arguments
                if let Some(type_args) = &type_ref.type_arguments {
                    let arg_types: Vec<String> = type_args
                        .params
                        .iter()
                        .map(|param| get_type_string(param))
                        .collect();
                    format!("{}<{}>", base_type, arg_types.join(", "))
                } else {
                    base_type.to_string()
                }
            } else {
                "unknown".to_string()
            }
        }
        _ => "unknown".to_string(),
    }
}
