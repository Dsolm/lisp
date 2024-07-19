# Intèrpret de llenguatge lisp-like escrit en rust.
Implementa un llenguatge funcional amb estat immutable, gestió automàtica de memoria (per reference counting), multithreading segur, closures, strings, llistes enllaçades, lexical scoping i recursivitat.

L'execució amb multithreading no fa servir cap Mutex trer d'un RwLock en l'entorn global, que només bloqueja l'execució quan es defineix una variable global, cosa que és impossible desde dins d'una funció. No té un Global Interpreter Lock, per tant, totes les threads avançaran sempre alhora (si no s'introdueix cap altre recurs extern, es clar).

Com que la gestió de memòria s'implementa amb reference counting és possible que les estructures de dades amb cicles de referències causin fugues de memòria.
