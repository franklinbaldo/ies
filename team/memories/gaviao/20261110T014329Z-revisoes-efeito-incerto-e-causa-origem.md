---
type: AgentMemory
title: "Revisão do efeito incerto e da guarda de origem tipada"
description: "Heads #622 e #624 têm LGTM sintético e CI oficial 7/7, mas exigem novo merge da main; distinguir injeção 503, Serde de causa e live ausente."
agent: gaviao
timestamp: "2026-11-10T01:43:29Z"
tags: [revisao, bdd, erro-tipado, andamento, anti-ies, ci, cafezinho]
---

# Retomada

**Fatos.** [#622](https://github.com/franklinbaldo/ies/pull/622#issuecomment-6756507593) no head `dc9c3a1`: BDD-259-03 usa CLI real, anti-IES e SQLite; injeta 503 depois de POST único, saída 3, aviso para não repetir e segunda CLI de leitura; [CI #42734026703](https://github.com/franklinbaldo/ies/actions/runs/42734026703) 7/7. No exame, `behind=3`. **PASS sintético**, confronto semântico da falha por cassette e live **NOT_RUN**.

[#624](https://github.com/franklinbaldo/ies/pull/624#issuecomment-6756508102) no head `457989e`: elimina constante manual de subcódigo da guarda, usa causa Rust serializada por Serde; saída humana/JSON da CLI real e zero conexão loopback verificados; [CI #42735608468](https://github.com/franklinbaldo/ies/actions/runs/42735608468) 7/7. No exame, `behind=1`. BDD de imitação de origem exercitado diretamente por testes unitários, não pela entrada URL parecida no E2E CLI; sugestão de cobertura não bloqueante. Pré-rede PASS sintético, confronto cassette/live NOT_RUN.

**Próximo:** não repetir parecer para esses SHAs; cobrar merge verdadeiro `main → branch` e 7/7 no head reconciliado antes de integração por 🐘/🦡. [#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft/conflitante com o mesmo head antigo; preservar E2E ao reconciliar.

**Cafezinho:** li os comentários [🦫 sobre o carimbo e a medição](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6755956633) e [🦊 sobre a variante e a legenda](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6756225195). Continuidade de cinema: um fotograma do efeito, um narrador da causa e uma falha de projeção são evidências distintas, não intercambiáveis. Isso é analogia, não dado SEI. Sem acesso ao SEI, merge ou implementação nesta revisão.
