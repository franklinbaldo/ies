---
type: AgentMemory
title: "Presença de marcador não é prova de retirada oferecida; PR #642"
description: "Corrigida a falsa indicação de disponibilidade de remoção, com prévia E2E CLI↔anti-IES↔SQLite; cassettes UnB/GO por hash e CI final em avaliação."
agent: raposa
timestamp: "2026-11-10T05:32:40Z"
tags: [ux, processo-marcar, marcador, bdd, anti-ies, sqlite, cassette-first, testes]
---

# Para retomar

**Fato:** Na [PR #642](https://github.com/franklinbaldo/ies/pull/642), a saída JSON de `ies processo marcar --retirar` deixou de usar apenas a presença do marcador como prova de que sua remoção é oferecida. Enquanto o formulário de remoção não for validado, `retirada_possivel=false` mesmo que `nada_a_fazer=false`. A confirmação continua fechada antes de ler sessão, com orientação para executar sem `--confirmar` e obter a prévia. Sem novo subcódigo nem relaxamento de segurança.

**BDD-255-02 parcial:** o novo cenário E2E usa a CLI Rust real por HTTPS loopback, anti-IES e SQLite efêmero v31: retira somente na prévia, consulta novamente de forma independente e confirma que o marcador e seu texto persistiram, com zero POST. Saídas humana e JSON examinadas. A implementação do POST real, seu aceite causal e releitura pós-escrita **não** foram feitos. A PR #637/BDD-255-03 já está integrada.

**Cassette-first:** [inspeção GitHub Actions #42761750568](https://github.com/franklinbaldo/ies/actions/runs/42761750568) conferiu SHA-1/SHA-256 de dois cards UnB/GO de escrita de marcadores: 160 interações, 7 GET de gerenciamento e 1 POST de remoção por instalação. O manifesto temporário foi retirado. Esta é prova de integridade e estrutura, **não** replay semântico dos corpos, identidade do POST ou observação live. Treinamento/live NOT_RUN; produção não acessada. O possível resíduo da #264 não foi tocado.

**Validação e evolução:** PR iniciada sobre `main@0222f688`, depois reconciliada por merge verdadeiro de dois pais `a2babef3` com `main@4e09f6a`, preservando a refatoração de causas tipadas feita paralelamente. O teste UX foi escrito antes da alteração, mas o primeiro gate falhou em Rustfmt antes da execução do teste, portanto **não chamar esse gate de RED observado**. O [CI rápido #42762592949](https://github.com/franklinbaldo/ies/actions/runs/42762592949) passou no head `f91dcc0`. O [CI oficial #42762179028](https://github.com/franklinbaldo/ies/actions/runs/42762179028) falhou exclusivamente no Clippy: `executar` com 101/100 linhas; extraí a checagem humana e reexecutei o [CI oficial #42762606568](https://github.com/franklinbaldo/ies/actions/runs/42762606568), **em andamento ao gravar esta memória**. Conferir gates no head final.

**Cafezinho:** li os comentários recentes e acrescentei [a distinção de UX na #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6759810862).

**CONTINUAR:** confirmar CI oficial 7/7 no último SHA, sincronizar com `main` após esta memória, revisar com 🦅 Gavião e encaminhar integração a 🐘 Elefante/🦡 Texugo. Para concluir BDD-255-01/02, observar e reproduzir formulários de cadastro/remoção desde as cassettes, gerar POST único pelo formulário servido, exigir aceite causal e releitura independente, documentando resíduo. Não experimentar em produção, nem repetir operação de efeito incerto.
