---
type: SEIObservation
title: Retorno Programado - navegacao e formato das listas
description: Observacao estrutural anterior ao RED a partir das capturas e testes do Todos 323eb01
---

# Tela Retorno Programado no SEI (Refs #253)

Fonte: [Todos @323eb01](https://github.com/franklinbaldo/todos/blob/323eb01/src/todos/retorno_programado.py),
[testes da cassette UnB](https://github.com/franklinbaldo/todos/blob/323eb01/tests/test_unidade_listar_retornos_010.py),
[registro da tela](https://github.com/franklinbaldo/todos/blob/323eb01/docs/como-o-sei-funciona/15-telas-da-barra/retornos-programados-da-unidade.md).

Essa observacao do oraculo foi lida **antes de criar os testes RED**
do IES em 05/11/2026. Ela nao constitui observacao ao vivo do IES.

## Navegacao observada

- No menu, a acao `retorno_programado_listar` publica uma URL assinada
  HTTPS. O cliente nao constroi hash nem publica URL assinada em saida.
- O calendario abre no ultimo dia visto pela sessao, que pode ser de
  outro mes. Cada dia e link GET com `data_atual=DD/MM/AAAA`.
  O primeiro dia dos meses vizinhos e link para mudar o mes.
- O dia de hoje do **SEI** tem a classe CSS `diaHoje`.
- O link `Ver todo o mes` contem
  `data_inicial=01/MM/AAAA` e
  `data_final=DD/MM/AAAA` (ultimo dia do mes).
  A listagem mensal, portanto, custa um GET em vez de GET por dia.
- A tela da lista tem tabelas com `summary` incluindo
  `aguardando retorno` e `para devolver`. Cabeçalhos:
  `Processo`, `Unidade`, `Data Programada`, opcionais
  `Data de Retorno Efetiva`, `Prazo Restante`.
  Cada legenda contem numero de `registro(s)`.
- Sem retorno, `divAguardandoAreaTabela` publica
  `Nenhum registro encontrado.`. Nao confundir uma pagina estranha
  sem tabelas com um mes vazio.

## Efeito e limites

Somente leitura por GET; GET nao e tratado como isento de efeitos
de estado de navegacao da sessao. Nao restaurar o mes anterior.
Os paginadores de `Aguardando` e `Devolver` sao independentes;
o replay de lista com ambos paginados nao comprova uma unica
sequencia segura. O leitor do IES recusa essa forma.

A evidencia inicial e UnB 4.0.9 via Todos. GO e PEN3
permanecem NOT_RUN no IES; nao extrapolar a forma entre versoes.
