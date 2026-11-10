---
type: AgentMemory
title: "Três integrações sem loop de sincronização, uma refatoração ampla em revisão"
description: "Integrei #666, #667 e #669 com CI 7/7 no head e compatibilidade de arquivos; #671 está verde mas exige revisão adversarial do contrato público."
agent: texugo
timestamp: "2026-11-10T14:52:23.623Z"
tags: [antichurn, integracao, ci, cassettes, anotacao, acompanhamento, erros-tipados]
---

# Ponto de retomada

**Fatos:** após ler AGENTS, ficha, guia cassette-first e cinco memórias, visitei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388). Integrei as PRs [#666](https://github.com/franklinbaldo/ies/pull/666), merge `43ee48d5`, head `51318b5`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42835433250); [#667](https://github.com/franklinbaldo/ies/pull/667), merge `f7e49e5e`, head `c452e96`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42833649579); e [#669](https://github.com/franklinbaldo/ies/pull/669), merge `c13e1ea8`, head `b7f0f2f`, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42837573998). CI rápido também verde nos três heads. A comparação com a main mostrou que as defasagens da #666/#667 eram apenas memórias OKF disjuntas; os arquivos da #669 tampouco sobrepunham as duas integrações. GitHub aceitou três merges com SHA esperado, sem rebase/force-push e sem CI pós-merge. Main observada ao fim: `c13e1ea8d76caf08c0caf28b6196297c5fcd2212`.

**Provas separadas:** #666 contém PASS **sintético** da CLI real ↔ anti-IES ↔ SQLite v35, com lista vazia conhecida distinta de lista não observada; cassette UnB teve integridade conferida, sem confronto bilateral específico. #667 contém PASS **sintético** da retirada de anotação (prévia sem POST, confirmação única e releitura; saída 3 após falha de comprovação), sem replay bilateral. #669 contém **PASS parcial com cassettes** originais GO/UFRJ para seleção/identidade/árvore; recusa 404/405 é sintética e não há novo E2E CLI nessa variante. Live/SEI permanece NOT_RUN em todos.

**Pendências:** [#671](https://github.com/franklinbaldo/ies/pull/671), head `42f9eed`, já tinha [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42837609481) e rápido verdes, mas muda 327 arquivos e ainda não tinha revisão independente 🦅 registrada. Entre as três integrações novas e seus arquivos, só `src/processo_anotar_cli.rs` coincide por nome; GitHub reportava `mergeable_state=clean`. [Registrei orientação na PR](https://github.com/franklinbaldo/ies/pull/671#issuecomment-6770521546): revisar representação pública de erro, preservação de causas tipadas e efeito incerto antes de mesclar a refatoração transversal; não reconstruir a fatia por defasagem nominal. [#526](https://github.com/franklinbaldo/ies/pull/526) segue Draft conflitante: preservar E2E existente de criação e escolher migração disponível após conciliar; v35 passou a estar ocupada. [#672](https://github.com/franklinbaldo/ies/pull/672) é Draft recente.

**Cafezinho:** a distinção recorrente entre ausência de evidência e ausência observada também se aplica a CI: queued não é verde e mudança de memória não é conflito de código. Nenhuma requisição ao SEI, escrita em produção, novo subcódigo, workflow novo ou gate acrescentado nesta rodada.
