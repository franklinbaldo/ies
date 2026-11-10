---
type: OperationEvidence
title: processo.relacionar
operation: processo.relacionar
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: true
upstream_steps: 8
unresolved_steps: 0
---

# processo.relacionar

Origem: matriz de demonstrações do Todos. IES Rust: PARCIAL WEB — a CLI publica prévia; `--confirmar` de adição e remoção é recusado antes da rede enquanto a escrita não for validada em treinamento. O motor de POST único e releitura independente existe apenas para testes sintéticos. Live NOT_RUN; ainda não há confirmação de execução em instalações de treinamento.
Os estados abaixo pertencem ao Todos, não são vereditos do IES.

As referências #N são passos de roteiro; o link leva à revisão com
o mesmo cenário, mas ainda não certifica igualdade de revisão.
Confronte os hashes com o manifesto atual antes de alegar paridade.

## unb

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-unb#6` → [processo-escritas-unb--47c0acc994ac.md](../../cassettes/unb/processo-escritas-unb--47c0acc994ac.md)

## pen1

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen1#6` → [processo-escritas-pen1--1332f6c7cd6d.md](../../cassettes/pen1/processo-escritas-pen1--1332f6c7cd6d.md)
- `processo-relacionar-remover-pen1#4` → [processo-relacionar-remover-pen1--3ed1bd40300e.md](../../cassettes/pen1/processo-relacionar-remover-pen1--3ed1bd40300e.md)

## pen2

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen2#6` → [processo-escritas-pen2--fa172ed36530.md](../../cassettes/pen2/processo-escritas-pen2--fa172ed36530.md)

## ufrj

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-ufrj#6` → [processo-escritas-ufrj--d21a20db9b10.md](../../cassettes/ufrj/processo-escritas-ufrj--d21a20db9b10.md)

## pen3

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen3#6` → [processo-escritas-pen3--cc4742d79c53.md](../../cassettes/pen3/processo-escritas-pen3--cc4742d79c53.md)

## pen4

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-pen4#6` → [processo-escritas-pen4--bc8c8929ced1.md](../../cassettes/pen4/processo-escritas-pen4--bc8c8929ced1.md)

## go

Estado registrado no Todos: **demonstrada**.

- `processo-escritas-go#6` → [processo-escritas-go--841b0341a184.md](../../cassettes/go/processo-escritas-go--841b0341a184.md)

## Limite da evidência

As respostas HTTP de ambientes públicos não comprovam, sozinhas,
corpos integrais de POST, paridade da CLI ou efeito de uma escrita.
Exija conferência pós-operação antes de declarar sucesso.

Consulte [cassettes](../../docs/cassettes.md),
[SEI empiricamente](../../docs/como-o-sei-funciona/README.md)
e [contrato provisório](../cli.md).

## Evidência própria IES #258

- [Contrato da prévia](../cli/processo-relacionar.md).
- [Decisão](../../docs/decisoes/processo-relacionar.md).
- [Testes estruturais](../../src/processo_relacionar/tests.rs) e
  [testes da CLI](../../tests/cli_processo_relacionar.rs).
- RED local NOT_RUN; replay NOT_RUN; live PEN1 e UnB NOT_RUN,
  zero requisições, nenhum resíduo SEI.
- **Separação entre CLI e motor WIP**: os flags `--confirmar` e `--remover --confirmar` são aceitos pelo parser da CLI, mas, até 09/11/2026, produziam `confirmacao_de_relacionamento_nao_validada` (saída 1) sem requisição ao SEI. Em testes sintéticos internos, a escrita usa um único POST mutador, exige aceite e releitura independente; esse caminho **não está acessível pela CLI**. Execuções reais em treinamento permanecem NOT_RUN; produção jamais acessada.
