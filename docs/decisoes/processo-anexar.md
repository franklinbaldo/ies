---
type: ImplementationDecision
title: Anexação de processo com prévia Web e confirmação verificada em bancada
operation: processo.anexar
implementation: partial
status: adopted-for-scope
scope: prévia Web, POST único em origem de treinamento e verificação por leitura independente, comprovados sinteticamente
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
   principal, destinatário e assinaturas sem escrita na prévia; sob
   `--confirmar` autorizado, deriva da tela o POST único, verifica a
   resposta e relê a lista e a árvore do principal.
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
Na prévia, não há POST mutador. Em origens autorizadas de treinamento,
`--confirmar` deriva a submissão da tela servida, efetua **um único POST**,
confere o aceite HTTP e só publica sucesso depois de ler novamente a lista
e a árvore do principal, identificando o anexado. Resposta 302 isolada
não prova a mudança. Quando a releitura falha, a CLI retorna **saída 1**,
sem retry, e informa que não se deve repetir `--confirmar`. A implementação
passou em teste com CLI Rust real, anti-IES por HTTPS local e SQLite
compartilhado; **isso é prova sintética, não validação externa do SEI**.

A [especificação](../../contracts/cli/processo-anexar.md) estabelece o
orçamento e os subcódigos; a [evidência do Todos](../../contracts/evidencias/processo-anexar.md)
distingue a proveniência das cassettes da validação própria do IES.
A [matriz de derivação](../matriz-derivacao-cli.md) registra a fatia.

## Evidências verificáveis

- [Testes Rust da CLI](../../tests/cli_processo_anexar.rs) cobrem
  argumentos e recusas antes da rede, inclusive a política de origens.
- [BDD-270-02 em bancada](../../tests/aies_anexacao_cli.rs) executa a CLI
  real contra HTTPS local, anti-IES e SQLite: um POST mutador, releitura
  independente por segunda invocação e saída 1 após HTTP 503 sintético
  posterior à persistência, sem repetição da escrita.
- [CI oficial #42767244366](https://github.com/franklinbaldo/ies/actions/runs/42767244366):
  7/7 PASS no head `c588b06`, antes desta atualização documental.
- [Inspeção de cassettes UnB/GO](https://github.com/franklinbaldo/ies/actions/runs/42755568647):
  integridade e estrutura de 102 interações PASS; **confronto semântico
  independente de resposta e transição NOT_RUN**.
- [Oito cenários unitários](../../src/processo_anexar/tests.rs) partiram
  do oráculo do Todos antes do código. O RED observou oito falhas no CI
  [42084762362](https://github.com/franklinbaldo/ies/actions/runs/42084762362).
- [Relatório live](../validacao-live/2026-11-05-processo-anexar-handoff.md):
  **NOT_RUN**, zero requisições, nenhum resíduo criado no SEI.

## Limitações e revisão

O confronto independente dos leitores IES e anti-IES com as respostas e
transições das cassettes continua **NOT_RUN**; hashes íntegros não são replay
semântico. A prévia e a confirmação em duas instalações externas de
**treinamento** também estão **NOT_RUN**, apesar do sucesso sintético.
Orçamento completo de requisições por instalação e fidelidade do POST/302
seguem sujeitos a observação. Não fazer escrita em produção nem retry.
A recusa antiga de `--confirmar` antes da sessão foi um limite da fatia
anterior, não descreve o caminho confirmado agora disponível em treinamento.
