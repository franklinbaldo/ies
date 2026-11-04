---
type: ImplementationDecision
title: Movido cujo documento está entre os documentos do processo
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter (Refs #59); DOCUMENTO_MOVIDO cuja primeira parte do id composto é o id de um DOCUMENTO da mesma árvore deixa de recusar a árvore e publica presente_entre_os_documentos
selected_strategy: web-movido-presente-por-id
alternatives: web-movido-presente-por-id,web-movido-presente-recusa,web-movido-devolvido,web-movido-presente-omitido
evidence_level: replay-offline-do-acervo-publico-e-testes-sinteticos
live_validation: not-run-mover-documento-e-escrita
---

# Movido cujo documento está entre os documentos do processo

Apêndice da [decisão de processo.obter](../processo-obter.md), ao lado do
de [documentos movidos](documentos-movidos.md). Contrato em
[movidos](../../../contracts/cli/processo-obter-movidos.md). Refs #59.

## O que o acervo mostra

O [replay do acervo](../../validacao-replay/2026-11-04-processo-obter-acervo.md)
recusava 32 árvores com `documento_movido_presente_entre_os_documentos`,
todas de `documento-mover-cancelar-{go,pen1,pen2,pen3}`, 8 por
instalação. Lidas nas cassettes (estrutura e texto das telas públicas de
treinamento), as quatro contam a mesma história:

1. o processo A tem o `DOCUMENTO` `n`; dois `documento_mover` (POST);
2. a árvore de A traz só o `DOCUMENTO_MOVIDO` `n-m1`, destino B;
3. a de B traz o `DOCUMENTO` `n`, depois o `DOCUMENTO_MOVIDO` `n-m2`,
   destino A (dica com o motivo `…, de volta`);
4. as 8 árvores seguintes de A, as recusadas, trazem `Nos[1]` o movido
   `n-m1` (destino B publicado e conferido) e `Nos[2]` o `DOCUMENTO` `n`,
   com o **mesmo** rótulo e o **mesmo** número SEI (a15); `m1 ≠ m2`.

GO: A `202600005005672`, B `202600005005673`, `n` 8458734, ícone
`documento_txt`; PEN1 a PEN3: ícone `documento_html`. O histórico
resumido gravado (um por cassette) só tem "gerado" e "remetido": os
movimentos não aparecem nele, então a evidência é a sequência das árvores
e dos dois POST, não o histórico. Nenhum movido repete a primeira parte
de outro; nenhum está em pasta.

## Alternativas consideradas

1. **web-movido-presente-por-id** (escolhida): aceitar; o `DOCUMENTO` em
   `documentos` como sempre, o movido em `documentos_movidos` com
   `presente_entre_os_documentos: true` (a primeira parte de a1 é id de
   um `DOCUMENTO` da mesma árvore) ou `false`.
2. **web-movido-presente-recusa**: manter a recusa.
3. **web-movido-devolvido**: o mesmo, com o campo `devolvido`.
4. **web-movido-presente-omitido**: esconder o movido quando o documento
   está presente.

## Comparação

| Critério | 1. por id | 2. recusa | 3. devolvido | 4. omitido |
| --- | --- | --- | --- | --- |
| Requisições a mais | 0 | 0 | 0 | 0 |
| Processo real abre | sim | não (32 árvores) | sim | sim |
| Fato publicado | conferível no próprio JSON | — | inferido | some um nó servido |

## Decisão

Escolhida a 1. A 2 derruba `ies processo` em quatro instalações depois
de um movimento de ida e volta, uso normal da tela. A 3 nomeia o caminho:
o IES vê o nó de saída e o documento presente, não o retorno (só a
árvore de B, outra requisição, mostra o segundo movimento). A 4 apaga um
nó que o SEI serve e a ligação com o destino. O nome do campo repete o
fato verificado e o subcódigo antigo: `true` se confere com um `id_documento`
de `documentos` da mesma saída. O fluxo só publica árvore sem pasta
fechada, então `false` não depende de pasta não carregada.

Continuam recusa: composto repetido e primeira parte repetida entre
movidos (um documento movido duas vezes deste processo não foi
observado), pai diferente do processo, nó ou destino fora da forma.
Rótulo, número SEI e ordem não são exigidos iguais: o acervo os mostra
iguais, mas renomear o documento depois não muda a identidade. O texto
humano acrescenta `(o documento está de novo neste processo)`: o nó diz
que ele saiu, o `DOCUMENTO` diz que está.

## Evidências verificáveis

- [Relatório do replay](../../validacao-replay/2026-11-04-processo-obter-acervo.md)
  (antes: 32 recusas; depois: lidas).
- [Contrato](../../../contracts/cli/processo-obter-movidos.md) e
  [observação](../../como-o-sei-funciona/arvore-do-processo.md#documentos-movidos).
- Testes sintéticos: `src/arvore_processo/tests_movidos.rs`,
  `src/processo_obter_cli/tests_movidos.rs`.

## Limitações e revisão

Validação live **NOT_RUN**: mover documento é escrita e não foi feito; a
evidência é o replay das cassettes públicas. Rever se aparecer um
documento movido duas vezes do mesmo processo, um movido em pasta ou um
movido presente cuja primeira parte não seja o `id_documento` do link.
