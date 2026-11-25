---
type: AgentMemory
title: "Cassette UnB: 302 bloco_escolher sem corpo HTTP disponível"
description: "Inspeção saneada confirmou GET 200 → POST 302 → GET 200 e formulário original, mas a gravação não contém corpo bruto do POST; BDD-381-02 depende da prévia #694 e de releitura independente."
agent: polvo
timestamp: "2026-11-10T19:56:31Z"
tags: [blocos, documentos, cassettes, anti-ies, bdd, prova-causal]
---

# Retomar na próxima rodada

- **Fatos observados:** [Actions #42889943882](https://github.com/franklinbaldo/ies/actions/runs/42889943882) verificou hashes da UnB (152 interações). GETs 25/44/49/51 têm `frmBlocoEscolher`, `selBloco`, `chkDocumentosItem`, `hdnDocumentosItensSelecionados`. Sequência 49→50→51 = GET 200 → POST 302 (Location com ação `bloco_escolher`) → GET 200.
- **Lacuna:** cada requisição gravada contém `method`, `operation`, `operation_parameter` textual e `pathname`, não campos de corpo POST. O registro não comprova os parâmetros submetidos, inclusão persistida nem releitura de detalhe. Ver [comentário #381](https://github.com/franklinbaldo/ies/issues/381#issuecomment-6777743302); não equiparar inspeção estrutural a paridade semântica.
- **Continuidade:** [PR #694](https://github.com/franklinbaldo/ies/pull/694) do 🐘 continua aberta e ainda precisa comprovar segunda leitura no **mesmo SQLite** da prévia. Só depois integrar/iniciar BDD-381-02 com IES↔anti-IES↔SQLite, único POST e nova CLI de leitura; bilateral cassette e live seguem NOT_RUN.
- **Higiene:** [PR temporária #699](https://github.com/franklinbaldo/ies/pull/699) fechada sem merge, alterações e gatilho revertidos (diff vazio). Nenhum acesso ou escrita SEI. Visitei o [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388), sem publicação; conversa sobre estado parcial e identidade mantém relevância como analogia, não prova.
