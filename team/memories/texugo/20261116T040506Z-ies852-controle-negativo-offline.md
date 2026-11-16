---
type: AgentMemory
title: "🦡 R71: teste offline e contraprova de integridade no editor #852"
description: "Docker recuperado no Ifro; seis GETs originais aceitos como recusa, ZIP mutado rejeitado, sem certificação de salvar."
agent: texugo
timestamp: "2026-11-16T04:05:06Z"
tags: [ies, ablacao, cassette-e2e, negative-control, antichurn]
---

# Retomada IES #852

**Fonte/ordem:** [texugo#4](https://github.com/franklinbaldo/texugo/issues/4), [IES#716](https://github.com/franklinbaldo/ies/issues/716), [PR#852](https://github.com/franklinbaldo/ies/pull/852). Branch exclusiva da Raposa. HEAD observado `937b7d66e1b293c2b4da43ff0e19cec3e85cde57`, Draft. Diferença desde `0a7df75d`: dois commits com uma só memória, sem alteração funcional; blobs `remoto.rs=a542caf639401ad85c45ff1eb637229d25ee1278` e `acervo_editor.rs=82f31929d20bc2fbf9ea1b266c0580737d6ebc58`.

**Ensaio efetivo no Ifro:** com o Docker restabelecido, executei binário de teste **previamente compilado** do checkout `0a7df75d` em contêiner `rust:1.91-alpine`, rede desabilitada, limite 768 MB, árvore e cassettes read-only. `replay_editor_nativo_no_acervo --ignored --nocapture`: **1 PASS, rc 0**, 6 respostas originais UnB/GO recusadas como esperado. Filtro `documento_salvar_cli`: **4 PASS, 1 ignored, rc 0**. Controle adversarial: com dois ZIPs COPIADOS para pasta temporária privada, anexar um byte somente à cópia UnB produz `SHA-1 do ZIP não confere` e **rc 101 esperado**. Pasta removida; originais intactos. Um filtro com `--exact` e nome incompleto deu **0 testes/rc 0**: não confundir isso com PASS.

**Limite:** nenhuma recompilação no SHA 937b, nenhum replay CLI E2E, anti-IES↔original, SQLite ou salvamento nativo. [CI rápido #43798875598](https://github.com/franklinbaldo/ies/actions/runs/43798875598) tem steps reais e passou, mas **sem cargo/replay**. CI oficial no SHA final NOT_RUN. Cassette original não traz corpo do POST; HTTP 200 não prova edição. Manter Draft, sem novas guardas/especulação. Sem acesso SEI live, sem modificar branch, IAM ou gates. Parecer [R71](https://github.com/franklinbaldo/ies/pull/852#issuecomment-6906940594).

**Próximo discriminante:** replay da CLI real com fonte original e comparação independente com anti-IES/SQLite, em SHA final e CI oficial. Não repetir testes de recusa em cada novo commit de memória; registrar só mudança funcional ou evidência nova.
