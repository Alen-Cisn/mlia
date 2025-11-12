# Documentación Completa del Compilador MLIA

## Índice

1. [Introducción al Compilador MLIA](#introducción-al-compilador-mlia)
2. [Teoría General de Compiladores](#teoría-general-de-compiladores)
3. [Arquitectura del Compilador MLIA](#arquitectura-del-compilador-mlia)
4. [Fase 1: Análisis Léxico (Tokenizador)](#fase-1-análisis-léxico-tokenizador)
5. [Fase 2: Análisis Sintáctico (Parser)](#fase-2-análisis-sintáctico-parser)
6. [Fase 3: Generación de Código (CodeGen)](#fase-3-generación-de-código-codegen)
7. [El Lenguaje MLIA](#el-lenguaje-mlia)
8. [Flujo de Compilación Completo](#flujo-de-compilación-completo)
9. [Ejemplos Prácticos](#ejemplos-prácticos)
10. [Conceptos Avanzados](#conceptos-avanzados)

---

## Introducción al Compilador MLIA

MLIA es un compilador moderno escrito en Rust que demuestra los principios fundamentales de construcción de compiladores. Este compilador implementa un lenguaje funcional simple con características como:

- **Declaraciones locales** con la palabra clave `decl`
- **Secuencias de expresiones** separadas por punto y coma
- **Funciones de impresión** para salida
- **Compilación a código nativo** usando LLVM

### Dependencias Principales

El compilador utiliza tres librerías clave:

- **`pomelo`**: Generador de parsers LR(1) para Rust
- **`inkwell`**: Bindings de Rust para LLVM
- **`lazy_static`**: Para inicialización estática de estructuras de datos

---

## Teoría General de Compiladores

### ¿Qué es un Compilador?

Un **compilador** es un programa que traduce código fuente escrito en un lenguaje de alto nivel a código en un lenguaje de bajo nivel (típicamente código máquina o código intermedio). Es esencialmente un traductor sofisticado que no solo convierte sintaxis, sino que también:

1. **Analiza** la estructura del programa
2. **Verifica** que el código sea válido sintáctica y semánticamente
3. **Optimiza** el código para mejor rendimiento
4. **Genera** código ejecutable eficiente

### Las Tres Fases Fundamentales

Todo compilador moderno se estructura en tres fases principales:

#### 1. Frontend (Análisis)

- **Análisis Léxico**: Convierte el texto fuente en tokens
- **Análisis Sintáctico**: Organiza tokens en un árbol de sintaxis abstracta (AST)
- **Análisis Semántico**: Verifica tipos y semántica del programa

#### 2. Middle-end (Optimización)

- **Representación Intermedia**: Convierte el AST a una forma intermedia
- **Optimizaciones**: Mejora el código sin cambiar su comportamiento

#### 3. Backend (Generación de Código)

- **Generación de código**: Produce código máquina o código intermedio
- **Optimizaciones de bajo nivel**: Específicas para la arquitectura objetivo

### Árboles de Sintaxis Abstracta (AST)

Un **AST** es una representación estructurada del código fuente que:

- Elimina detalles sintácticos irrelevantes (paréntesis, espacios)
- Preserva la estructura jerárquica del programa
- Facilita el análisis y transformación del código

**Ejemplo**: La expresión `decl x <- 5 in x + 2` se representa como:

```mlia
Decl
├── variable: "x"
├── valor: Number(5)
└── cuerpo: Call("+", [Ident("x"), Number(2)])
```

---

## Arquitectura del Compilador MLIA

El compilador MLIA sigue una arquitectura de **pipeline clásico** con tres módulos principales:

### Estructura de Archivos

```bash
src/
├── main.rs          # Punto de entrada y coordinación
├── tokenizer.rs     # Análisis léxico (lexer)
├── parser.rs        # Análisis sintáctico (parser)
└── codegen.rs       # Generación de código LLVM
```

### Flujo de Datos

```txt
Código MLIA → Tokenizador → Parser → CodeGen → Ejecutable
    ↓             ↓           ↓        ↓
  String      Vec<Token>    AST    LLVM IR
```

### Módulo Principal (`main.rs`)

El archivo `main.rs` actúa como **orquestador** del proceso de compilación:

```rust
// Flujo principal de compilación
let source_code = fs::read_to_string(input_file)?;  // 1. Leer archivo
let ast = parse_program(source_code)?;              // 2. Parsear
let mut codegen = CodeGen::new(&context)?;          // 3. Inicializar generador
codegen.compile_to_executable(&ast, &output_path)?; // 4. Compilar
```

El programa principal también maneja:

- **Argumentos de línea de comandos** (archivos de entrada/salida)
- **Modos de ejecución** (JIT vs compilación a ejecutable)
- **Manejo de errores** y reportes detallados

---

## Fase 1: Análisis Léxico (Tokenizador)

### Propósito del Análisis Léxico

El **lexer** (analizador léxico) es la primera fase del compilador. Su trabajo es:

1. **Leer** el código fuente carácter por carácter
2. **Agrupar** caracteres en unidades significativas llamadas **tokens**
3. **Clasificar** cada token según su tipo (número, identificador, operador, etc.)
4. **Filtrar** elementos irrelevantes (espacios en blanco, comentarios)

### Implementación con Autómata Finito

El tokenizador MLIA implementa un **autómata finito determinista (DFA)** para reconocer tokens:

#### Estados del Autómata

```rust
pub enum State {
    Start = 0,                             // Estado inicial
    Digit = 1,                             // Reconociendo números
    PipeOrIdentifier = 2,                  // Pipe (|) o identificador
    AssignOrIdentifier = 3,                // Asignación (<-) o identificador
    FinishAssignOrIdentifier = 4,          // Completando <-
    Identifier = 5,                        // Identificadores generales
    FinishArrowOrIdentifier = 6,           // Completando ->
    ArrowOrIdentifierOrNegativeNumber = 7, // Flecha, identificador o número negativo
    ParenLOrComment = 8,                   // Paréntesis o inicio de comentario
    Comment = 9,                           // Dentro de comentario
    MayFinishComment = 10,                 // Posible fin de comentario
    ParenR = 11,                           // Paréntesis derecho
}
```

#### Clasificación de Caracteres

Cada carácter se clasifica en una **clase de caracteres**:

```rust
pub enum CharClass {
    Digit = 0,       // 0-9
    LowerAlpha = 1,  // a-z (incluye Unicode)
    UpperAlpha = 2,  // A-Z (incluye Unicode)
    Less = 3,        // <
    Greater = 4,     // >
    Minus = 5,       // -
    // ... más clases
}
```

#### Tabla de Transiciones

La **tabla de transiciones** define cómo cambiar de estado:

```rust
// Ejemplo simplificado de transiciones desde el estado Start
pub const STATE_TRANSITIONS: [[i8; NUM_CLASSES]; NUM_STATES] = [
    // Estado Start: [Digit, LowerAlpha, UpperAlpha, Less, Greater, ...]
    [1, 5, 3, 3, 7, 7, 5, 5, 5, 5, 5, 5, 5, 2, 2, 8, 11, 0, 0, -1],
    // ... más estados
];
```

Donde:

- **Números positivos**: Nuevo estado
- **-1**: Transición inválida (error)
- **-2**: Carácter no permitido

### Tipos de Tokens

El tokenizador produce diferentes tipos de tokens:

```rust
pub enum Token {
    // Literales
    IntegerLiteral(i64),    // 42, -10, 0

    // Identificadores
    Identifier(String),      // variables, funciones

    // Palabras clave
    Decl,                   // decl
    In,                     // in
    While,                  // while
    Do,                     // do
    Done,                   // done

    // Operadores
    Assign,                 // <-
    Arrow,                  // ->
    Plus,                   // +
    Minus,                  // -

    // Delimitadores
    ParenL,                 // (
    ParenR,                 // )
    Semicolon,              // ;

    // Especiales
    Eof,                    // Fin de archivo
}
```

### Manejo de Comentarios

Los comentarios en MLIA son **anidados** estilo ML: `(* comentario *)`

El algoritmo para manejar comentarios:

1. Al ver `(`, verificar si el siguiente carácter es `*`
2. Si es así, entrar en modo comentario
3. Dentro del comentario, ignorar todos los caracteres excepto `*`
4. Al ver `*`, verificar si el siguiente es `)`
5. Si es así, terminar el comentario y volver al estado normal

### Características Especiales

#### Soporte Unicode

El tokenizador soporta identificadores con caracteres Unicode:

```rust
pub const fn classify_char(c: char) -> Option<CharClass> {
    match c {
        'a'..='z' | '\u{00DF}'..='\u{00F6}' | '\u{00F8}'..='\u{00FF}' => Some(LowerAlpha),
        'A'..='Z' | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00DE}' => Some(UpperAlpha),
        // ...
    }
}
```

Esto permite variables con nombres como: `ñ`, `café`, `número`

#### Operadores Compuestos

El tokenizador maneja operadores de múltiples caracteres:

- `<-` (asignación)
- `->` (flecha en pattern matching)
- `!=` (no igual)

### Algoritmo de Tokenización

```rust
pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = self.input.chars().collect();
    let mut state = State::Start;

    for &c in &chars {
        let class = classify_char(c)?;
        let next_state = next_state(state, class)?;

        if let Some(next) = next_state {
            // Ejecutar acción de transición
            self.execute_action(state, class, c);
            state = next;
        } else {
            // No hay transición: finalizar token actual
            self.finalize_current_token(state)?;
            state = State::Start;
            // Reprocesar carácter actual
        }
    }

    // Finalizar último token
    self.finalize_current_token(state)?;
    Ok(self.tokens)
}
```

---

## Fase 2: Análisis Sintáctico (Parser)

### Propósito del Análisis Sintáctico

El **parser** (analizador sintáctico) toma la secuencia de tokens del lexer y construye un **Árbol de Sintaxis Abstracta (AST)** que representa la estructura jerárquica del programa.

### Gramática del Lenguaje MLIA

El parser de MLIA implementa la siguiente gramática (en notación BNF):

```bnf
programa ::= expresión

expresión ::= "decl" identificador "<-" expresión "in" expresión
           | expresión_secuencia

expresión_secuencia ::= expresión_secuencia ";" expresión_asignación
                     | expresión_asignación

expresión_asignación ::= identificador "<-" expresión_asignación
                      | expresión_llamada

expresión_llamada ::= identificador expresión_atómica
                   | "print" expresión_atómica
                   | expresión_atómica

expresión_atómica ::= literal_entero
                   | identificador
                   | "(" expresión ")"
```

### Representación del AST

El AST se define con un enum recursivo:

```rust
#[derive(Debug, Clone)]
pub enum Expr {
    Number(i64),                                    // 42
    Ident(String),                                  // variable
    Call(String, Vec<Expr>),                        // print x
    Seq(Box<Expr>, Box<Expr>),                      // expr1; expr2
    Assign(String, Box<Expr>),                      // x <- 5
    Decl(String, Vec<String>, Box<Expr>, Box<Expr>), // decl x <- 5 in x
}
```

Cada variante representa un tipo diferente de expresión:

- **`Number`**: Literales numéricos
- **`Ident`**: Identificadores (variables)
- **`Call`**: Llamadas a funciones
- **`Seq`**: Secuencias de expresiones
- **`Assign`**: Asignaciones a variables
- **`Decl`**: Declaraciones de variables con alcance

### Parser LR(1) con Pomelo

MLIA utiliza la librería **Pomelo** que genera un parser **LR(1)** automáticamente:

#### ¿Qué es LR(1)?

- **L**: Lee de izquierda a derecha (**L**eft-to-right)
- **R**: Construye derivaciones por la derecha (**R**ightmost derivation in reverse)
- **1**: Usa 1 token de lookahead

Los parsers LR(1) son:

- **Deterministas**: No hay ambigüedad en las decisiones
- **Eficientes**: O(n) en tiempo y espacio
- **Potentes**: Pueden manejar una gran clase de gramáticas

#### Definición con Pomelo

```rust
pomelo! {
    %token #[derive(Debug, Clone, PartialEq)] pub enum Token {};

    %type expr Expr;
    %start_symbol program;

    // Reglas de la gramática
    program ::= expr(e) { e }

    expr ::= Decl Identifier(var) Assign assign_expr(val) In expr(body) {
        Expr::Decl(var, vec![], Box::new(val), Box::new(body))
    }

    seq_expr ::= seq_expr(first) Semicolon assign_expr(second) {
        Expr::Seq(Box::new(first), Box::new(second))
    }

    // ... más reglas
}
```

#### Acciones Semánticas

Cada regla de gramática incluye una **acción semántica** que construye el nodo AST correspondiente:

```rust
// Regla: expr ::= Decl Identifier Assign expr In expr
expr ::= Decl Identifier(var) Assign assign_expr(val) In expr(body) {
    // Acción semántica: construir nodo Decl
    Expr::Decl(var, vec![], Box::new(val), Box::new(body))
}
```

### Precedencia y Asociatividad

La gramática MLIA maneja precedencia implícitamente a través de la estructura de reglas:

1. **Declaraciones** (`decl`) - Precedencia más baja
2. **Secuencias** (`;`) - Precedencia media-baja
3. **Asignaciones** (`<-`) - Precedencia media
4. **Llamadas a función** - Precedencia media-alta
5. **Expresiones atómicas** - Precedencia más alta

### Análisis Sintáctico Paso a Paso

Ejemplo: Parsing de `decl x <- 5 in x`

#### 1. Tokens de Entrada

```txt
[Decl, Identifier("x"), Assign, IntegerLiteral(5), In, Identifier("x"), Eof]
```

#### 2. Proceso de Parsing

| Paso | Pila | Entrada | Acción |
|------|------|---------|---------|
| 1 | [] | [Decl, ...] | Shift Decl |
| 2 | [Decl] | [Identifier("x"), ...] | Shift Identifier |
| 3 | [Decl, Identifier("x")] | [Assign, ...] | Shift Assign |
| 4 | [Decl, Identifier("x"), Assign] | [IntegerLiteral(5), ...] | Reduce: expr → IntegerLiteral |
| 5 | [Decl, Identifier("x"), Assign, expr] | [In, ...] | Shift In |
| 6 | [Decl, Identifier("x"), Assign, expr, In] | [Identifier("x"), ...] | Reduce: expr → Identifier |
| 7 | [Decl, Identifier("x"), Assign, expr, In, expr] | [Eof] | Reduce: expr → Decl ... |

#### 3. AST Resultante

```
Decl {
    variable: "x",
    parámetros: [],
    valor: Number(5),
    cuerpo: Ident("x")
}
```

### Manejo de Errores

El parser reporta errores detallados:

```rust
pub fn parse_program(input: String) -> Result<Expr, String> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = parser::Parser::new();

    for (i, token) in tokens.iter().enumerate() {
        if let Err(e) = parser.parse(token.clone()) {
            return Err(format!("Error de parsing en token {}: {:?}, error: {:?}", i, token, e));
        }
    }

    parser.end_of_input()
        .map_err(|e| format!("Error de parsing al final: {:?}", e))
}
```

---

## Fase 3: Generación de Código (CodeGen)

### Propósito de la Generación de Código

El **generador de código** toma el AST y produce código ejecutable. MLIA genera **LLVM IR** (Representación Intermedia de LLVM), que luego se compila a código máquina nativo.

### ¿Por qué LLVM?

**LLVM** (Low Level Virtual Machine) es una infraestructura de compilación moderna que proporciona:

- **Representación intermedia independiente de arquitectura**
- **Optimizaciones avanzadas** automáticas
- **Soporte para múltiples arquitecturas** (x86, ARM, etc.)
- **JIT compilation** para ejecución inmediata
- **Herramientas maduras** y bien documentadas

### Estructura del Generador de Código

```rust
pub struct CodeGen<'ctx> {
    context: &'ctx Context,              // Contexto LLVM
    module: Module<'ctx>,                // Módulo LLVM (unidad de compilación)
    builder: Builder<'ctx>,              // Constructor de instrucciones
    execution_engine: ExecutionEngine<'ctx>, // Motor de ejecución JIT

    variables: HashMap<String, PointerValue<'ctx>>, // Tabla de símbolos
    current_function: Option<FunctionValue<'ctx>>,  // Función actual
    print_function: Option<FunctionValue<'ctx>>,    // Función printf externa
}
```

### Conceptos Clave de LLVM IR

#### 1. Módulos

Un **módulo** es la unidad básica de compilación en LLVM. Contiene:

- Funciones
- Variables globales
- Declaraciones de funciones externas
- Metadatos

#### 2. Funciones

Las **funciones** en LLVM tienen:

- **Tipo de función** (parámetros y valor de retorno)
- **Bloques básicos** (secuencias de instrucciones sin saltos)
- **Instrucciones** dentro de cada bloque

#### 3. Tipos de Datos

MLIA usa principalmente:

- **`i64`**: Enteros de 64 bits
- **`i8*`**: Punteros a cadenas (para printf)
- **Punteros**: Para variables locales en la pila

#### 4. Valores (Values)

Todo en LLVM IR es un **valor**:

- **Constantes**: `42`, `"Hello"`
- **Instrucciones**: resultado de operaciones
- **Argumentos de función**: parámetros

### Generación de Código por Tipo de Expresión

#### 1. Números (`Expr::Number`)

```rust
fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, &'static str> {
    match expr {
        Expr::Number(n) => {
            // Crear constante entera de 64 bits
            Ok(self.context.i64_type().const_int(*n as u64, true))
        }
        // ...
    }
}
```

**LLVM IR generado**:

```llvm
; Para el número 42
%1 = i64 42
```

#### 2. Variables (`Expr::Ident`)

```rust
Expr::Ident(name) => {
    match self.variables.get(name) {
        Some(var_ptr) => {
            // Cargar valor desde la pila
            Ok(self.build_load(*var_ptr, name))
        }
        None => Err("Variable no definida"),
    }
}
```

**LLVM IR generado**:

```llvm
; Cargar variable 'x'
%2 = load i64, ptr %x_ptr
```

#### 3. Declaraciones (`Expr::Decl`)

Las declaraciones crean variables en la pila:

```rust
Expr::Decl(var_name, _params, value, body) => {
    // 1. Compilar valor inicial
    let val = self.compile_expr(value)?;

    // 2. Crear espacio en la pila
    let alloca = self.create_entry_block_alloca(var_name);

    // 3. Almacenar valor inicial
    self.builder.build_store(alloca, val)?;

    // 4. Agregar variable al scope
    let old_binding = self.variables.insert(var_name.clone(), alloca);

    // 5. Compilar cuerpo con nueva variable
    let result = self.compile_expr(body);

    // 6. Restaurar scope anterior
    match old_binding {
        Some(old) => { self.variables.insert(var_name.clone(), old); }
        None => { self.variables.remove(var_name); }
    }

    result
}
```

**LLVM IR generado**:

```llvm
; decl x <- 5 in ...
%x_ptr = alloca i64          ; reservar espacio en la pila
store i64 5, ptr %x_ptr      ; guardar valor inicial
; ... código del cuerpo ...
```

#### 4. Secuencias (`Expr::Seq`)

```rust
Expr::Seq(first, second) => {
    // Compilar primera expresión (descartar resultado)
    self.compile_expr(first)?;
    // Compilar y retornar segunda expresión
    self.compile_expr(second)
}
```

#### 5. Asignaciones (`Expr::Assign`)

```rust
Expr::Assign(var_name, value) => {
    let val = self.compile_expr(value)?;

    match self.variables.get(var_name) {
        Some(var_ptr) => {
            self.builder.build_store(*var_ptr, val)?;
            Ok(val)
        }
        None => Err("No se puede asignar a variable no definida"),
    }
}
```

#### 6. Llamadas a Función (`Expr::Call`)

Actualmente solo soporta `print`:

```rust
Expr::Call(func_name, args) if func_name == "print" => {
    let arg_val = self.compile_expr(&args[0])?;

    // Crear cadena de formato para printf
    let format_str = self.builder
        .build_global_string_ptr("%lld\n", "fmt_str")?;

    // Llamar a printf
    let printf_fn = self.print_function.ok_or("Función print no disponible")?;
    self.builder.build_call(
        printf_fn,
        &[format_str.as_pointer_value().into(), arg_val.into()],
        "printf_call"
    )?;

    Ok(arg_val)
}
```

**LLVM IR generado**:

```llvm
; print 42
@fmt_str = private constant [6 x i8] c"%lld\12\00"
%printf_result = call i32 @printf(ptr @fmt_str, i64 42)
```

### Manejo de Alcance (Scoping)

MLIA implementa **alcance léxico** usando una tabla de símbolos:

```rust
// Al entrar en un nuevo scope
let old_binding = self.variables.insert(var_name.clone(), new_var);

// Al salir del scope
match old_binding {
    Some(old_var) => self.variables.insert(var_name, old_var), // Restaurar
    None => self.variables.remove(&var_name),                  // Eliminar
}
```

### Compilación a Ejecutable

El proceso completo incluye:

1. **Generar LLVM IR** desde el AST
2. **Verificar** la función generada
3. **Crear target machine** para la arquitectura objetivo
4. **Generar archivo objeto** (.o)
5. **Enlazar** con GCC para crear ejecutable

```rust
pub fn compile_to_executable(&mut self, expr: &Expr, output_path: &str) -> Result<(), Box<dyn Error>> {
    // 1. Crear función main
    let main_function = self.create_main_function();

    // 2. Compilar expresión
    let result = self.compile_expr(expr)?;
    self.builder.build_return(Some(&result))?;

    // 3. Verificar función
    main_function.verify(true);

    // 4. Generar archivo objeto
    let target_machine = self.create_target_machine()?;
    target_machine.write_to_file(&self.module, FileType::Object, Path::new(&obj_path))?;

    // 5. Enlazar con GCC
    std::process::Command::new("gcc")
        .args(&[&obj_path, "-o", output_path])
        .output()?;
}
```

### Ejecución JIT

Para ejecución inmediata, MLIA usa el **motor de ejecución JIT**:

```rust
pub fn execute_program(&mut self, expr: &Expr) -> Result<i64, Box<dyn Error>> {
    let main_func = self.compile_program(expr)?;

    unsafe {
        let result = main_func.call(); // ¡Ejecutar inmediatamente!
        Ok(result)
    }
}
```

---

## El Lenguaje MLIA

### Características del Lenguaje

MLIA es un **lenguaje funcional minimal** con las siguientes características:

#### 1. **Variables Inmutables**

```mlia
decl x <- 42 in x    (* x no puede cambiar después de la declaración *)
```

#### 2. **Alcance Léxico**

```mlia
decl x <- 1 in
  decl x <- 2 in
    print x        (* imprime 2 *)
  end;
  print x          (* imprime 1 *)
```

#### 3. **Expresiones como Valores**

Todo en MLIA es una expresión que retorna un valor:

```mlia
decl resultado <- (
  print 42;
  100              (* valor retornado *)
) in resultado
```

#### 4. **Secuencias de Expresiones**

```mlia
print 1;
print 2;
print 3;
0                  (* valor final del programa *)
```

#### 5. **Comentarios Anidados**

```mlia
(*
  Comentario principal
  (* comentario anidado *)
  más texto
*)
```

### Gramática Completa

```bnf
programa ::= expresión

expresión ::= declaración
           | secuencia

declaración ::= "decl" identificador "<-" expresión "in" expresión

secuencia ::= expresión ";" expresión
           | asignación

asignación ::= identificador "<-" expresión
            | llamada

llamada ::= identificador expresión
         | "print" expresión
         | atómica

atómica ::= entero
         | identificador
         | "(" expresión ")"

entero ::= ["-"] dígito {dígito}
identificador ::= letra {letra | dígito | símbolo}
```

### Semántica del Lenguaje

#### 1. **Evaluación de Expresiones**

- Las expresiones se evalúan de **izquierda a derecha**
- El valor de una secuencia es el valor de la **última expresión**
- Las declaraciones introducen una **nueva variable en scope**

#### 2. **Modelo de Memoria**

- Variables se almacenan en la **pila**
- No hay **heap allocation** (no hay objetos dinámicos)
- **Gestión automática** de memoria por LLVM

#### 3. **Sistema de Tipos**

- **Monotipos**: Solo enteros de 64 bits
- **Sin inferencia de tipos**: Todos los valores son enteros
- **Sin verificación estática**: Errores en tiempo de ejecución

### Ejemplos de Programas

#### Programa Simple

```mlia
(* Declarar variable y usarla *)
decl x <- 42 in print x
```

#### Programa con Secuencias

```mlia
(* Múltiples declaraciones y prints *)
decl a <- 2 in
decl b <- 3 in
print b;
print a;
0
```

#### Programa con Shadowing

```mlia
(* Sombreado de variables *)
decl x <- 1 in
  print x;           (* imprime 1 *)
  decl x <- 2 in
    print x;         (* imprime 2 *)
  print x            (* imprime 1 otra vez *)
```

---

## Flujo de Compilación Completo

### Visión General del Pipeline

```mermaid
graph LR
    A[Código MLIA] --> B[Tokenizador]
    B --> C[Lista de Tokens]
    C --> D[Parser LR(1)]
    D --> E[AST]
    E --> F[Generador de Código]
    F --> G[LLVM IR]
    G --> H[Optimizador LLVM]
    H --> I[Código Objeto]
    I --> J[Enlazador]
    J --> K[Ejecutable]
```

### Paso a Paso Detallado

#### Entrada: Programa MLIA

```mlia
decl x <- 42 in print x
```

#### 1. **Análisis Léxico**

```rust
// Tokens generados
[
    Token::Decl,
    Token::Identifier("x".to_string()),
    Token::Assign,
    Token::IntegerLiteral(42),
    Token::In,
    Token::Print,
    Token::Identifier("x".to_string()),
    Token::Eof
]
```

#### 2. **Análisis Sintáctico**

```rust
// AST generado
Expr::Decl(
    "x".to_string(),           // nombre de variable
    vec![],                    // parámetros (vacío)
    Box::new(Expr::Number(42)), // valor inicial
    Box::new(Expr::Call(       // cuerpo
        "print".to_string(),
        vec![Expr::Ident("x".to_string())]
    ))
)
```

#### 3. **Generación de LLVM IR**

```llvm
; Función main generada
define i64 @main() {
entry:
  ; Alocar espacio para variable x
  %x_ptr = alloca i64

  ; Almacenar valor inicial 42
  store i64 42, ptr %x_ptr

  ; Cargar valor de x para print
  %x_val = load i64, ptr %x_ptr

  ; Llamar a printf
  %printf_result = call i32 @printf(ptr @fmt_str, i64 %x_val)

  ; Retornar el valor de x
  ret i64 %x_val
}

; Cadena de formato para printf
@fmt_str = private constant [6 x i8] c"%lld\12\00"

; Declaración de printf externo
declare i32 @printf(ptr, ...)
```

#### 4. **Optimización LLVM**

LLVM puede aplicar optimizaciones como:

- **Eliminación de código muerto**
- **Propagación de constantes**
- **Inline de funciones**
- **Optimizaciones de bucles**

#### 5. **Generación de Código Objeto**

```assembly
; Código assembly x86-64 generado (simplificado)
main:
    push   %rbp
    mov    %rsp,%rbp
    sub    $0x10,%rsp

    ; Almacenar 42 en la pila
    movq   $42,-8(%rbp)

    ; Preparar llamada a printf
    mov    $fmt_str,%rdi
    mov    -8(%rbp),%rsi
    call   printf

    ; Retornar valor
    mov    -8(%rbp),%rax
    leave
    ret
```

#### 6. **Enlazado**

El enlazador (GCC) combina:

- **Código objeto del programa**
- **Bibliotecas del sistema** (libc para printf)
- **Runtime de LLVM** (si es necesario)

#### 7. **Ejecutable Final**

```bash
$ ./programa
42
$ echo $?    # Código de salida
42
```

### Manejo de Errores en el Pipeline

#### 1. **Errores Léxicos**

```mlia
decl x <- @invalid_char in x
```

```
Error: Carácter inesperado '@' en la línea 1, columna 11
```

#### 2. **Errores Sintácticos**

```mlia
decl x <- 42 x  (* falta 'in' *)
```

```
Error de parsing en token 4: Identifier("x"), error: ...
```

#### 3. **Errores Semánticos**

```mlia
print y  (* variable no definida *)
```

```
Error: Variable no definida 'y'
```

#### 4. **Errores de Generación**

```mlia
unknown_function 42  (* función desconocida *)
```

```
Error: Llamada a función desconocida 'unknown_function'
```

---

## Ejemplos Prácticos

### Ejemplo 1: Variable Simple

#### Código MLIA

```mlia
decl x <- 42 in print x
```

#### Proceso de Compilación

**Tokens**:

```
[Decl, Identifier("x"), Assign, IntegerLiteral(42), In, Print, Identifier("x")]
```

**AST**:

```
Decl("x", [], Number(42), Call("print", [Ident("x")]))
```

**LLVM IR**:

```llvm
define i64 @main() {
entry:
  %x_ptr = alloca i64
  store i64 42, ptr %x_ptr
  %x_val = load i64, ptr %x_ptr
  %call = call i32 @printf(ptr @fmt_str, i64 %x_val)
  ret i64 %x_val
}
```

**Salida**:

```
42
```

### Ejemplo 2: Múltiples Declaraciones

#### Código MLIA

```mlia
decl a <- 2 in
decl b <- 3 in
print b;
print a;
0
```

#### AST Resultante

```
Decl("a", [], Number(2),
  Decl("b", [], Number(3),
    Seq(
      Seq(
        Call("print", [Ident("b")]),
        Call("print", [Ident("a")])
      ),
      Number(0)
    )
  )
)
```

#### Trace de Ejecución

1. **Declarar `a = 2`**: Crear variable en la pila
2. **Declarar `b = 3`**: Crear otra variable
3. **Print `b`**: Cargar valor 3 y imprimir
4. **Print `a`**: Cargar valor 2 y imprimir
5. **Retornar 0**: Valor final del programa

**Salida**:

```
3
2
```

### Ejemplo 3: Shadowing de Variables

#### Código MLIA

```mlia
decl x <- 1 in
  print x;
  decl x <- 2 in
    print x;
  print x
```

#### Análisis de Scoping

1. **Scope externo**: `x = 1`
   - Print `x` → imprime `1`

2. **Scope interno**: `x = 2` (sombrea el `x` externo)
   - Print `x` → imprime `2`

3. **Vuelta al scope externo**: `x = 1` otra vez
   - Print `x` → imprime `1`

#### LLVM IR (simplificado)

```llvm
define i64 @main() {
entry:
  ; Variable x externa
  %x_outer = alloca i64
  store i64 1, ptr %x_outer

  ; Print x externa (1)
  %val1 = load i64, ptr %x_outer
  call i32 @printf(ptr @fmt_str, i64 %val1)

  ; Variable x interna
  %x_inner = alloca i64
  store i64 2, ptr %x_inner

  ; Print x interna (2)
  %val2 = load i64, ptr %x_inner
  call i32 @printf(ptr @fmt_str, i64 %val2)

  ; Print x externa otra vez (1)
  %val3 = load i64, ptr %x_outer
  call i32 @printf(ptr @fmt_str, i64 %val3)

  ret i64 %val3
}
```

### Ejemplo 4: Compilación y Ejecución

#### Uso de Línea de Comandos

```bash
# Compilar y ejecutar con JIT (por defecto)
$ cargo run -- test_simple.mlia
Parsing source code...
Parse result: Decl("x", [], Number(42), Call("print", [Ident("x")]))

Compiling and executing with JIT...
42

Generated LLVM IR:
; ModuleID = 'mlia_module'
source_filename = "mlia_module"

@fmt_str = private constant [6 x i8] c"%lld\12\00"

declare i32 @printf(ptr, ...)

define i64 @main() {
entry:
  %x_ptr = alloca i64
  store i64 42, ptr %x_ptr
  %x_val = load i64, ptr %x_ptr
  %printf_call = call i32 @printf(ptr @fmt_str, i64 %x_val)
  ret i64 %x_val
}

Program returned: 42
```

```bash
# Compilar a ejecutable
$ cargo run -- test_simple.mlia --exe
Parsing source code...
Parse result: Decl("x", [], Number(42), Call("print", [Ident("x")]))

Compiling to executable...
Successfully compiled to executable: test_simple.exe

# Ejecutar el programa compilado
$ ./test_simple.exe
42
$ echo $?
42
```

---

## Conceptos Avanzados

### 1. **Autómatas Finitos en el Lexer**

#### ¿Qué es un Autómata Finito?

Un **autómata finito** es un modelo matemático de computación que consiste en:

- **Estados finitos**: Un conjunto limitado de estados
- **Alfabeto**: Conjunto de símbolos de entrada
- **Función de transición**: Define cómo cambiar de estado
- **Estado inicial**: Punto de partida
- **Estados de aceptación**: Estados finales válidos

#### Implementación en MLIA

El tokenizador implementa un **DFA (Autómata Finito Determinista)**:

```rust
// Cada estado representa una situación específica
pub enum State {
    Start,                    // Estado inicial
    Digit,                   // Reconociendo números
    Identifier,              // Reconociendo identificadores
    Comment,                 // Dentro de comentario
    // ...
}

// Tabla de transiciones codifica el autómata
pub const STATE_TRANSITIONS: [[i8; NUM_CLASSES]; NUM_STATES] = [
    // [Estado][Clase_Carácter] = Estado_Siguiente
    [1, 5, 3, 3, 7, 7, ...],  // Transiciones desde Start
    [1, -2, -2, -2, ...],     // Transiciones desde Digit
    // ...
];
```

#### Ventajas del Enfoque con Autómata

1. **Eficiencia**: O(n) en tiempo, donde n es la longitud del texto
2. **Determinismo**: No hay ambigüedad en el reconocimiento
3. **Facilidad de mantenimiento**: Cambios localizados en la tabla
4. **Verificabilidad**: Se puede probar matemáticamente

### 2. **Parsers LR(1) y Teoría de Lenguajes**

#### Jerarquía de Gramáticas (Chomsky)

1. **Tipo 0**: Irrestrictas (máquinas de Turing)
2. **Tipo 1**: Sensibles al contexto
3. **Tipo 2**: Libres de contexto (CFG)
4. **Tipo 3**: Regulares (autómatas finitos)

MLIA es un **lenguaje libre de contexto** parseable con LR(1).

#### ¿Por qué LR(1)?

- **L**: Left-to-right scan (lectura izq. a der.)
- **R**: Rightmost derivation in reverse (derivación por derecha reversa)
- **1**: 1 token de lookahead

**Ventajas**:

- Detecta errores **tan pronto como sea posible**
- **No necesita backtracking**
- Maneja **asociatividad y precedencia** naturalmente

#### Algoritmo LR(1)

```python
def parse_lr1(tokens):
    stack = [0]  # Pila con estados
    input_idx = 0

    while True:
        state = stack[-1]
        token = tokens[input_idx]
        action = ACTION_TABLE[state][token]

        if action.type == SHIFT:
            stack.append(token)
            stack.append(action.next_state)
            input_idx += 1

        elif action.type == REDUCE:
            rule = GRAMMAR[action.rule]
            # Pop 2 * len(rule.rhs) elementos
            for _ in range(2 * len(rule.rhs)):
                stack.pop()
            # Construir nodo AST
            node = rule.semantic_action()
            # Goto
            stack.append(rule.lhs)
            stack.append(GOTO_TABLE[stack[-2]][rule.lhs])

        elif action.type == ACCEPT:
            return stack[1]  # AST raíz

        else:  # ERROR
            raise ParseError(f"Error en token {token}")
```

### 3. **LLVM IR y Representaciones Intermedias**

#### ¿Por qué Representaciones Intermedias?

Las **IR (Intermediate Representations)** proporcionan:

1. **Independencia de arquitectura**: El mismo IR funciona en x86, ARM, etc.
2. **Optimizaciones**: Más fácil optimizar IR que código fuente o assembly
3. **Verificación**: Se puede verificar correctitud del IR
4. **Reutilización**: Múltiples frontends pueden usar el mismo backend

#### Características de LLVM IR

- **SSA Form**: Single Static Assignment
- **Tipado estático**: Cada valor tiene un tipo
- **Estructura jerárquica**: Módulos → Funciones → Bloques básicos → Instrucciones

#### Ejemplo de Transformación SSA

**Código original**:

```c
x = 1;
x = x + 2;
y = x;
```

**Forma SSA**:

```llvm
%x1 = i64 1
%x2 = add i64 %x1, 2
%y1 = i64 %x2
```

Cada variable se **asigna exactamente una vez**.

#### Bloques Básicos

Un **bloque básico** es una secuencia de instrucciones:

- Con un **punto de entrada único** (primera instrucción)
- Con un **punto de salida único** (última instrucción)
- **Sin saltos** en el medio

```llvm
entry:                          ; Etiqueta del bloque
  %x = alloca i64              ; Instrucción 1
  store i64 42, ptr %x         ; Instrucción 2
  %val = load i64, ptr %x      ; Instrucción 3
  ret i64 %val                 ; Instrucción terminal
```

### 4. **Tabla de Símbolos y Gestión de Scope**

#### Implementación de Scoping

MLIA implementa **alcance léxico estático** con una tabla hash:

```rust
variables: HashMap<String, PointerValue<'ctx>>
```

#### Algoritmo de Scoping

```rust
fn enter_scope(&mut self, var_name: String, var_ptr: PointerValue) -> Option<PointerValue> {
    // Guardar binding anterior (si existe)
    let old_binding = self.variables.insert(var_name, var_ptr);
    old_binding
}

fn exit_scope(&mut self, var_name: String, old_binding: Option<PointerValue>) {
    match old_binding {
        Some(old_ptr) => {
            // Restaurar binding anterior
            self.variables.insert(var_name, old_ptr);
        }
        None => {
            // No había binding anterior, eliminar variable
            self.variables.remove(&var_name);
        }
    }
}
```

#### Ejemplo de Trace de Scoping

```mlia
decl x <- 1 in        (* [x₁] *)
  decl y <- 2 in      (* [x₁, y₁] *)
    decl x <- 3 in    (* [x₂, y₁] - x₁ está sombreado *)
      print x         (* accede a x₂ = 3 *)
    (* salir: [x₁, y₁] - restaurar x₁ *)
  (* salir: [x₁] - eliminar y₁ *)
(* salir: [] - eliminar x₁ *)
```

### 5. **Optimizaciones Potenciales**

#### Optimizaciones de Frontend

1. **Eliminación de código muerto**:

```mlia
decl x <- 42 in     (* x nunca se usa *)
print 100
(* → optimizado a: print 100 *)
```

2. **Propagación de constantes**:

```mlia
decl x <- 5 in
decl y <- x + 3 in
print y
(* → optimizado a: print 8 *)
```

3. **Inline de expresiones**:

```mlia
decl f arg <- arg + 1 in
f 42
(* → optimizado a: 42 + 1 *)
```

#### Optimizaciones de LLVM

LLVM aplica automáticamente muchas optimizaciones:

- **Eliminación de loads/stores redundantes**
- **Optimización de expresiones constantes**
- **Eliminación de código inalcanzable**
- **Desenrollado de bucles**
- **Inline de funciones**

### 6. **Extensiones del Lenguaje**

#### Características que se Podrían Agregar

1. **Funciones de primera clase**:

```mlia
decl add x y <- x + y in
decl apply f a b <- f a b in
apply add 3 4
```

2. **Condicionales**:

```mlia
decl max x y <-
  if < x y then y else x in
max 10 20
```

3. **Listas**:

```mlia
decl list <- [1, 2, 3] in
decl head <- first list in
print head
```

4. **Pattern matching**:

```mlia
match list with
| [] -> 0
| x :: xs -> x + sum xs
```

5. **Sistema de tipos**:

```mlia
decl add : Int -> Int -> Int =
  fun x y -> x + y
```

#### Desafíos de Implementación

- **Inferencia de tipos**: Algoritmo Hindley-Milner
- **Gestión de memoria**: Garbage collection o ownership
- **Polimorfismo**: Generics y monomorphization
- **Concurrencia**: Threads, async/await
- **Interoperabilidad**: FFI con C/C++

### 7. **Herramientas de Desarrollo**

#### Debugging del Compilador

1. **Visualización del AST**:

```rust
fn print_ast(expr: &Expr, indent: usize) {
    match expr {
        Expr::Number(n) => println!("{}Number({})", " ".repeat(indent), n),
        Expr::Decl(var, _, val, body) => {
            println!("{}Decl({})", " ".repeat(indent), var);
            print_ast(val, indent + 2);
            print_ast(body, indent + 2);
        }
        // ...
    }
}
```

2. **Visualización de LLVM IR**:

```rust
pub fn print_ir(&self) {
    self.module.print_to_stderr();
}
```

3. **Profiling de compilación**:

```rust
use std::time::Instant;

let start = Instant::now();
let tokens = lexer.tokenize()?;
println!("Tokenizing took: {:?}", start.elapsed());

let start = Instant::now();
let ast = parse_program(source)?;
println!("Parsing took: {:?}", start.elapsed());
```

#### Testing del Compilador

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let program = "decl x <- 2 + 3 in x";
        let result = compile_and_run(program).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_scoping() {
        let program = r"
            decl x <- 1 in
            decl x <- 2 in
            x
        ";
        let result = compile_and_run(program).unwrap();
        assert_eq!(result, 2);
    }
}
```

---

## Conclusión

El compilador MLIA demuestra los principios fundamentales de construcción de compiladores en un paquete completo y funcional. Desde el análisis léxico con autómatas finitos hasta la generación de código nativo con LLVM, cada fase implementa técnicas estándar de la industria.

### Conceptos Clave Aprendidos

1. **Arquitectura de compiladores**: Pipeline de tres fases
2. **Análisis léxico**: Autómatas finitos y tokenización
3. **Análisis sintáctico**: Parsers LR(1) y construcción de AST
4. **Generación de código**: LLVM IR y compilación nativa
5. **Gestión de scope**: Tablas de símbolos y alcance léxico
6. **Representaciones intermedias**: Beneficios y diseño
7. **Optimizaciones**: Técnicas de frontend y backend

### Aplicabilidad

Los principios demonstrados en MLIA se aplican a:

- **Compiladores de producción**: GCC, Clang, rustc
- **Interpretes**: Python, Ruby, JavaScript V8
- **Transpiladores**: TypeScript, Babel, CoffeeScript
- **DSLs**: Lenguajes específicos de dominio
- **Herramientas de análisis**: Linters, formateadores

### Próximos Pasos

Para profundizar en compiladores, considera:

1. **Implementar extensiones** al lenguaje MLIA
2. **Estudiar compiladores reales** como rustc o LLVM
3. **Leer literatura académica** sobre optimizaciones
4. **Experimentar con diferentes arquitecturas** objetivo
5. **Contribuir a proyectos** de compiladores open source

El compilador MLIA proporciona una base sólida para entender cualquier sistema de compilación moderno.
