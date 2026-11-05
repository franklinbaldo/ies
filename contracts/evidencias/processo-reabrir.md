---
type: OperationEvidence
title: processo.reabrir
operation: processo.reabrir
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: true
upstream_steps: 11
unresolved_steps: 0
---

# processo.reabrir

Origem: matriz de demonstrações do Todos. IES Rust: PARCIAL WEB
([contrato](../cli/processo-reabrir.md), Refs #172): prévia, o GET da URL
`procedimento_reabrir` que a tela do nó publica (corpo de
`reabrirProcesso()` no 4.0, `var linkReabrirProcesso` no 4.1/5.0) e
verificação por `aberto_em`; escrita live PASS na UFRJ 4.1.5 (variável) e
na UnB 4.0.9 (função). Os estados abaixo pertencem ao Todos, não são
vereditos do IES.

## Evidência própria do IES

- [Escrita live](../../docs/validacao-live/2026-11-04-processo-concluir.md):
  **PASS** na UFRJ 4.1.5 (URL em variável) e UnB 4.0.9 (URL em função),
  com verificação por leitura; outras instalações **NOT_RUN** no ensaio.

- [Observação estrutural](../../docs/como-o-sei-funciona/reabertura-do-processo.md)
  das 11 GET de `procedimento_reabrir` e das 431 telas do nó gravadas.
- [Replay do acervo](../../docs/validacao-replay/2026-11-04-processo-reabrir-acervo.md):
  190 cassettes, as 11 escritas aceitas e verificadas pela situação.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-unb#12` → [processo-escritas-unb--47c0acc994ac.md](../../cassettes/unb/processo-escritas-unb--47c0acc994ac.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen1#12` → [processo-escritas-pen1--1332f6c7cd6d.md](../../cassettes/pen1/processo-escritas-pen1--1332f6c7cd6d.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen2#12` → [processo-escritas-pen2--fa172ed36530.md](../../cassettes/pen2/processo-escritas-pen2--fa172ed36530.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `processo-escrita-ciclo-ufrj#14` → [processo-escrita-ciclo-ufrj--9b07e32f03c8.md](../../cassettes/ufrj/processo-escrita-ciclo-ufrj--9b07e32f03c8.md)
- `processo-escritas-ufrj#12` → [processo-escritas-ufrj--d21a20db9b10.md](../../cassettes/ufrj/processo-escritas-ufrj--d21a20db9b10.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `processo-escrita-ciclo-pen3#14` → [processo-escrita-ciclo-pen3--0bd247494de6.md](../../cassettes/pen3/processo-escrita-ciclo-pen3--0bd247494de6.md)
- `processo-escritas-pen3#12` → [processo-escritas-pen3--cc4742d79c53.md](../../cassettes/pen3/processo-escritas-pen3--cc4742d79c53.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `processo-escrita-ciclo-pen4#14` → [processo-escrita-ciclo-pen4--bc7c41370e64.md](../../cassettes/pen4/processo-escrita-ciclo-pen4--bc7c41370e64.md)
- `processo-escritas-pen4#12` → [processo-escritas-pen4--bc8c8929ced1.md](../../cassettes/pen4/processo-escritas-pen4--bc8c8929ced1.md)

## go

Estado registrado no Todos: **demonstrada**.

- `processo-escrita-ciclo-go#15` → [processo-escrita-ciclo-go--092d6ba86600.md](../../cassettes/go/processo-escrita-ciclo-go--092d6ba86600.md)
- `processo-escritas-go#12` → [processo-escritas-go--841b0341a184.md](../../cassettes/go/processo-escritas-go--841b0341a184.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
