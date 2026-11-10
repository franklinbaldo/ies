---
type: ImplementationDecision
title: Processo marcar por id e nome com remoção sintética causal e atribuição ainda fechada
operation: processo.marcar
implementation: partial
status: adopted-for-scope
scope: previa e idempotencia de marcador, com remocao confirmavel por POST unico em treinamento e releitura independente; atribuicao ainda fechada
selected_strategy: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario
alternatives: reaproveitar-leitor-web-e-falhar-fechado-sem-formulario,post-inferido-da-tela,usar-api-rest
evidence_level: estrutura-observada-unb-e-cassettes-de-escrita-nao-reproduzidas
live_validation: not-run-sandbox-sem-acesso-sei
---

# Decisão: `processo marcar` (Refs #255)

## Contexto

A [tela observada na UnB](../como-o-sei-funciona/marcadores-do-processo.md)
e o relatório de [leitura live](../validacao-live/2026-11-04-processo-obter-marcadores.md)
demonstram que o mesmo `id_marcador` aparece em processos distintos;
o rótulo `id#nome` não é identidade da associação ao processo.
O `tblMarcadores` declara seu total, mas pode publicar menos linhas,
portanto ausência só é demonstrada numa lista completa. A ação servida
permite ensaiar um POST único de remoção no anti-IES; o resultado real
em cada SEI continua sem comprovação.

## Alternativas consideradas

1. **Reusar o leitor `processo_obter`**, conferir identidade e unidade,
   oferecer idempotência e prévia e falhar fechado antes da escrita
   (selecionada neste recorte).
2. Inferir um POST a partir do `acaoRemover`, sem reproduzir formulário
   e resultado: rejeitada porque não prova estado posterior nem campos.
3. REST/SOAP: não é a via Web observada nem provou equivalência.

## Comparação

| Critério | Leitor existente | POST inferido | REST |
| --- | --- | --- | --- |
| URL criada artificialmente | não | possivelmente | sim |
| Alteração antes de observar resposta | não | sim | sim |
| `id#nome` e unidade conferidos | sim | incompleto | desconhecido |
| Comportamento demonstrável offline | sim | não | não |
| Prévia sem escrita | sim | incerto | incerto |

## Decisão

Antes de qualquer arquivo, validar número e rótulo. Presença significa
id exato e nome exato; nome divergente é recusa, não substituição.
Ausência só é afirmada se `total_declarado == itens.len()`.
`null` nunca equivale a zero. Retirada de marcador presente retorna prévia sem POST. Em confirmação,
a origem é conferida **antes de qualquer requisição**; na origem de
treinamento aceita, o IES só envia uma vez após conferir os campos do
formulário servido e a ação de `acaoRemover`. Sucesso requer um 302
identificado como retirada do mesmo processo **mais** a ausência
comprovada por releitura independente. Escrita não confirmada sai 3 sem retry.
Atribuir marcador ausente exige prova do catálogo **da unidade** e a
confirmação de atribuição continua fechada.
Nenhum novo transporte, cookie próprio ou restaurador de tela.

## Evidências verificáveis

- [Contrato da operação](../../contracts/cli/processo-marcar.md).
- [Evidência, com limitações explícitas](../../contracts/evidencias/processo-marcar.md).
- [Testes de CLI](../../tests/cli_processo_marcar.rs).
- [Testes sintéticos de decisão](../../src/processo_marcar/tests.rs).
- [Relatório live](../validacao-live/2026-11-05-processo-marcar.md).

## Limitações e revisão

A retirada já percorre, em **hipótese sintética**, CLI real, formulário
servido, anti-IES, POST único, estado SQLite e releitura independente.
Isso **não** demonstra equivalência semântica com os POSTs das cassettes,
nem prova efeito em SEI real. Ainda faltam replay independente de
`marcador-escritas-*` e `marcador-ciclo-completo-*`, validação live em
treinamento e o catálogo/formulário de atribuição. Nenhuma escrita
externa foi realizada nesta fatia; produção permanece proibida para agentes.
