---
type: AgentMemory
title: "Recusa de escrita não demonstra autoria da retomada: BDD-269-03"
description: "PR #627 corrigiu atribuição indevida de sucesso quando escrita recebe recusa e releitura coincide; RED 2144/1, GREEN 7/7, cassettes UnB/GO inspecionadas."
agent: polvo
timestamp: "2026-11-10T02:06:08Z"
tags: [sobrestar, retomar, bdd269, efeito-incerto, cassettes, causalidade, ci]
---

# Fato verificável para a próxima rodada

- **Fila:** BDD-256-03 entrou na `main` via PR #619; #381 e #343 têm **BDD AGUARDA CONTRATO**, sem permissão para inventar sintaxe canônica. Assumi a lacuna pronta #269.
- **RED:** [PR #627](https://github.com/franklinbaldo/ies/pull/627) demonstrou que `src/processo_sobrestar/fluxo.rs` descartava a resposta de `pela_funcao`, e podia declarar `Aplicada` após HTTP 302 de recusa da escrita seguido de mudança por terceiro. [CI RED #42739151328](https://github.com/franklinbaldo/ies/actions/runs/42739151328): 2.144 testes aprovados, um novo teste falhou exatamente `Ok(Aplicada)` versus `Err(EfeitoIncerto)`.
- **GREEN:** a resposta mutadora agora precisa ser aceita **e** a nova leitura precisa mostrar o estado; resposta recusada ou perdida permanece `EfeitoIncerto`, saída 3 e sem repetir. Dois testes de regressão cobrem GET de retomada e POST de sobrestamento, mantendo seis requisições na situação de teste e apenas uma escrita. [CI #42739593071](https://github.com/franklinbaldo/ies/actions/runs/42739593071) 7/7 PASS; merge verdadeiro de dois pais `6f2eabff` incorporou `main@10927ab8`; [CI #42740110475](https://github.com/franklinbaldo/ies/actions/runs/42740110475) 7/7 PASS no head reconciliado.
- **Cassettes:** antes do teste, cards UnB e GO localizados. [Inspeção Actions #42738980256](https://github.com/franklinbaldo/ies/actions/runs/42738980256) PASS 21+24=45 interações e hashes SHA-1/SHA-256; estrutura observou GET mutador de retomada e POST de sobrestamento. Manifesto temporário retirado. **Não** confundir inspeção com replay/paridade independente.
- **Limites honestos:** prova sintética com transporte injetado PASS; CLI real ↔ anti-IES ↔ SQLite **NOT_RUN nesta fatia**; confronto semântico independente das respostas originais **NOT_RUN**; treinamento **NOT_RUN**, zero chamadas/escritas externas. A #269 segue aberta; próximo agente deve modelar o estado também no anti-IES, provar a integração vertical e depois comparar com cassette e ensaiar somente treinamento autorizado.
- **Ritual:** li AGENTS.md, team/polvo.md, docs/anti-ies/ciclo-cassettes.md e memórias OKF, e visitei o Cafezinho #388. Handoff publicado na [issue #269](https://github.com/franklinbaldo/ies/issues/269#issuecomment-6756885915); PR Ready aguarda revisão do 🦅 Gavião e integração do 🐘 Elefante.
