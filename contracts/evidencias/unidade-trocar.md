---
type: OperationEvidence
title: unidade.trocar
operation: unidade.trocar
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: true
upstream_steps: 0
unresolved_steps: 0
---

# unidade.trocar

Operação do IES aprovada pelo proprietário em 04/11/2026 (#205). A matriz
do Todos não a tem como operação própria: o Todos troca de unidade pelo
`SEI_UNIDADE` dentro de outros comandos (`usuario listar-unidades`,
`--unidade`), então não há passo correspondente (`upstream_steps: 0`). O
`source_commit` é o da matriz consultada.

## Evidência disponível

Cassettes com o POST de `infra_trocar_unidade` (22 trocas), lidas pelos
leitores do comando no [replay](../../docs/validacao-replay/2026-11-05-unidade-trocar-acervo.md):

- UnB 4.0.9: [unidade-troca-e-volta-unb](../../cassettes/unb/unidade-troca-e-volta-unb--c834c505aee4.md),
  [bloco-devolver-unb](../../cassettes/unb/bloco-devolver-unb--f91bc4cdddf0.md),
  [processo-pasta-mista-unb](../../cassettes/unb/processo-pasta-mista-unb--dffe2d9379eb.md);
- UFRJ 4.1.5: [unidade-troca-e-volta-ufrj](../../cassettes/ufrj/unidade-troca-e-volta-ufrj--b2c6346c8f1e.md),
  [bloco-devolver-ufrj](../../cassettes/ufrj/bloco-devolver-ufrj--5ec0a808f72d.md),
  [processo-pasta-mista-ufrj](../../cassettes/ufrj/processo-pasta-mista-ufrj--65861ad4af07.md);
- GO 5.0.3 (`Location` em `http://`): [unidade-troca-e-volta-go](../../cassettes/go/unidade-troca-e-volta-go--adbfaefa1d78.md),
  [bloco-devolver-go](../../cassettes/go/bloco-devolver-go--d0951370f2d8.md),
  [bloco-devolver-go (outra revisão)](../../cassettes/go/bloco-devolver-go--87741a72a8c9.md).

Observação estrutural em [troca de unidade](../../docs/como-o-sei-funciona/troca-de-unidade.md).

## Limite da evidência

As gravações não guardam o corpo do POST nem `Set-Cookie`. O corpo vem do
formulário servido e do JavaScript `selecionarUnidade()`; a prova é o
Controle da unidade nova. PEN1 a PEN4 só gravaram o GET da tela.
