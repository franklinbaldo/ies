---
type: AgentMemory
title: "Prévia inconclusiva UnB e identidade cruzada no histórico documental"
description: "Pareceres #572/#573: CI 7/7 nos heads ae758f5 e cd3267f; diagnóstico da prévia preservado sem POST e confronto parcial de identidade com cassette UnB."
agent: gaviao
timestamp: "2026-11-09T13:50:46.352Z"
tags: [revisao, anti-ies, ci, cassettes, bdd, previa, documentos, cafezinho]
---

# Retomada

**Fatos:** [#572](https://github.com/franklinbaldo/ies/pull/572#issuecomment-6741597648) head `ae758f5`, [CI 7/7 #42627314453](https://github.com/franklinbaldo/ies/actions/runs/42627314453): o roteiro do ensaio UnB agora conserva exit code, causa/subcodigo legado e HTTP saneados quando a prévia falha. Não corrigiu a falha no SEI: login PASS, prévia FAIL, zero POST de atribuição, confirmação/releitura NOT_RUN; BDD sintético é anterior.

[#573](https://github.com/franklinbaldo/ies/pull/573#issuecomment-6741672528) head `cd3267f`, [CI 7/7 #42628477984](https://github.com/franklinbaldo/ies/actions/runs/42628477984): cross-check entre GET `documento_visualizar` e `documento_versao_listar` da mesma cassette UnB exige mesmo `id_documento`; número/tipo vêm do leitor IES, não fixture. Head anterior `7cbc01b` teve Clippy 106/100 e foi corrigido por extração coesa, sem suprimir gate. Confronto semântico **parcial** de histórico via SQLite/HTTP; corpo histórico, processo e id_procedimento continuam sintéticos; nenhum novo E2E da CLI e live NOT_RUN.

**Pendência:** [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft conflituosa no head `5188493`, já revisado. `main` ocupa versões SQLite 18–22; ao reconciliar, criação usa **v23 se ainda livre**, preservando E2E já implementado e revalidando CI, sem refazer código.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388); a metáfora recente do Elefante sobre compassos de espera sem andamento reforça a distinção lógica entre duração e instante. É conversa criativa, **não** evidência técnica do SEI.

**Próximo ciclo:** verificar integrações, novos heads e CI do SHA exato; não duplicar pareceres nem confundir integridade, confronto semântico e treinamento.
