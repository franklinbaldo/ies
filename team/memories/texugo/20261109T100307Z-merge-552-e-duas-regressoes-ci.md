---
type: AgentMemory
title: "Histórico documental integrado, falhas concretas em duas PRs"
description: "Integrei #552 com CI 7/7; #551 reduziu requests mas expôs seis falhas de criação documental; #553 parou em Clippy 103/100; #526 ainda precisa de SQLite v20."
agent: texugo
timestamp: "2026-11-09T10:03:07Z"
tags: [antichurn, integracao, ci, anti-ies, sqlite, cli, cassettes]
---

# Para retomar

**Merge confirmado:** [#552](https://github.com/franklinbaldo/ies/pull/552) entrou na `main` pelo merge `aeab60d`, head `1aea0b5`, [CI oficial #42597450901](https://github.com/franklinbaldo/ies/actions/runs/42597450901) com **7/7 success** e CI rápido verde. Recusa sintética HTTP/SQLite de `id_protocolo` divergente em versões/conteúdo. **Sem novo E2E CLI** dessa subcausa; confronto semântico independente com cassette e treinamento `NOT_RUN`.

**#551:** no head `0f90133` o novo `href` do Controle substituiu o POST de pesquisa por navegação direta, reduzindo de 5→4 requisições na prévia e 10→8 na confirmação. A 🦫 corrigiu os orçamentos no head `7d3acfd`, mas [CI oficial #42597514424](https://github.com/franklinbaldo/ies/actions/runs/42597514424) terminou **6/7**, agora com **seis falhas em `aies_documento_criar_cli`**: a fixture mantém tabelas R/D vazias e não há link para o NUP. [Orientação objetiva](https://github.com/franklinbaldo/ies/pull/551#issuecomment-6737275752): preservar fallback de pesquisa se o processo não está na caixa ou coerência da linha se observada, sem falsear fixture; revalidar depois de merge verdadeiro da main. **Não inverter a melhoria de requests por reflexo.**

**#553:** [CI #42597536975](https://github.com/franklinbaldo/ies/actions/runs/42597536975) 6/7; Clippy `too_many_lines`, nova função BDD-271-02 com 103/100 linhas. [Orientação](https://github.com/franklinbaldo/ies/pull/553#issuecomment-6737288418) para extrair apenas conferência coesa, sem `#[allow]` e sem reescrever a fatia.

**#526:** branch da 🦉 ainda Draft/conflitante em `5188493`; criação causal sintética previamente comprovada no SHA antigo, migrar criação para **v20** depois de v18 atribuição e v19 ciência e reconciliar main, sem recomeçar. Cassette semantic/live continuam `NOT_RUN`.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), inclusive a discussão da 🦊 sobre mensageiro, estado final e autoria. A distinção inspira cautela causal, não aumenta prova externa. Sem produção, novas credenciais ou subcódigos.
