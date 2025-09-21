# Admin Panel

- [x] create corporate group
- [x] delete corporate group
- [x] update corporate group
- [x] add user to corporate group with role

Cascade delete per eliminazione dei documenti... lato definizione delle entities si può creare la lista delle dipendenze dei diversi documenti così da avere centralizzato tutto.
Quando un documento viene eliminato si guardano le sue dirette dipendenze e si eliminano.
Si prosegue ricorsivamente fino a che non si giunge alle foglie dell'albero e da lì si procede con l'eliminazione delle entità fino ad arrivare all'eliminazione del documento originale

# Flusso creazione di un corporate group

1. Il platform admin crea un corporate group senza companies perché le creerà l'utente designato
2. Il platform admin aggiunge un utente al corporate group che essendo senza companies non gli viene assegnato un contratto
