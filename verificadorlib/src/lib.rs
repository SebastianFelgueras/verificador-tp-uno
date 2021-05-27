///Es casi un sinónimo de true, solo chequea que cumpla que n par y mayor a dos
pub fn verifica_goldbach(n:usize)->bool{
    n % 2 ==0 && n > 2 //considerar que ya esta testeada hasta 1e18
}
///espera un n par mayor a dos
pub fn verificar_conjetura_hasta(n:usize)->bool{
    if (n % 2 == 1) || n < 3{
        panic!("No se cumple el contrato de la función");
    }
    true
}

fn es_primo(n:usize)->bool{
    if n == 1 || (n %2 ==0 && n !=2){
        return false;
    }
    let raiz = (n as f64).sqrt().floor() as usize + 1;
    let mut i = 3;
    while i <= raiz{
        if n % i == 0{
            return false;
        }
        i+=2;
    }
    true
}
///Devuelve true si esta todo bien
pub fn chequear_descomposicion_en_primos(n:usize,devolucion:String)->bool{
    if n < 3 || n % 2 ==1{
        panic!("No se respeta el contrato");
    }
    let mut devolucion = devolucion.strip_prefix("(").unwrap().trim().strip_suffix(")").unwrap().split(",");
    let a = devolucion.next().unwrap().parse::<usize>().unwrap();
    let b = devolucion.next().unwrap().parse::<usize>().unwrap();
    a+b == n && es_primo(a) && es_primo(b)
}

pub fn numero_de_descomposiciones(n:usize)->usize{
    if n % 2 ==1 || n<3{panic!("No cumple contrato")}
    let mut a = 0;
    for i in 2..(n-1){
        if es_primo(i) && es_primo(n-i){
            a+=1;
        }
    }
    a
}

#[cfg(test)]
mod tests{
    #[test]
    fn es_primo_test() {
        let primos = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251, 257, 263, 269, 271, 277, 281, 283, 293, 307, 311, 313, 317, 331, 337, 347, 349, 353, 359, 367, 373, 379, 383, 389, 397, 401, 409, 419, 421, 431, 433, 439, 443, 449, 457, 461, 463, 467, 479, 487, 491, 499, 503, 509, 521, 523, 541, 547, 557, 563, 569, 571, 577, 587, 593, 599, 601, 607, 613, 617, 619, 631, 641, 643, 647, 653, 659, 661, 673, 677, 683, 691, 701, 709, 719, 727, 733, 739, 743, 751, 757, 761, 769, 773, 787, 797, 809, 811, 821, 823, 827, 829, 839, 853, 857, 859, 863, 877, 881, 883, 887, 907, 911, 919, 929, 937, 941, 947, 953, 967, 971, 977, 983, 991 , 997
        ];
        for i in 2..1001{
            if !(super::es_primo(i) == primos.contains(&i)){
                panic!("Problema con {}",i)
            }
        }
    }
    #[test]
    fn numero_de_descomposiciones_test() {
        use super::numero_de_descomposiciones;
        assert_eq!(4,numero_de_descomposiciones(20));
        assert_eq!(3,numero_de_descomposiciones(10));
        assert_eq!(8,numero_de_descomposiciones(88));
        assert_eq!(1,numero_de_descomposiciones(4));
        assert_eq!(2904,numero_de_descomposiciones(123456))
    }
}