# CoinPusherTDL

Trabajo práctico de la materia Teoria de Lenguajes de Programción de FIUBA. Un empuja-monedas tambien llamado coin-pusher en ingles.

## Introduccion

Un coin pusher - empuja-monedas - es una máquina tragamonedas en la cual las monedas ingresadas se acumulan en una superficie. El objetivo del usuario es que la moneda que inserte empuje a otras monedas previamente ingresadas haciéndolas caer y ganándolas como premio.

### Propuesta

Se plantea desarrollar un coin pusher online donde varios clientes pueden jugar simultáneamente, y en tiempo real, arrojar sus monedas con el objetivo de quedarse las de jugadores previos.

### Especificaciones técnicas

- El servidor es desarrollado en Rust

### Especificaciones funcionales

- Puede arrojarse de a una moneda a la vez por jugador. Se evalua ofrecer la opcion de tiradas de a N monedas.
- El límite es de N jugadores por coin pusher.
- Existe un único coin pusher o se le ofrece al usuario una pool de coin pushers en los cuales jugar?


### Ejecución

Primero, ejecutar el servidor:

```bash
cargo build && cargo run -p server
```

El servidor se ejecutará en `localhost:1883`. Dichos parámetros, al igual que la cantidad de monedas iniciales de la máquina del servidor, pueden ser modificados en el archivo `server/resources/config.txt`.

Luego, ejecutar el cliente:

```bash
cargo build && cargo run -p client localhost 1883
```

Alternativamente, si se tiene el comando `make` instalado, se puede ejecutar:

```bash
make run_server
```

y posteriormente:

```bash
make run_client
```

### Comandos adicionales

Para ejecutar los tests, el linter y el formateador de código, se puede ejecutar:

```bash
make all
```

que ejecutará los siguientes comandos en todos los directorios del proyecto:

```bash
cargo test && cargo fmt && cargo clippy
```

### Acciones del cliente

```
t : Insert coin
y : Check coins
q : Quit
```




