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

## Evidencia adicional do calendario — replay de 05/11/2026

O [diagnostico offline conferido](https://github.com/franklinbaldo/ies/actions/runs/42120788751)
percorreu as respostas GET dos sete cards. Em UnB, UFRJ, GO e PEN1–4,
os calendarios apresentaram normalmente **duas ancoras** mensais
com `data_inicial` e `data_final`, mas **uma unica URL mensal
distinta**. Isso foi observado inclusive nos calendarios de GO e PEN3.
A quantidade de ancoras nao e a quantidade de escolhas: duas ancoras
com href assinado identico significam uma unica oferta do mes.

O leitor antigo de `mes_inteiro` recusava a segunda ancora mesmo com
href identico, produzindo `calendario_dos_retornos_desconhecido`.
A correcao da issue #253 aceita apenas duplicatas literalmente iguais;
duas URLs assinadas diferentes continuam ambiguas. Nao foram impressos
hrefs, hashes, cookies ou HTML da sessao. Essa observacao offline
explica uma causa **plausivel**, mas nao prova sozinha que foi
exatamente a causa da recusa live PEN3, pois essa tela nao foi
capturada integralmente na tentativa anterior.

## Efeito e limites

Somente leitura por GET; GET nao e tratado como isento de efeitos
de estado de navegacao da sessao. Nao restaurar o mes anterior.
Os paginadores de `Aguardando` e `Devolver` sao independentes;
o replay de lista com ambos paginados nao comprova uma unica
sequencia segura. O leitor do IES recusa essa forma.

A evidencia inicial e UnB 4.0.9 via Todos. Em 05/11 o IES tentou leitura
em GO e PEN3, com logins bem-sucedidos, mas sem lista validada; na PEN3
houve `calendario_dos_retornos_desconhecido`. Consultar o
[relatorio live](../validacao-live/2026-11-05-unidade-retornos.md).
Nao extrapolar a forma entre versoes.
