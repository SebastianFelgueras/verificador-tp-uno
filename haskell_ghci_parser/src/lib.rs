// Dado que rust no ofrece un método no bloqueante para leer una linea de stdout o stderr,
//uso dos threads cuyo unico objeto es bloquearse hasta que leen una linea y cuando la leen, 
//se la pasan a el coso
use std::{
    process,
    env,
    io::{self,BufRead,Write},
    thread,
    sync::{Mutex,Arc}
};
#[cfg(target_os = "windows")]
const NOMBREGHCI: &str = "ghci.exe";
#[cfg(target_os = "macos")]
const NOMBREGHCI: &str = "ghci";
#[cfg(target_os = "linux")]
const NOMBREGHCI: &str = "ghci";
type Result<A> = std::result::Result<A,HaskellError>;

pub struct HaskellGHCIParser{
    proceso: process::Child,
    input: process::ChildStdin,
    ///Contiene la combinacion de stdout y stderr
    streams: Arc<Mutex<String>>,
    last_line_read: Vec<String>,
    _threads_handlers: Vec<thread::JoinHandle<()>>,
}
impl HaskellGHCIParser{
    ///Inicializa el intérprete de Haskell y hace los pipes correspondientes
    //internamente solamente inicia el intérprete
    pub fn init()->Result<Self>{
        let hold = env::var("PATH").unwrap();
        let paths = env::split_paths(&hold);
        for mut path in paths{
            path.push(NOMBREGHCI);
            if let Ok(val) = process::Command::new(path)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn(){
                return Self::get_ready(val)
            }
        }
        Err(HaskellError::InterpreteNoIniciado)
    }
    ///toma el handler del intérprete y lo deja listo para procesar entradas y salidas
    fn get_ready(mut proceso: process::Child)->Result<Self>{
        let streams = Arc::from(Mutex::new(String::new()));
        let mut threads_handlers = Vec::new();
        //El thread con i=0 captura stdout, el otro stderr
        enum Capturar{
            Stdout(io::BufReader<process::ChildStdout>),
            Stderr(io::BufReader<process::ChildStderr>),
        }
        for i in 0..2 {
            let weak_ref = Arc::downgrade(&streams); //quiero un weak pointer
            let stream =
            if i == 0{
                Capturar::Stdout(io::BufReader::new(proceso.stdout.take().unwrap()))
            }else{
                Capturar::Stderr(io::BufReader::new(proceso.stderr.take().unwrap()))
            };
            threads_handlers.push(thread::spawn(
                move ||{
                    fn interna<T: BufRead>(mut stream: T,weak_ref:std::sync::Weak<Mutex<String>>){
                        let mut temp = String::new();
                        loop{
                            //devuelve None si no hay mas strong references y como la única que existe
                            //la tiene la propia estructura, eso quiere decir que la struct ya no existe
                            //por lo que el thread tiene que terminar
                            if let Some(valor) = weak_ref.upgrade(){
                                temp.clear();
                                stream.read_line(&mut temp).unwrap();
                                let mut mutex = (*valor).lock().expect("Mutex envenenado");
                                mutex.push_str(&temp);
                                drop(mutex);
                            }else{
                                return;
                            }
                        }
                    }
                    match stream{
                        Capturar::Stderr(stream)=>interna(stream,weak_ref),
                        Capturar::Stdout(stream)=>interna(stream,weak_ref),
                    };   
                }
            ));            
        }
        let mut temp = Self{
            input: proceso.stdin.take().unwrap(),
            proceso,
            streams,
            last_line_read: Vec::new(),
            _threads_handlers: threads_handlers, 
        };
        temp.avanzar_linea(); //la primer linea es ghci bla bla bla
        Ok(temp)
    }
    pub fn cargar_modulo(&mut self,path: String)->Result<()>{
        self.ejecutar_comando(&format!(":load {}\n",path))?;
        let coso = self.avanzar_linea();
        //println!("{} {}",self.last_line_read,self.last_line_read.starts_with("Couldn't"));
        if coso.starts_with("target") || coso.starts_with("Couldn't"){
            return Err(HaskellError::ModuloNoCargado);
        }else{
            loop{
                if self.avanzar_linea().starts_with("Ok"){
                    return Ok(())
                }else if self.avanzar_linea().starts_with("Failed"){
                    return Err(HaskellError::ModuloNoCargado)
                }
            }
        }
    }
    pub fn avanzar_linea(&mut self)->String{
        loop{
            if let Some(v) = self.last_line_read.get(0){
                let a = v.clone();
                //println!("{}",a);
                self.last_line_read.remove(0);
                return a.to_string();
            }else{
                self.actualizar_last_line_read()
            }
        }
        /*println!("{}",self.last_line_read);
        Ok(0)*/
    }
    fn actualizar_last_line_read(&mut self){
        let mut mutex = self.streams.lock().expect("Mutex envenenado");
        self.last_line_read.append(&mut mutex
            .lines()
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect());
        mutex.clear();
        drop(mutex);
    }
    ///Devuelve true si son iguales lo esperado a lo devuelto, **si o si tiene que incluir el newline character al final la input**
    /// y el resultado esperado no debe incluir newline character
    pub fn chequear_valor(&mut self,input:&str,output_esperada:&str)->Result<Comparar>{
        interprete_terminado_lectura(self.input.write(input.as_bytes()))?;
        //print!("{}",self.last_line_read);
        let linea = self.avanzar_linea();
        //println!("{}",linea);
        if linea.starts_with("<interactive>"){
            return Err(HaskellError::ErrorDeEjecucion);
        }
        let t = Self::parsear_avanzar_linea(&linea)?;
        if t == output_esperada{
            Ok(Comparar::Iguales)
        }else{
            Ok(Comparar::Diferentes(linea))
        }
    }
    pub fn terminar(self){
        drop(self)
    }
    ///Pasa el comando a ghci
    pub fn ejecutar_comando(&mut self,command:&str)->Result<usize>{
        interprete_terminado_lectura(self.input.write(command.as_bytes()))
    }
    ///Descarta n lineas de la output, puede ser util cuando se llama ejecutar comando
    pub fn descartar_n_lineas(&mut self,n:usize)->Result<usize>{
        for _ in 0..n{
            self.avanzar_linea();
        }
        Ok(n)
    }
    pub fn parsear_avanzar_linea(linea:&String)->Result<String>{
        match linea.splitn(2,"> ").last(){
            Some(v)=>Ok(v.trim().to_string()),
            None=>return Err(HaskellError::PosibleErrorInterno(linea.clone()))
        }
    }
}

