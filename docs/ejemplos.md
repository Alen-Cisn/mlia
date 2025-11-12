# Códigos de ejemplo de MLia

```
decl variable <- 20 in
decl otraVariable <- 30 in
decl miFunción arg1 arg2 <-
 + arg1 arg2 in
(*
Comentario
*)
decl +función-con/símbolos!raros arg1 <- 0 in


while < variable otraVariable do
 otraVariable <- - otraVariable 1;

    decl nuevoValor <-
  match otraVariable with
      | 30 ->
       decl a <- 2 in
          miFunción a 3
         | 20 ->
          2
 in
 print nuevoValor
done

1

(* Ejemplo de operador AND *)
decl x <- 5 in
decl y <- 10 in
decl z <- 3 in

print (& (< x y) (> x z));

0

(* Ejemplo de combinacion AND/OR *)

print (| (| (& 1 1) (& 1 0)) (| (& 0 0) (& 0 1)));

0

(* Ejemplo funcion con match y 3 argumentos *)
decl fun a b c <- (+ (+ a b) (match (% c 2) with
    | 0 -> -20
    | _ -> 10)
) in
print (fun 6 5 5);
0

(* Ejemplo funciones anidadas *)
decl fn1 a b <- + a b in
decl fn2 c d <- 
    decl fn3 f g <- (fn1 (fn1 f g) 1) in
    (fn3 c d)
in
print (fn2 3 4);
0

(* Ejemplo fibonacci: imprime los primeros n numeros de la sucesion *)
decl fib n <- 
    match n with
    | 0 -> 0
    | 1 -> 1
    | _ -> + (fib (- n 1)) (fib (- n 2))
in
decl printFibSeries i n <-
    match > i n with
    | 1 -> 0
    | _ -> 
        print (fib i);
        (printFibSeries (+ i 1) n)
in
(printFibSeries 0 10);
0

(* Ejemplo fibonacci 2: imprime los numeros de la sucesion menores a n *)
decl fib n <- 
    match n with
    | 0 -> 0
    | 1 -> 1
    | _ -> + (fib (- n 1)) (fib (- n 2))
in
decl printFibUpTo i n <-
    decl fibValue <- (fib i) in
    match > fibValue n with
    | 1 -> 0
    | _ -> 
        print fibValue;
        (printFibUpTo (+ i 1) n)
in
(printFibUpTo 0 100);
0

(* Ejemplo Closure simple *)
decl x <- 5 in
decl fun n <- + n x in
print (fun 10);
0

(* Ejemplo nested closures *)
decl x <- 10 in
decl y <- 20 in
decl outer a <- 
    decl inner b <- + (+ a b) (+ x y) in
    (inner 5)
in
print (outer 3);
0

(* Ejemplo secuencia de prints con decls *)
print 1;
print 2;
decl x <- 10 in
print x;
decl y <- 20 in
print (+ x y);
print 3

```
