---
type: OperationEvidence
title: processo.marcar
operation: processo.marcar
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: false
upstream_steps: 14
unresolved_steps: 0
---

# Evidências relacionadas ao comando `processo.marcar`

Este **card derivado** agrupa as operações distintas
[`marcador.aplicar`](marcador-aplicar.md) e
[`marcador.remover`](marcador-remover.md) da matriz do Todos.
`upstream_steps: 14` é a soma dos **7 passos de aplicação + 7 de
remoção**, cada qual identificado nas duas fichas originais.
`unresolved_steps: 0` significa que os 14 vínculos possuem fichas de
cassette no catálogo, **não** que a escrita foi validada no IES.
Não afirma que `processo.marcar` seja uma operação literal da matriz.

## Estado por instalação na matriz original

- UnB: `marcador-escritas-unb#7` (aplicar) e `#10` (remover).
- PEN1: `marcador-escritas-pen1#7` e `#10`.
- PEN2: `marcador-escritas-pen2#7` e `#10`.
- UFRJ: `marcador-escritas-ufrj#7` e `#10`.
- PEN3: `marcador-escritas-pen3#7` e `#10`.
- PEN4: `marcador-escritas-pen4#7` e `#10`.
- Goiás: `marcador-escritas-go#7` e `#10`.

Esses nomes e índices são **referências do roteiro Todos**;
as revisões imutáveis e hashes estão nas fichas originais, que devem
ser confrontadas no replay antes de alegar equivalência.

## Observado pelo IES

- [Estrutura da UnB 4.0.9](../../docs/como-o-sei-funciona/marcadores-do-processo.md):
  `tblMarcadores`, `id_marcador` compartilhado em processos diferentes,
  botões e formulário de gerenciamento.
- [Leitura live em 04/11](../../docs/validacao-live/2026-11-04-processo-obter-marcadores.md):
  mais de um marcador e ausência de marcadores; **não prova escrita**.
- [Fixture sintética](../../src/marcadores_processo/tests_apoio.rs)
  reproduz a forma observada.

## Não executado na nova PR

Replay das 14 referências e prévia/escrita live em treinamento:
**NOT_RUN** (zero requisições nesta sessão; sem resíduo no SEI).
Corpo POST, sucesso e verificação posterior por leitura **não foram
demonstrados**. Não tratar dados do Todos como resultado do IES.
Ver [relatório](../../docs/validacao-live/2026-11-05-processo-marcar.md).

[Contrato](../cli/processo-marcar.md) ·
[Decisão](../../docs/decisoes/processo-marcar.md) ·
[Teste de CLI](../../tests/cli_processo_marcar.rs).
