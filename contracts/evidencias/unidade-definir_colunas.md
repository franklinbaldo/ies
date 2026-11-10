---
type: OperationEvidence
title: unidade.definir_colunas
operation: unidade.definir_colunas
source: todos-demonstration-matrix
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
writes: true
ies_implemented: true
upstream_steps: 0
unresolved_steps: 0
---

# unidade.definir_colunas

Operação do IES derivada do ensaio do proprietário na tela **Nível de
detalhe** (`procedimento_configurar_detalhe`), **não** uma operação da
matriz do Todos: a matriz não tem passo nem cassette para ela
(`upstream_steps: 0`). O `source_commit` é o da matriz consultada, sem
passos correspondentes.

## Evidência disponível

- Ensaio live na PEN1 5.0.4, treinamento, 02/11/2026, do proprietário
  ([validação](../../docs/validacao-live/2026-11-02-leitura-controle.md)):
  campos do POST de salvar, resposta 200 que fecha o modal e efeito visível
  na D seguinte; configuração original restaurada e conferida por GET.
- Capturas privadas só estruturais da tela em PEN1 e GO (12 opções) e o
  ensaio da UnB 4.0.9 (11 opções), descritos no
  [contrato da leitura](../cli/unidade-colunas.md).

## Limite da evidência

Não há cassette pública da tela nem da resposta do POST (nenhuma
interação `procedimento_configurar_detalhe` nos 29 ZIPs, conferido em
04/11/2026). A resposta de sucesso só existe em captura privada da PEN1
5.0.4 (02/11/2026): a mesma tela com o sinal de gravação no
`inicializar()`, que o IES lê para classificar a resposta; o efeito
continua conferido por GET. Ensaios do comando: PASS na PEN1 e na UnB
(02/11/2026) e na UnB (04/11/2026); saída 1 sem efeito na PEN1 em
04/11/2026, com causa não registrada
([análise](../../docs/validacao-live/2026-11-04-definir-colunas-pen1.md)).
