---
type: AgentMemory
title: "Prévia live: o NUP conhecido deixou de ter unidade ativa observável"
description: "Segundo ensaio UnB da #264: saída 1 e unidade_consta_na_arvore=false em outro processo antes válido; a guarda impediu POST. Buscar processo na unidade ativa antes de nova prévia."
agent: raposa
timestamp: "2026-11-09T17:23:24Z"
tags: [ux, processo-atribuir, unb, treinamento, observacao, unidade]
---

# Para reencontrar

**Fato:** o [ensaio UnB #42662410727](https://github.com/franklinbaldo/ies/actions/runs/42662410727) usou outro NUP já validado em 06/11 e encontrou `situacao_da_atribuicao_nao_observada`, saída 1, quatro requisições e `unidade_consta_na_arvore=false`. Portanto o estado da **unidade ativa publicada pela tela** não coincide com o conjunto de unidades abertas da árvore nessa observação. Não houve `--confirmar`, atribuição, remoção nem retry. O NUP do ensaio anterior não foi repetido.

**Limite:** o confronto não revela se alguém transferiu o processo, se mudou a unidade da sessão ou qual foi a causa da falha anterior de outro NUP. O [relatório da PR #593](https://github.com/franklinbaldo/ies/pull/593) registra apenas a observação nova e a separação das três provas.

**Próximo passo:** antes de gastar outra prévia live, descobrir numa leitura segura um processo **atualmente aberto na unidade ativa e ainda sem atribuição**, evitando NUPs já recusados. A discussão de [Castor sobre medir de novo a junta](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6744993802) é uma boa analogia para a validade temporal: uma evidência antiga não se transporta automaticamente ao estado presente. Não é evidência adicional do SEI.
