---
type: AgentMemory
title: "Replays efetivos de #777/#781 aprovados; falso verde no replay #794"
description: "CI oficial 7/7 e replay pertinente de anexação e troca confirmados, mas branches defasadas; #794 aceita qualquer 302 e autovalida texto da releitura, impedindo integração."
agent: elefante
timestamp: "2026-11-11T15:12:35Z"
tags: [integracao, cassette, replay, ci, ablação, fila, cafezinho]
---

# Retomar na próxima rodada

- **Sem merge nesta rodada.** Reli integralmente `AGENTS.md` e `team/elefante.md` da `main`, cinco memórias próprias, comentários do Cafezinho [#388](https://github.com/franklinbaldo/ies/issues/388) e [roadmap #56](https://github.com/franklinbaldo/ies/issues/56). A `main` consultada era `e98ed56b7166817e88d6ba016d2267f574a2cde7`.
- **#777:** HEAD `022554a5`, `behind=6`; [CI #43039971648](https://github.com/franklinbaldo/ies/actions/runs/43039971648) **7/7 PASS**. Log do job `replay unidade.retornos`: `RUSTUP_TOOLCHAIN=1.99.1 scripts/replay acervo`, **495 cards/29 ZIPs hash PASS**, **32 acervo + 18 cassette_ PASS**. O teste `processo_anexar::acervo::replay_de_processo_anexar_no_acervo` efetivamente confronta 10 leituras originais e 10 respostas HTTP do anti-IES; o 503 pós-POST é hipótese sintética, externo/live **NOT_RUN**. Precisa merge verdadeiro de main no branch, revisão e CI no novo HEAD.
- **#781:** HEAD `8ddd45ea`, `behind=8`; [CI #43035581862](https://github.com/franklinbaldo/ies/actions/runs/43035581862) **7/7 PASS**, com `unidade_trocar::acervo::replay_de_unidade_trocar_no_acervo` efetivo, 32 cassettes lidas, **31 acervo + 18 cassette_ PASS**. Refatoração de testes, não prova transição anti-IES. Precisa sincronização e revisão.
- **#778/#790/#792/#794:** respectivamente HEADs `06cca418`, `2b0f0950`, `d9baf9f9`, `487871e6`, com CI **6/7** e replay longo em andamento; atrás da main 4/2/1/5 commits. #526 e #784 seguem Draft.
- **Achado verificável na [#794](https://github.com/franklinbaldo/ies/pull/794#issuecomment-6799510624):** `processo_comentar_web/acervo.rs` trata qualquer GET 302 de `comentario_listar` como "lista vazia" sem inspecionar `Location`. O mesmo replay passa `&item.texto` extraído da releitura a `conferir_efeito`, sem oráculo independente do conteúdo submetido. Solicitei falsificação do 302 e limite do confronto; a variante `A` é inferida de HTML e deve ser rotulada. Replays em CI final são necessários.
- **Coordenação:** [#56](https://github.com/franklinbaldo/ies/issues/56#issuecomment-6799536497) registra a fila e as evidências, sem abrir issue duplicada nem editar branch alheia.
- **Cafezinho:** li conversas recentes sobre `body_base64` corrompido, replays que precisam de asserts e preservação da causa pós-escrita. Nenhuma informação live foi inferida. Sem BDD/Gherkin, novos subcódigos, segredos, retry, produção ou Jatobá.

**Próximo ciclo:** reler instruções/ficha/memórias/Cafezinho; conferir novos HEADs, CI 7/7, replay pertinente e `behind=0`; reavaliar correção concreta da #794 antes de integrar. Priorizar #777 e #781 sincronizadas, e #790/#792 conforme gate e revisão.
