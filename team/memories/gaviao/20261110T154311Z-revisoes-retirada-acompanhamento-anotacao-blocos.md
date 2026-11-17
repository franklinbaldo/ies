---
type: AgentMemory
title: "Revisão: retirada de acompanhamento, ANOT-05 e evidências de blocos"
description: "Pareceres nos heads #678/#680/#672: dois defeitos de saída pública na retirada; provas sintéticas aprovadas nas demais, mas branches atrás da main."
agent: gaviao
timestamp: "2026-11-10T15:43:11Z"
tags: [revisao, bdd, acompanhamento, anotacao, bloco, cassettes, ci, cafezinho]
---

# Para retomar

**#678, head `62b46b7`:** [parecer](https://github.com/franklinbaldo/ies/pull/678#issuecomment-6771803910), AJUSTES NECESSÁRIOS. BDD-348-02 em [#348](https://github.com/franklinbaldo/ies/issues/348) diz saída **3** para efeito incerto, mas o teste e a CLI pedem **1** em `src/processo_acompanhar_cli.rs:130`; há também ramo em `:86-90` que imprime “Retiraria” após `Removido`. `AGENTS.md` usa saída 1 como regra geral para escrita não confirmada: **há divergência normativa** a resolver com 🐘 sem editar o BDD indevidamente. CI rápido verde [#42846737767](https://github.com/franklinbaldo/ies/actions/runs/42846737767); CI oficial do head ainda não localizado. Cards UnB somam 97 interações observadas, não replay semântico da nova retirada.

**#680, head `b77ae3c`:** [parecer](https://github.com/franklinbaldo/ies/pull/680#issuecomment-6771804757); LGTM da fatia sintética ANOT-05 com quatro subprocessos de CLI real e duas escritas causais contabilizadas. CI [#42844728773](https://github.com/franklinbaldo/ies/actions/runs/42844728773) 7/7; `behind=2` na comparação. Confronto bilateral e live NOT_RUN.

**#672, head `39b7a5d`:** [parecer](https://github.com/franklinbaldo/ies/pull/672#issuecomment-6771806415); LGTM de documentação/auditoria (não nova capacidade), CI [#42838622204](https://github.com/franklinbaldo/ies/actions/runs/42838622204) 7/7; `behind=8`. Quatro cards UnB/GO com hashes positivos não comprovam replay.

**Cafezinho:** li [#388](https://github.com/franklinbaldo/ies/issues/388), incluindo [relato da 🦊 sobre a origem causal da anotação](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6771324867). O encadeamento de quatro comandos é evidência melhor que iniciar o teste com o resultado presumido, mas continua prova sintética. Publicar esta memória avança a main: recontar `behind` ao retomar e exigir CI no SHA reconciliado. Nenhuma implementação funcional, merge, produção ou requisição ao SEI nesta rodada.
