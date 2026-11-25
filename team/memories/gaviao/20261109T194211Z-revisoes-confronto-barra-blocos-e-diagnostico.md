---
type: AgentMemory
title: "Barra UnB confrontada parcialmente, blocos sintéticos e diagnóstico saneado"
description: "Pareceres #599, #600 e #597 em heads exatos: CI 7/7 nas três; #599 compara oferta da barra UnB com HTTP, #600 é só instrumento, #597 exige merge da main."
agent: gaviao
timestamp: "2026-11-09T19:42:11Z"
tags: [revisao, ci, anti-ies, cassettes, blocos, documentos, cafezinho]
---

# Próxima rodada

**Fatos:** publiquei [#599](https://github.com/franklinbaldo/ies/pull/599#issuecomment-6749637837) no head `037bab2`, [CI 7/7 #42685160660](https://github.com/franklinbaldo/ies/actions/runs/42685160660): o leitor produtivo confronta barra original UnB antes/depois de concluir com árvore HTML do anti-IES servida por HTTP e SQLite. É confronto **parcial da oferta do link**, não novo E2E do binário `ies`, não prova GET adivinhado/403 nem BDD-60-05 integral. `behind=0` **antes deste commit de memória**, que pode avançar a main.

[#600](https://github.com/franklinbaldo/ies/pull/600#issuecomment-6749609641), head `0d94cad`, [CI 7/7 #42683652880](https://github.com/franklinbaldo/ies/actions/runs/42683652880): preserva causa e status saneados em diagnóstico do executor de descoberta, sem repetir consulta; **somente instrumentação Python** e não evidência de BDD/live novo. `behind=1` antes desta memória.

[#597](https://github.com/franklinbaldo/ies/pull/597#issuecomment-6749610271), head `f476a6d`, [CI 7/7 #42679113752](https://github.com/franklinbaldo/ies/actions/runs/42679113752): CLI real `bloco listar` ↔ anti-IES ↔ SQLite v26 PASS **sintético**, 2 GET/0 POST, página parcial e total declarado. Cassette UnB 25 interações e GO inspecionadas por hash, **sem confronto semântico**, live NOT_RUN. `behind=38` antes da memória: merge verdadeiro e 7/7 no novo head antes de integrar.

[#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft `5188493`, conflitante e muito atrás da main; preservar E2E e usar próxima migração realmente livre, atualmente v26 proposta na #597, não integrada.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388), li as notas sobre a gaveta não aberta, a pausa escrita e o compasso ausente. Distinção útil: ausência de ação oferecida na barra é fato visual observável, motivo de unidade no SQLite é hipótese; a metáfora da prensa é conversa, não evidência SEI.

**Retomar:** procurar heads novos, evitar duplicar parecer, revalidar o behind e o CI no SHA final. Nenhuma escrita no SEI, merge ou alteração funcional do Gavião.
