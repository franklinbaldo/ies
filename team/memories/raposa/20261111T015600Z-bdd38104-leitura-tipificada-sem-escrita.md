---
type: AgentMemory
title: "BDD-381-04: distinguir POST de consulta da escrita e ler bloco tipado"
description: "PR #737 Ready no SHA fbd10506, 7/7 CI oficial após merge verdadeiro com #738. Prévia processual e releitura por CLI real no anti-IES/SQLite PASS sintético; confronto PEN1 e live NOT_RUN; aguarda revisão e integração."
agent: raposa
timestamp: "2026-11-11T01:56:00Z"
tags: [ux, bloco, bdd-381-04, anti-ies, sqlite, cassette-first, ci, cafezinho]
---

# Retomar

**Fato:** BDD-381-04 da [#381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6775528602) está na [PR #737](https://github.com/franklinbaldo/ies/pull/737), head `fbd10506409ad1d9ce9dea0e703c72fc76614089`, Ready e mergeável. `ies bloco N` resolve a identidade pelas listas de assinatura/interno/reunião realmente oferecidas, recusa colisão e lista parcial que não exclua colisão. A prévia `ies --json bloco incluir N --processo NUP` abre o processo por leitura, não inclui, e outra execução de `ies bloco N` prova ausência do NUP no detalhe do mesmo SQLite, com projeção tipada do anti-IES. `--confirmar` processual continua recusado até BDD-381-05.

**Gates:** [CI oficial #42941958023](https://github.com/franklinbaldo/ies/actions/runs/42941958023) **7/7 PASS** e [rápido #42941898152](https://github.com/franklinbaldo/ies/actions/runs/42941898152) **PASS**, ambos no SHA final. A branch incorporou por merge verdadeiro a [#738](https://github.com/franklinbaldo/ies/pull/738), sem rebase/force-push, respeitando a ação tipada `AcaoDoSei` e o novo lint de produção. No primeiro CI, falharam Clippy e duas asserções; isso **não comprova RED comportamental**. O GET/POST da pesquisa rápida é somente leitura, portanto o teste passou a proibir **POST mutador**, não todo verbo POST. Outros testes ganharam o orçamento correto após a resolução de duas listas.

**Cassette-first:** card [PEN1](https://github.com/franklinbaldo/ies/blob/main/cassettes/pen1/bloco-interno-processos-pen1--69795bb274d6.md) tem 41 interações (34 GET/7 POST), hashes anteriormente verificados no [Actions #42878787801](https://github.com/franklinbaldo/ies/actions/runs/42878787801). CLI Rust ↔ TLS local ↔ anti-IES ↔ SQLite **PASS sintético**; confronto semântico bilateral PEN1 **NOT_RUN**; treinamento/live **NOT_RUN**. Nenhum SEI externo, credencial, escrita de produção ou novo subcódigo.

**Coordenação:** [handoff](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6784712925); visita ao [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6784477552). Última comparação mostrava a PR mergeável, 18 commits atrás da `main`, sem sobreposição dos arquivos modificados na última rodada. **CONTINUAR:** 🦅 revisão independente do SHA; 🐘/🦡 checam sincronização e integram; iniciar BDD-381-05 apenas após isso, sem transformar o PASS sintético em prova da instalação real.
