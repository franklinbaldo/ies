---
type: ImplementationDecision
title: Anexação de processo com prévia Web e confirmação fechada
operation: processo.anexar
implementation: partial
status: adopted-for-scope
scope: prévia somente, duas consultas AJAX de leitura e recusa da escrita até teste integrado de verificação
selected_strategy: web-formulario-da-tela-com-duas-consultas-observadas
alternatives: web-formulario-da-tela-com-duas-consultas-observadas,web-sem-verificacao-de-assinaturas,rest-wssei
evidence_level: todos-anexacao-unb-409-20261015-e-testes-sinteticos
live_validation: not-run
---

# Decisão de anexação de processo (issue #270)

## Contexto

A anexação é uma operação de efeito potencialmente irreversível e merece
conferência de identidade, assinaturas, envio único e leitura pós-escrita.
O oráculo `Todos/src/todos/anexacao.py` registra UnB 4.0.9 (15/10/2026):
link `procedimento_anexar`, formulário `frmProcedimentoAnexar`,
consultas `protocolo_RI1023` e `procedimento_anexacao_verificar`,
resposta 302 à escrita e verificação da nova lista de anexados.

## Alternativas consideradas

1. **Tela servida com duas consultas de leitura** (escolhida): confere
   principal, destinatário e assinaturas sem escrita e sem inventar URL.
2. **Sem consulta de assinaturas**: menos uma requisição, mas perde a
   observação de pendências antes da ação irreversível.
3. **REST/WSSEI**: fora da prioridade Web e sem observação equivalente.

## Comparação

| Critério | Tela e duas consultas | Sem assinaturas | REST |
| --- | --- | --- | --- |
| Identidade conferida antes da escrita | sim | sim | não observado |
| Assinaturas informadas | sim | não | não observado |
| POST de escrita da prévia | 0 | 0 | não observado |
| Requisições após abertura da árvore | 1 GET + 2 POST de leitura | 1 GET + 1 POST | desconhecidas |
| Evidência | Todos + testes Rust | cenário incompleto | nenhuma |

## Decisão

Abrir o processo principal pelo fluxo Web compartilhado, conferir o
link assinado da barra, ler `frmProcedimentoAnexar` via
`html_atributos` e observar as duas consultas AJAX de leitura.
As recusas são tipadas e não há tentativa automática de recuperação.
O `--confirmar` recusa antes de acessar a sessão, porque o transporte
de escrita única e a verificação por leitura ainda não foram testados.
Não ativar o POST pela simples aparência de sucesso de 302.

A [especificação](../../contracts/cli/processo-anexar.md) estabelece o
orçamento e os subcódigos; a [evidência do Todos](../../contracts/evidencias/processo-anexar.md)
distingue a proveniência das cassettes da validação própria do IES.
A [matriz de derivação](../matriz-derivacao-cli.md) registra a fatia.

## Evidências verificáveis

- [Testes Rust da CLI](../../tests/cli_processo_anexar.rs) cobrem
  argumento inválido, ausência de sessão, identidade igual e
  `--confirmar` recusado **antes de rede**.
- [Oito cenários unitários](../../src/processo_anexar/tests.rs) partiram
  do oráculo do Todos antes do código. O RED observou oito falhas no CI
  [42084762362](https://github.com/franklinbaldo/ies/actions/runs/42084762362).
- [Relatório live](../validacao-live/2026-11-05-processo-anexar-handoff.md):
  **NOT_RUN**, zero requisições, nenhum resíduo criado no SEI.

## Limitações e revisão

A forma exata dos scripts AJAX, o orçamento completo do fluxo,
o replay das cassettes por instalação, a prévia em UnB e outra
instalação de treinamento e a escrita ainda precisam ser verificados.
Para `--confirmar`, implementar POST único sem retry, conferência da
resposta e releitura nova da árvore do principal; se o efeito não for
comprovado, saída 3 sem reenviar. Não fazer escrita em produção.
