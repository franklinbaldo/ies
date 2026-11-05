---
type: OperationEvidence
title: processo.marcar
operation: processo.marcar
source: estrutura-observada-pelo-ies-e-indice-de-cassettes-do-todos
writes: true
ies_implemented: false
---

# Evidências verificáveis para `processo.marcar`

Esta operação contém **escrita**, mas somente o recorte de prévia e
idempotência está implementado no IES. `ies_implemented: false` refere-se
à **escrita**, que continua bloqueada. Não há equivalência provada de
corpo POST por replay ou teste live desta PR.

## Observado antes dos testes RED

- [Estrutura da UnB 4.0.9](../../docs/como-o-sei-funciona/marcadores-do-processo.md):
  tela `andamento_marcador_gerenciar`, `tblMarcadores`, coluna
  `id_marcador` usada também em processos diferentes e
  controles `btnAdicionar`/`btnRemover`.
- [Leitura live preexistente](../../docs/validacao-live/2026-11-04-processo-obter-marcadores.md):
  marcadores múltiplos e processo vazio; **não prova escrita**.
- [Fixture sintética do leitor](../../src/marcadores_processo/tests_apoio.rs)
  reproduz a forma observada e distingue `id` da associação.

## Não observado ou não executado

O inventário cita `marcador-escritas-*` e
`marcador-ciclo-completo-*`, porém esta PR ainda não executou seu
replay; nenhuma contagem de cassettes ou POST é alegada.
[Relatório da sessão](../../docs/validacao-live/2026-11-05-processo-marcar.md):
prévia live e escrita **NOT_RUN**, zero requisições e resíduo nulo.

[Contrato](../cli/processo-marcar.md) ·
[Decisão](../../docs/decisoes/processo-marcar.md) ·
[Teste de CLI](../../tests/cli_processo_marcar.rs).
