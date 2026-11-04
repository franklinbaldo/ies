---
type: ImplementationDecision
title: Documentos movidos na árvore de ies processo NUP
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de processo.obter; nó DOCUMENTO_MOVIDO da árvore na forma da PEN1 4.0.12, fora dos documentos, com o destino lido do link assinado do .html do nó
selected_strategy: web-movidos-pelo-link-do-no
alternatives: web-movidos-pelo-link-do-no,web-movidos-pela-dica,web-movidos-recusa,web-movidos-como-documentos
evidence_level: captura-privada-pen1-estrutura-e-testes-sinteticos
live_validation: not-run
---

# Documentos movidos na árvore de `ies processo NUP`

Apêndice da [decisão de processo.obter](../processo-obter.md), separado
para mantê-la abaixo de 20.000 bytes. Refs #59.

## Alternativas consideradas

1. **web-movidos-pelo-link-do-no** (escolhida): aceitar o nó
   `DOCUMENTO_MOVIDO` na forma observada, publicá-lo em
   `documentos_movidos` (fora de `documentos`) e tirar o destino do único
   `<a>` do `.html` do nó: o `procedimento_trabalhar` assinado do
   destino, conferido como os outros links (ação, hash, `id_procedimento`
   decimal diferente do aberto) e amarrado ao nó pelo `id_documento`
   igual à primeira parte do a1. O NUP é o texto do link.
2. **web-movidos-pela-dica**: publicar a dica (a6) como `descricao` ou
   tirar dela o NUP do destino.
3. **web-movidos-recusa**: continuar recusando a árvore inteira
   (`tipo_de_no_da_arvore_nao_suportado`).
4. **web-movidos-como-documentos**: tratar o movido como um `DOCUMENTO`.

## Comparação

| Critério | 1. link do nó | 2. dica | 3. recusa | 4. documento |
| --- | --- | --- | --- | --- |
| Requisições a mais | 0 | 0 | 0 | 0 |
| Destino estrutural | sim (chaves e texto do link) | não (prosa) | — | — |
| Processo real abre | sim | sim | não | sim, com documento falso |
| Ensina o índice | par e link assinado conferidos | nada confiável | nada | nada |

## Decisão

Escolhida a 1. A 4 contaria como documento do processo algo que não está
mais nele (o `n` não é id de nenhum `DOCUMENTO` da árvore). A 3 derruba o
comando num processo real da PEN1. A 2 foi rejeitada para o destino: a
dica é frase (`Documento movido para o processo <NUP>: <motivo>`), e
mensagem é apresentação, nunca classificação; o mesmo NUP já vem no texto
de um link cujas chaves são conferidas. Publicar a dica inteira como
`descricao` também foi deixado de fora: traz o motivo digitado por uma
pessoa, como a dica dos e-mails traz remetentes, e não acrescenta
identidade. Pode entrar depois, como texto opaco, se alguém pedir.

Com destino conferido, o índice aprende o par NUP ↔ id e o link
`procedimento_trabalhar` do destino (objeto = id do destino), como o pai
de um anexado: depois, `ies processo <NUP do destino>` sai pelo passo 1b
em 2 GET. Sem `<a>` com `href` no `.html` (ou sem `.html`), `destino` é
`null` e nada é aprendido. `<a>` fora da forma, dois links ou dois
`.html` do nó são recusa tipada: com um link candidato malformado,
`destino: null` seria afirmar que o SEI não publicou o que publicou.

Movido com pai numa pasta não foi observado e é recusa
(`documento_movido_com_pai_diferente_do_processo`); como o movido nunca
entra em `documentos`, as conferências de `Pastas[k]['protocolos']` não
o veem. Primeira parte do a1 repetida entre movidos ou igual ao id de um
`DOCUMENTO` (um documento movido e devolvido, por exemplo) também é
recusa até haver captura.

## Evidências verificáveis

- [Contrato](../../../contracts/cli/processo-obter-arvore.md) e
  [evidência](../../../contracts/evidencias/processo-obter.md)
  (captura privada da PEN1 4.0.12, 04/11/2026, só estrutura; o leitor a
  aceitou em teste local temporário, apagado).
- [Observação](../../como-o-sei-funciona/arvore-do-processo.md).
- Testes sintéticos: `src/arvore_processo/tests_movidos.rs`,
  `src/processo_obter/tests_movidos.rs`,
  `src/processo_obter_cli/tests_movidos.rs`.
- Oráculo: o Todos não menciona `DOCUMENTO_MOVIDO`.

## Limitações e revisão

Validação live **NOT_RUN** até o agente pai rodar `ies processo` na PEN1.
Uma instalação só: rever se outra servir o nó noutra forma, se o `.html`
vier sem link (sem acesso ao destino, por exemplo), se aparecer movido
em pasta ou se o link do destino com `id_documento` não abrir a tela do
destino no passo 1b.
