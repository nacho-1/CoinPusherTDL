# CoinPusherTDL

Trabajo práctico de la materia Teoria de Lenguajes de Programción de FIUBA. Un empuja-monedas tambien llamado coin-pusher en ingles.

## Introduccion

Un coin pusher - empuja-monedas - es una máquina tragamonedas en la cual las monedas ingresadas se acumulan en una superficie. El objetivo del usuario es que la moneda que inserte empuje a otras monedas previamente ingresadas haciéndolas caer y ganándolas como premio.

### Propuesta

Se plantea desarrollar un coin pusher online donde varios clientes pueden jugar simultáneamente, y en tiempo real, arrojar sus monedas con el objetivo de quedarse las de jugadores previos.

### Especificaciones tecnicas

- El servidor es desarrollado en Rust

### Especificaciones funcionales

- Puede arrojarse de a una moneda a la vez por jugador. Se evalua ofrecer la opcion de tiradas de a N monedas.
- El limite es de N jugadores por coin pusher.
- Existe un único coin pusher o se le ofrece al usuario una pool de coin pushers en los cuales jugar?

