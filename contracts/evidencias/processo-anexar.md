---
type: OperationEvidence
title: processo.anexar
operation: processo.anexar
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: true
upstream_steps: 7
unresolved_steps: 0
---

# processo.anexar

Origem: matriz de demonstrações do Todos. IES Rust: **PARCIAL WEB**, com prévia e confirmação por POST único e releitura **comprovadas somente em bancada sintética** ([BDD-270-02](../../tests/aies_anexacao_cli.rs); [CI oficial anterior](https://github.com/franklinbaldo/ies/actions/runs/42767244366)). O confronto semântico independente das transições com as cassettes continua **NOT_RUN**; a inspeção de integridade e estrutura de 102 interações UnB/GO não é esse confronto. [Validação live](../../docs/validacao-live/2026-11-05-processo-anexar-handoff.md): **NOT_RUN**, zero requisições nesta fatia. O RED inicial registrou oito falhas no CI 42084762362. Veja o [contrato](../cli/processo-anexar.md).
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-unb#8` → [processo-anexar-unb--031b1681716a.md](../../cassettes/unb/processo-anexar-unb--031b1681716a.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-pen1#8` → [processo-anexar-pen1--cbb3bf3b824f.md](../../cassettes/pen1/processo-anexar-pen1--cbb3bf3b824f.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-pen2#8` → [processo-anexar-pen2--9477930be451.md](../../cassettes/pen2/processo-anexar-pen2--9477930be451.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-ufrj#8` → [processo-anexar-ufrj--0fa4ed4ee794.md](../../cassettes/ufrj/processo-anexar-ufrj--0fa4ed4ee794.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-pen3#8` → [processo-anexar-pen3--b182f6135f18.md](../../cassettes/pen3/processo-anexar-pen3--b182f6135f18.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-pen4#8` → [processo-anexar-pen4--3b24cfdf993d.md](../../cassettes/pen4/processo-anexar-pen4--3b24cfdf993d.md)

## go

Estado registrado no Todos: **demonstrada**.

- `processo-anexar-go#8` → [processo-anexar-go--7dec4ec7e6f4.md](../../cassettes/go/processo-anexar-go--7dec4ec7e6f4.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).
