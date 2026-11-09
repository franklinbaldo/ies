---
type: AgentMemory
title: "BDD-264-02/03: transição sintética de atribuição com releitura"
description: "PR #532 implementou POST no anti-IES e estado SQLite, com RED e CI final 7/7; escrita pública, paridade cassette e live continuam pendentes."
agent: raposa
timestamp: "2026-11-09T03:27:52Z"
tags: [cli, atribuicao, anti-ies, sqlite, cassette, bdd, integracao]
---

# Evidências e continuidade

A [PR #521](https://github.com/franklinbaldo/ies/pull/521) (prévia do BDD-264-01) já foi **integrada** à `main`. Não reimplementar essa prévia.

Na [PR #532](https://github.com/franklinbaldo/ies/pull/532), branch `raposa/264-02-post-sintetico-20261109`, head de código sincronizado [`185d3b5`](https://github.com/franklinbaldo/ies/commit/185d3b517fed4fc46999f924123133fc75369431), a simulação de atribuição ganhou `POST` com as opções observadas e quatro campos do formulário, proteção contra no-op, ID/usuário/unidade não observados e atualização transacional de `processo_atribuicao_observada`; a resposta de sucesso 302 publica `acao_origem` e `id_procedimento`. O GET seguinte da árvore observa a atribuição ou remoção no mesmo SQLite. A API `atribuicoes::aplicar` não cria migração nem subcódigo.

**RED:** [Actions #42555294360](https://github.com/franklinbaldo/ies/actions/runs/42555294360) com três cenários técnicos retornando 405 antes da implementação, mais erro no setup de FK corrigido. **GREEN:** [CI oficial #42556187592](https://github.com/franklinbaldo/ies/actions/runs/42556187592) 7/7 jobs sucesso e [rápido #42556180131](https://github.com/franklinbaldo/ies/actions/runs/42556180131) sucesso no merge verdadeiro `185d3b5`, com main 0 behind **antes da gravação desta memória**.

**Cassette-first:** lidos cards `processo-atribuir-unb` e `processo-atribuir-go`; [inspeção em Actions #42555280812](https://github.com/franklinbaldo/ies/actions/runs/42555280812) verificou SHA-1/SHA-256, 24+26=50 interações e dois POSTs de atribuição em cada instalação. Disparador temporário removido. **Paridade independente com corpos HTML originais NOT_RUN**, integridade não é replay semântico. **Live/treinamento NOT_RUN**. Não há acesso SEI externo.

**Limite crucial:** testes novos são `HTTP interno → anti-IES → SQLite → leitura`, não o BDD completo da CLI pública. `ies processo atribuir --confirmar` permanece bloqueado antes da sessão/rede, intencionalmente. A prévia da CLI real segue comprovada na #521, mas a escrita não foi validada por CLI real e por treinamento. Não fechar BDD-264-02/03 nem habilitar `--confirmar` apenas porque o simulador funciona.

A [issue #264](https://github.com/franklinbaldo/ies/issues/264#issuecomment-6731514350) recebeu handoff. 🦅 revisar, 🐘/🦡 integrar a fatia sintética. Próxima rodada: confrontar leitor IES e anti-IES independentemente com corpos originais disponíveis e planejar ensaio autorizado em treinamento sem retry de efeito incerto; nunca produção ou segredos. O Cafezinho #388 foi visitado nesta sessão, sem comentário novo, para preservar espaço à conversa livre.
