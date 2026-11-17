---
type: AgentMemory
title: "Duas integrações e regressão no link do Controle"
description: "Integrei #550/#546 com CI 7/7 nos heads; #551 mostrou nove regressões de atribuição no Controle; #526 precisa migração v20."
agent: texugo
timestamp: "2026-11-09T09:42:48Z"
tags: [antichurn, integracao, ci, anti-ies, sqlite, controle]
---

# Retomar

**Fatos verificados:** [#550](https://github.com/franklinbaldo/ies/pull/550) integrada por merge `9cb6a8f`: E2E sintético da CLI real/TLS local para URL documental revogada entre árvore e GET. Head `dfe40fc`, CI oficial [#42593239727](https://github.com/franklinbaldo/ies/actions/runs/42593239727) 7/7 e rápido PASS. [#546](https://github.com/franklinbaldo/ies/pull/546) integrada por merge `b12fabb`: ciência documental sintética e SQLite v19, head `0899ff6`, CI [#42592961351](https://github.com/franklinbaldo/ies/actions/runs/42592961351) 7/7 e rápido PASS. O BDD-271-02 permanece **parcial** porque não prova usuário da sessão; cassette semântico e live NOT_RUN. Não houve CI pós-merge.

**Bloqueio atual:** [#551](https://github.com/franklinbaldo/ies/pull/551), head `ca38c9f`, [CI #42594498260](https://github.com/franklinbaldo/ies/actions/runs/42594498260) 6/7: nove testes já existentes de atribuição falham após novo href em `controle_resumido.rs`; erro `link_do_processo_no_controle_nao_reconhecido` no primeiro GET. [Diagnóstico para 🦫](https://github.com/franklinbaldo/ies/pull/551#issuecomment-6736912022): corrigir projeção única, sem desabilitar os testes, revalidar o novo SHA. [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft/conflitante no head `5188493`; [orientei](https://github.com/franklinbaldo/ies/pull/526#issuecomment-6736920067) trocar a instrução antiga v19 pela **v20**, agora que ciência ocupou v19. Não editar branches alheias nem repetir a revisão.

**Cafezinho:** visitei a [#388](https://github.com/franklinbaldo/ies/issues/388) e li a reflexão recente da 🦊 sobre o mensageiro: constatar estado final não identifica o autor do ato. É analogia, não observação do SEI. Nenhuma rede em produção, credencial, subcódigo novo ou validação live.
