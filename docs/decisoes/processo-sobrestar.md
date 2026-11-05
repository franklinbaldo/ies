---
type: ImplementationDecision
title: Sobrestar e retomar por ação Web assinada com situação verificada
operation: processo.sobrestar
implementation: partial
status: adopted-for-scope
scope: os dois comandos da unidade da sessão com prévia; escrita ainda bloqueada
selected_strategy: barra-assinada-com-situacao-lida
alternatives: barra-assinada-com-situacao-lida,rest-nao-observado,heuristica-textual
evidence_level: cassettes-todos-sete-instalacoes-e-testes-sinteticos
live_validation: not-run
---

# Decisão — sobrestar e retomar (#269)

## Contexto

A barra da árvore do processo publica `procedimento_sobrestar` como
URL assinada. A retomada é `removerSobrestamentoProcesso();` e o
endereço de escrita está na tela do nó. A unidade ativa é observada
na tela do processo (`lnkInfraUnidade`); o estado está no HTML
do nó raiz (`AbertoEm::Unidades` ou `AbertoEm::Sobrestado`).
A leitura não pode supor a instalação nem a unidade a partir de cache.

## Alternativas consideradas

1. **Barra assinada e situação lida** (escolhida): reutiliza os
   leitores de HTML e URL assinada já tipados, confere ID e unidade,
   nenhuma requisição extra na prévia.
2. **REST não observado**: não há rota demonstrada com equivalência
   de autenticação, estado e efeitos.
3. **Heurística textual**: decidir por ícone ou texto da barra
   permite falsos positivos e operação em outro processo.

## Comparação

| Critério | Barra assinada | REST | Heurística |
| --- | --- | --- | --- |
| Evidência nas sete instalações do Todos | sim | não | parcial |
| Identidade por ID assinado | sim | a provar | não |
| URL construida pela CLI | não | a provar | possivelmente |
| Prévia a frio | 3 GET | desconhecido | desconhecido |

## Decisão

Usar `arvore_processo::barra_do_processo` e
`arvore_processo::endereco` para extrair a ação do próprio nó
`PROCESSO`; reutilizar `abrir_para_escrever` sem o link da árvore
guardado para conferir a unidade. Não executar JavaScript, não usar
regex, não seguir 302, não construir links com `infra_hash`.

Entregues: superfície CLI, motivo obrigatório, verificação de
estado e URL da barra, prévia funcional sem POST, testes sintéticos
para identidade, duplicidade, autorização e orçamento HTTP.
`--confirmar` ainda recusa ANTES de ler arquivos: é preferível
a executar uma escrita sem formulário e reconciliação comprovados.

## Evidências verificáveis

- [Contrato](../../contracts/cli/processo-sobrestar.md):
  condições de entrada, estados recusados e subcódigos.
- [Matriz da operação](../../contracts/evidencias/processo-sobrestar.md):
  sete cassettes do Todos, não é execução do novo código.
- [Teste Rust da CLI](../../tests/cli_processo_sobrestar.rs):
  uso inválido e confirmação não implementada sem rede.
- [Testes do leitor](../../src/arvore_processo/tests_sobrestar.rs)
  e [testes de fluxo](../../src/processo_sobrestar/tests.rs):
  URL assinada e três GET exatos a frio.
- [Live](../validacao-live/2026-11-05-processo-sobrestar.md):
  NOT_RUN, zero requisições e nenhum resíduo.
- [Issue #269](https://github.com/franklinbaldo/ies/issues/269)
  e [PR Draft #279](https://github.com/franklinbaldo/ies/pull/279).

## Limitações e revisão

Confirmação, POST do formulário, URL real de retomada, leitura
posterior reconciliada, saída 3 e replay precisam de outra fatia.
Só permitir escrita em treinamento depois dos testes RED/GREEN e
prévia, uma única confirmação e releitura; registrar o resíduo no SEI.
Não declarar a issue concluída enquanto estes itens estiverem ausentes.
