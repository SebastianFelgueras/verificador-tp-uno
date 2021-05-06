use haskell_ghci_parser::{
    HaskellError,
    HaskellGHCIParser,
    Comparar,
};
use verificadorlib::{
    verifica_goldbach,
    verificar_conjetura_hasta,
    chequear_descomposicion_en_primos,
};
use std::{
    process::exit,
    env,
    panic,
    io::Write
};
fn main() {
    panic::set_hook(
        Box::new(|info|{
            println!("El programa tuvo un error crítico. Por favor comunique la siguiente información a sebastian.felgueras@gmail.com:\n{}",info);
            exit(-1000);
        })
    );
    let mut argumentos = env::args().skip(1);
    let argumento = match argumentos.next(){
        Some(v)=>v,
        None=>{
            println!("Se esperaba un argumento por línea de comandos con el archivo a cargar");
            exit(1)
        }
    };
    let llegar_hasta = match match argumentos.next(){
        Some(v)=>v,
        None=>{
            println!("Se esperaba un segundo argumento por línea de comandos con el numero hasta el cual testear (exclusive)");
            exit(1)
        }
    }.parse::<usize>(){
        Ok(v)=>v,
        Err(_)=>{
            println!("Se esperaba un segundo argumento por línea de comandos con el numero hasta el cual testear");
            exit(1)
        }
    } +1; 
    let mut interprete = match HaskellGHCIParser::init(){
        Ok(v)=>v,
        Err(e)=>parse_err(e),
    };
    println!("Cargando modulo...");
    procesar_resultado(interprete.cargar_modulo(argumento));
    println!("Modulo cargado. Iniciando pruebas...");

    //GOLDBACH
    verificar(&mut interprete,"satisfaceGoldbach",verifica_goldbach,0,1,llegar_hasta);

    //VERIFICAR HASTA
    verificar(&mut interprete,"verificarConjeturaHasta",verificar_conjetura_hasta,4,2,llegar_hasta);

    //descomposicion en primos
    println!("Testeando descomposicionEnPrimos.");
    let mut errores = Vec::new();
    print!("Testeando ");
    for i in (4..llegar_hasta).step_by(2){
        imprimir(i);
        std::io::stdout().flush().unwrap();
        procesar_resultado(
            interprete.ejecutar_comando(
                &format!("descomposicionEnPrimos {}\n",i)
            )
        );
        let devuelto = procesar_resultado(
            HaskellGHCIParser::parsear_avanzar_linea(
                &interprete.avanzar_linea()
            )
        );
        if !chequear_descomposicion_en_primos(i,devuelto.clone()){
            errores.push((i,devuelto));
        }
    }
    print!("\n");
    if errores.is_empty(){
        println!("No se encontraron errores")
    }else{
        println!("Se encontraron {} errores:",errores.len());
        for (i,devuelto) in errores{
            println!("descomposicionEnPrimos fallo en {} al devolver {}",i,devuelto);
        }
    }
    println!("descomposicionEnPrimos testeada.");
    println!("Pruebas terminadas.");
    interprete.terminar();
}
#[inline]
fn adapt_bool(b:bool)->&'static str{
    if b{
        "True"
    }else{
        "False"
    }
}
#[inline]
fn procesar_resultado<T>(r: Result<T,HaskellError>)->T{
    if let Err(e) = r{
        parse_err(e)
    }else{
        r.unwrap()
    }
}

fn verificar<T: Fn(usize)->bool>(interprete: &mut HaskellGHCIParser,mensaje:&str,funcion_para_comparar: T,desde:usize,step:usize,llegar_hasta:usize){
    println!("Testeando {}.",mensaje);
    print!("Testeando ");
    let mut errores = Vec::new();
    for i in (desde..llegar_hasta).step_by(step){
        imprimir(i);
        std::io::stdout().flush().unwrap();
        if let Comparar::Diferentes(s) = procesar_resultado(interprete.chequear_valor(
            &format!("{} {}\n",mensaje,i), 
            &adapt_bool(funcion_para_comparar(i)))){
                errores.push((i,s))
                //println!("{} fallo en {} al devolver {}",mensaje,i,s);  
        }
    }
    print!("\n");
    if errores.is_empty(){
        println!("No se encontraron errores")
    }else{
        println!("Se encontraron {} errores:",errores.len());
        for (i,s) in errores{
            println!("{} fallo en {} al devolver {}",mensaje,i,s);
        }
    }
    println!("{} testeada.",mensaje);
}
//Es un asco el diseño de esto pero me da paja hacerla bien
fn imprimir(n:usize){
    let a = if n > 9999{
        format!("{}\x08\x08\x08\x08\x08",n)
    }else if n>999{
        format!("{}\x08\x08\x08\x08",n)
    }else if n>99{
        format!("{}\x08\x08\x08",n)
    }else if n>9{
        format!("{}\x08\x08",n)
    }else{
        format!("{}\x08",n)
    };
    print!("{}",a);
}
fn parse_err(e: haskell_ghci_parser::HaskellError)->!{
    match e{
        HaskellError::InterpreteNoIniciado=>{
            println!("ERROR: Error al iniciar el intérprete (¿Quizás no se encuentra en la variable del entorno PATH?)");
            exit(-1);
        }
        HaskellError::InterpreteTerminado=>{
            println!("ERROR: Intérprete terminado antes de tiempo (Alguno de los pipes se rompió inesperadamente)");
            exit(-2);
        }
        HaskellError::ModuloNoCargado=>{
            println!("ERROR: No fue posible cargar el módulo (¿Quizás no se dio la ruta correcta o quizá había un error de compilación?)");
            exit(-3);
        }
        HaskellError::ErrorDeEjecucion=>{
            println!("ERROR: Error de ejecución en el intérprete");
            exit(-4);
        }
        HaskellError::PosibleErrorInterno(v)=>{
            println!("ERROR: Linea de comparación inválida, puede ser un error interno del verificador, por favor enviar este mensaje a sebastian.felgueras@gmail.com: \"{}\"",v);
            exit(-5);
        }
    }
}
