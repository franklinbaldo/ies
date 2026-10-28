---
type: ImplementationDecision
title: Replay offline parcial do histórico do processo
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: leitura de uma resposta HTML histórica autenticada na cassette pública, sem rede
selected_strategy: web
alternatives: web, rest-wssei, soap-official
evidence_level: offline-replay-go-historico
live_validation: not-performed
---

# Histórico do processo — recorte offline

## Alternativas consideradas

- Web: resposta `procedimento_consultar_historico` já capturada,
  identificada por passo, com tabela `tblHistorico`.
- REST: especificação disponível, mas não confirmada na instalação
  nem equivalente ao histórico completo do Web.
- SOAP: requer cadastro e autorização próprios; não comprovado.

## Comparação

Web histórica: campos de andamento verificáveis no replay, sem rede.
REST: não comparado nesta instalação; SOAP: não comparado nem autorizado.
Não há medição de latência ou equivalência entre transportes.

## Decisão

Implementar apenas a projeção **parcial de andamentos históricos**
pela Web gravada. A consulta exige ZIP e card OKF conferidos
criptograficamente, passo GET selecionado, status 200, HTML válido e
número do processo confirmado no título da resposta. Não faz login,
requisição HTTP nem navegação que possa receber implicitamente o
processo. Sem origem explícita, `ies processo NUP` continua stub.

O resultado tem `escopo: historico_offline` e `parcial: true`.
`Andamento` traz campos observados; `AtuacaoNaUnidade` não é
inferida dos andamentos. Não criar identificador artificial de
andamento, não inferir completude de outros componentes do processo.

## Evidências verificáveis

- [Contrato CLI](../cli/processo-obter.md)
- [Card Goiás](../../cassettes/go/processo-andamento-go--ae86a5f02b6f.md)
- [Matriz](../evidencias/processo-obter.md)
- [Testes do contrato CLI](../../tests/cli_contract.rs)
- [Modelo](../../docs/dominio/modelo-conceitual-sei.md)
- [Acesso por instalação](../../docs/interfaces/acesso-por-instalacao.md)

O card de Goiás indexa `procedimento_consultar_historico` numa
cassette de sete interações. A resposta histórica possui
`tblHistorico`. Isto não comprova a equivalência de REST/SOAP,
nem valida a semântica de todas as sete instalações.

## Limitações e revisão

Reavaliar quando houver replay comprovado nas demais instalações,
histórico resumido/completo, paginação, múltiplos processos numa
cassette, ou qualquer proposta de leitura live. Antes de executar
abertura de processo com sessão real, investigar se a consulta pode
registrar recebimento e exigir intenção/consentimento explícito.
Não classificar um GET como leitura pura apenas pelo verbo HTTP.