impl Drop for HaskellGHCIParser{
    ///Notar que liquida los threads iniciados ya que estos paran cuando el strong_count de 
    /// last_line_read baja a cero
    fn drop(&mut self) {
        let _ = self.proceso.kill();
        //si ya termino devuelve err
    }
}
///Convierte un result a Result<a,HaskellError>
fn interprete_terminado_lectura<T>(a:io::Result<T>)->Result<T>{
    match a {
        Ok(v)=>Ok(v),
        Err(_)=>Err(HaskellError::InterpreteTerminado),
    }
}
#[derive(Debug)]
pub enum HaskellError{
    InterpreteNoIniciado,
    InterpreteTerminado,
    ModuloNoCargado,
    ErrorDeEjecucion,
    PosibleErrorInterno(String),
}
#[derive(Debug,PartialEq,Eq)]
pub enum Comparar{
    Iguales,
    Diferentes(String)
}
#[cfg(test)]
mod tests{
    use super::HaskellGHCIParser;
    #[test]
    fn inicializar() {
        HaskellGHCIParser::init().unwrap().terminar();
    }
    #[test]
    fn chequear_dos_mas_dos(){
        let mut interprete = HaskellGHCIParser::init().expect("Error al inicializar el interprete");
        assert_eq!(super::Comparar::Iguales,interprete.chequear_valor("2+2\n", "4").unwrap());
        interprete.terminar();
    }
    #[test]
    fn chequear_varias_cosas() {
        let mut interprete = HaskellGHCIParser::init().expect("Error al inicializar el interprete");
        assert_eq!(super::Comparar::Iguales,interprete.chequear_valor("2+2\n", "4").unwrap());
        assert_eq!(super::Comparar::Iguales,interprete.chequear_valor("2+3\n", "5").unwrap());
        assert_eq!(super::Comparar::Iguales,interprete.chequear_valor("2*9\n", "18").unwrap());
        interprete.terminar();
    }
}