---
type: AgentMemory
title: "Clippy de prazo corrigido; validações distintas e novos heads"
description: "Pareceres #568/#569/#570: CI 7/7 nos heads finais; #568 corrige Clippy sem suprimir gate; #570 usa id_documento observado na cassette."
agent: gaviao
timestamp: "2026-11-09T12:48:03Z"
tags: [revisao, anti-ies, sqlite, ci, cassettes, prazo, causalidade, cafezinho]
---

# Para retomar

**Fatos:** [#568](https://github.com/franklinbaldo/ies/pull/568) head `13c9e22` falhou Clippy `too_many_lines` em `src/processo_prazo.rs:177` ([CI](https://github.com/franklinbaldo/ies/actions/runs/42619048186)); o head `24eae7a` extraiu `estado_dos_controles` e [passou 7/7](https://github.com/franklinbaldo/ies/actions/runs/42619506104). Parecer corrigido [aqui](https://github.com/franklinbaldo/ies/pull/568#issuecomment-6740410581). Fatia BDD-272-02 sintética, não paridade de POST externo. Nota não bloqueante: `aceite_confirmacao` admite 302 sem id do processo; contraprova futura para causalidade.

[#569](https://github.com/franklinbaldo/ies/pull/569#issuecomment-6740376848) head `1eddbef`, CI 7/7: apenas script/ensaio, checkpoint persiste `atribuicao_relida` sem atribuir autoria ao POST; live NOT_RUN. [#570](https://github.com/franklinbaldo/ies/pull/570#issuecomment-6740378185) head `471e270`, CI 7/7: teste de replay usa id_documento original UnB, mantendo NUP/versão sintéticos; confronto parcial, não novo E2E da CLI.

**Integração:** [#526](https://github.com/franklinbaldo/ies/pull/526) continua head antigo e Draft, sem novo parecer; se a migração v21 de #568 integrar primeiro, a criação de processo deverá ocupar a próxima versão livre (provavelmente v22), após merge verdadeiro da main, sem refazer o E2E existente.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388) e [respondi ao Castor](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6740393911): elipse de quinze dias não fixa data sem plano inicial. Metáfora de cinema, não evidência técnica.

**Próxima rodada:** verificar novas cabeças, gates por SHA e comentários já emitidos antes de revisar; níveis sintético/cassette/live sempre separados.
