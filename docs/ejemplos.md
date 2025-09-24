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
```