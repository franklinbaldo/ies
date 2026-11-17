---
type: AgentMemory
title: "Assinatura tipada: CI do SHA e fechamento da lacuna"
description: "A PR #670 foi integrada com CI oficial 7/7; a issue #563 foi encerrada; confronto bilateral com cassettes e live não foram executados nesta fatia."
agent: polvo
timestamp: "2026-11-10T14:14:47.219Z"
tags: [documento, assinatura, estados-tipados, ci, cassettes, anti-ies]
---

# Para retomar

- **Fato:** [PR #670](https://github.com/franklinbaldo/ies/pull/670) integrada pelo merge [`68e7474`](https://github.com/franklinbaldo/ies/commit/68e7474c025760f714938417620b19a5bfcb43e2), head `51f0f2f8`; [CI oficial #42830556507](https://github.com/franklinbaldo/ies/actions/runs/42830556507) **7/7 PASS** e rápido #42830528227 **PASS**. Registrei a evidência e encerrei a [issue #563](https://github.com/franklinbaldo/ies/issues/563#issuecomment-6769583628) como concluída. Um CI intermediário cancelado não é prova.
- **Limites:** preservado o comando `ies documento assinar`, com estados internos excludentes; reexecutadas as regressões existentes no CI. **Nenhum E2E novo nem RED funcional nesta refatoração; confronto semântico bilateral da cassette UnB e treinamento/live NOT_RUN nesta rodada**, sem escrita externa.
- **Fila observada:** [#60](https://github.com/franklinbaldo/ies/issues/60) está em PR da 🐢; [#638](https://github.com/franklinbaldo/ies/issues/638) em PR da 🦊; [#381](https://github.com/franklinbaldo/ies/issues/381) e [#343](https://github.com/franklinbaldo/ies/issues/343) aguardam contrato unificado e BDD do 🐘. Não iniciar implementação dessas hipóteses sem cenário pronto nem invadir as frentes ativas.
- **Cafezinho:** a conversa sobre silêncio, vazio e estado não observado ([#388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6767001683)) reforça que um enum mais preciso restringe possibilidades, mas não substitui a conferência do SEI original. Esta é uma conclusão de modelagem, não nova evidência live.
