---
type: ImplementationDecision
title: Resumo de identidade de processo observado na caixa da unidade
operation: processo.obter
implementation: partial
status: adopted-for-scope
scope: resumo de processo na caixa integral da unidade sem abrir processo
selected_strategy: web-box-get
alternatives: web-box-get,web-search-post,rest-wssei
evidence_level: cassettes-offline-e-testes-tipados
live_validation: nao-executada
---

# Resumo de processo sem abrir o processo

## Alternativas consideradas

- **web-box-get:** reutilizar a resposta da Caixa de Entrada obtida por
  GET autorizado e comparar o número exibido sem normalização.
- **web-search-post:** pesquisa rápida assinada, como no Todos, que pode
  conduzir à abertura de processo e exige avaliar efeitos.
- **rest-wssei:** módulo REST opcional, ausente em muitas instalações.

## Comparação

A Caixa de Entrada contém identidade e categoria do processo sem
solicitar a árvore. O GET é uma requisição de consulta, mas só permite
encontrar processos que já aparecem na unidade. A pesquisa rápida pode
abrir processo e registrar recebimento; REST não oferece cobertura
nacional. Não executar pesquisas ou releituras apenas para preencher
dados que o GET já trouxe.

## Decisão

Disponibilizar `ies processo NUP --resumo` como consulta parcial,
com uma única leitura da caixa da unidade. Exigir cobertura dos totais
declarados antes de concluir e preservar a distinção recebidos/gerados.
Não interpretar ausência na caixa como inexistência do processo.
Não abrir árvores, não enviar POST e não inventar propriedades.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-obter.md)
- [Matriz por instalação](../../contracts/evidencias/processo-obter.md)
- [Testes unitários e integração](../../tests/cli_contract.rs)
- [Reprodução da caixa](../../tests/replay_unidade.rs)

## Limitações e revisão

A consulta integral de processo continua stub. A leitura de resumo não
inclui documentos, andamentos, marcadores ou propriedades. Não testar
GET live sem sessão e instalação autorizadas. Revisar apenas com
evidência dos efeitos da abertura Web e mecanismos adequados para
projeções completas.
